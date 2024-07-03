from contextlib import asynccontextmanager
from typing import (
    AsyncGenerator,
    Tuple,
    Union,
    List
)
import json
import os

from qdrant_client import (
    QdrantClient,
    AsyncQdrantClient
)
from qdrant_client import models
from qdrant_client.http import exceptions
from onnxruntime import InferenceSession
from transformers import (
    AutoTokenizer,
    PreTrainedTokenizer,
    PreTrainedTokenizerFast
)
from loguru import logger

from base_models.ServiceManagement import Configurations


def load_configurations() -> Configurations:
    
    """
    a method to initialize the configurations for the service.
    """
    
    try:
        with open(
            file='./configurations/Configurations.json', 
            mode='r', 
            encoding='utf-8'
        ) as config_file:
            data: dict = json.load(fp=config_file)
            
            logger.info(
                "Configurations loaded successfully."
            )
            
            return Configurations(**data)
    
    except Exception as e:
        logger.error(
            "Configurations could not be loaded. Exiting."
        )
        exit(1)

def initialize_qdrant_client(configurations: Configurations) -> Tuple[
    AsyncQdrantClient, QdrantClient
]:
    
    # acquire qdrant client first
    asynchronous_qdrant_client = AsyncQdrantClient(
        host=configurations.qdrant_configurations.host,
        port=configurations.qdrant_configurations.port
    )
    synchronous_qdrant_client = QdrantClient(
        host=configurations.qdrant_configurations.host,
        port=configurations.qdrant_configurations.port
    )
    
    # try to initialize the specified collections
    if configurations.notes_collections_configurations.notes_collections_vector_parameters.distance == "cosine":
        distance = models.Distance.COSINE
    
    for collection in configurations.notes_collections_configurations.notes_collections:
        try:
            synchronous_qdrant_client.create_collection(
                collection_name=collection,
                vectors_config=models.VectorParams(
                    size=configurations.notes_collections_configurations.notes_collections_vector_parameters.size,
                    distance=distance
                )
            )
            
            logger.info(
                f"Collection {collection} initialized successfully."
            )
        
        except exceptions.UnexpectedResponse:
            logger.warning(
                f"Collection {collection} has been created already. Skipped."
            )
            continue
    
    try:
        synchronous_qdrant_client.create_collection(
            collection_name=configurations.scalar_database_configurations.table_name,
            vectors_config=models.VectorParams(
                size=1,
                distance=models.Distance.COSINE
            )
        )
        
        logger.info(
            f"Scalar table {configurations.scalar_database_configurations.table_name} initialized successfully."
        )
    
    except exceptions.UnexpectedResponse:
        logger.warning(
            f"Collection {configurations.scalar_database_configurations.table_name} has been created already. Skipped."
        )
    
    return asynchronous_qdrant_client, synchronous_qdrant_client

def initialize_embedding_model(configurations: Configurations) -> Tuple[
    InferenceSession, Union[PreTrainedTokenizer,PreTrainedTokenizerFast]
]:
    """
    load embedding model and tokenizer upon service startup
    """
    try:
        tokenizer: PreTrainedTokenizer | PreTrainedTokenizerFast = AutoTokenizer.from_pretrained(
            pretrained_model_name_or_path=configurations.embedding_model_configurations.tokenizer_path
        )
        
        onnx_session: InferenceSession = InferenceSession(
            path_or_bytes=configurations.embedding_model_configurations.model_path,
            providers=configurations.embedding_model_configurations.onnx_providers
        )
        
        logger.info(
            "Embedding model initialized successfully."
        )
        
        return onnx_session, tokenizer
    
    except Exception as e:
        logger.error(
            f"Embedding model could not be initialized. Exiting. {e}"
        )
        exit(1)


class CollectionNames:
    
    def __init__(self) -> None:
        self.file_dictionary: dict = {
            "collections": []
        }
    
    def load_collection_names(
        self,
        collection_names_filepath: str = "./configurations/CollectionNames.json"
    ) -> None:
        
        # check if the directory exists
        os.path.exists(
            collection_names_filepath
        )
        
        with open(collection_names_filepath, 'r+') as file:
            file_dict: dict = json.loads(file.read())
        
        # check if the `collections` key exists, otherwise returns an empty list
        if file_dict.get("collections", None) is None:
            self.file_dictionary["collections"] = []
        
        else:
            self.file_dictionary['collections'] = file_dict['collections']
    
    def add_collection_name(self, collection_name: str) -> None:
        # save the collections to the file
        if collection_name not in self.file_dictionary['collections']:
            self.file_dictionary['collections'].append(collection_name)
        
        self.write_file()
    
    def delete_collection_name(self, collection_name: str) -> None:
        # delete the collection from the file
        if collection_name in self.file_dictionary['collections']:
            self.file_dictionary['collections'].remove(collection_name)
        
        self.write_file()
    
    def write_file(self, path: str = "./configurations/CollectionNames.json") -> None:
        with open(path, 'w') as file:
            json.dump(self.file_dictionary, file)