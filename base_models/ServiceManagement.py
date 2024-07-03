from typing import (
    Dict,
    List,
    Literal
)

from pydantic import BaseModel


"""
NotesCollections
"""


class NotesCollectionsVectorParameters(BaseModel):
    
    size: int
    distance: Literal['cosine']


class NotesCollectionsConfigurations(BaseModel):
    
    notes_collections: List[str]
    notes_collections_vector_parameters: NotesCollectionsVectorParameters
    text_split_maximum_size: int


"""
Qdrant
"""


class QdrantConfigurations(BaseModel):
    
    host: str
    port: int


"""
Embedding Configurations
"""


class EmbeddingModelConfigurations(BaseModel):
    
    model_path: str
    tokenizer_path: str
    onnx_providers: List[
        Literal['CPUExecutionProvider', 'CUDAExecutionProvider']
    ]


"""
Scalar Database Configurations
"""


class IncludedKeysValues(BaseModel):
    
    collection: str
    owner_id: str
    created_at: str


class ScalarDatabaseConfigurations(BaseModel):
    
    database_type: Literal['qdrant']
    table_name: str


"""
Configurations
"""


class Configurations(BaseModel):
    
    host: str
    port: int
    log_level: Literal['info', 'debug', 'warning', 'error']
    qdrant_configurations: QdrantConfigurations
    notes_collections_configurations: NotesCollectionsConfigurations
    scalar_database_configurations: ScalarDatabaseConfigurations
    embedding_model_configurations: EmbeddingModelConfigurations