import datetime
from typing import (
    List,
    Tuple,
    Union
)

from qdrant_client import (
    QdrantClient,
    AsyncQdrantClient
)
from qdrant_client import models
from loguru import logger
from onnxruntime import InferenceSession
import onnxruntime
from transformers import (
    AutoTokenizer,
    PreTrainedTokenizer,
    PreTrainedTokenizerFast
)

from .Lifespan import (
    Configurations, 
    CollectionNames
)
from base_models.ServiceManagement import IncludedKeysValues
from base_models.UserOperations import (
    SearchRequest,
    SearchResponse,
    SearchDocument,
    CollectionPayload,
    ListDocument
)


class QdrantVectorDatabaseOperations:
    
    def __init__(
        self, 
        configurations: Configurations, 
        asynchronous_qdrant_client: AsyncQdrantClient,
        tokenizer: Union[PreTrainedTokenizer, PreTrainedTokenizerFast],
        onnx_embedding_model: InferenceSession
    ) -> None:
        self.configurations = configurations
        self.asynchronous_qdrant_client = asynchronous_qdrant_client
        self.tokenizer = tokenizer
        self.onnx_embedding_model = onnx_embedding_model
    
    def __embedding(self, content: str) -> List[float]:
        input_tokens: List[str] = self.tokenizer(
            text=content,
            padding="longest",
            return_tensors="np"
        )
        
        inputs_to_onnx: dict = {
            k: onnxruntime.OrtValue.ortvalue_from_numpy(v) for k, v in input_tokens.items()
        }
        
        vector: List[float] = self.onnx_embedding_model.run(
            output_names=None,
            input_feed=inputs_to_onnx
        )
        
        return vector[1][0]

    async def create_collection(
        self,
        collection_name: str,
        owner_id: str
    ) -> Tuple[bool, int]:
        """
        create a collection record in the scalar database table for records. 
        """
        
        try:
            points_count: models.CountResult = await self.asynchronous_qdrant_client.count(
                collection_name=self.configurations.scalar_database_configurations.table_name
            )
            
            result: models.UpdateResult = await self.asynchronous_qdrant_client.upsert(
                collection_name=self.configurations.scalar_database_configurations.table_name,
                points=[
                    models.PointStruct(
                        id=points_count.count + 1,
                        vector=[0.0],
                        payload=IncludedKeysValues(
                            collection=collection_name,
                            owner_id=owner_id,
                            created_at=str(datetime.datetime.now()),
                        ).model_dump(),
                    )
                ]
            )
            
            logger.info(
                f"Created collection {collection_name} with owner_id {owner_id} and result {result}"
            )
            
            return True, points_count.count + 1
        
        except Exception as e:
            logger.error(
                f"Error creating collection {collection_name} with owner_id {owner_id}: {e}"
            )
            
            return False, 0
    
    async def delete_collection(
        self,
        collection_name: str, 
        existing_collection_names: CollectionNames
    ) -> bool:
        """
        delete the collection record from the scalar database table
        """
        try:
            result: models.UpdateResult = await self.asynchronous_qdrant_client.delete(
                collection_name=self.configurations.scalar_database_configurations.table_name,
                points_selector=models.FilterSelector(
                    filter=models.Filter(
                        should=[
                            models.FieldCondition(
                                key="collection",
                                match=models.MatchValue(value=collection_name)
                            )
                        ]
                    )
                )
            )
            
            existing_collection_names.delete_collection_name(
                collection_name=collection_name
            )
            
            logger.info(
                f"Deleted collection {collection_name} with result {result}"
            )
            
            return True
        
        except Exception as e:
            logger.error(
                f"Error deleting collection {collection_name}: {e}"
            )
            
            return False
    
    async def store_documents(
        self,
        existing_collection_names: CollectionNames,
        collection_name: str,
        content: str
    ) -> bool:
        
        points_count: models.CountResult = await self.asynchronous_qdrant_client.count(
            collection_name=self.configurations.notes_collections_configurations.notes_collections[0]
        )
        
        result: models.UpdateResult = await self.asynchronous_qdrant_client.upsert(
            collection_name=self.configurations.notes_collections_configurations.notes_collections[0],
            points=[
                models.PointStruct(
                    id=points_count.count + 1,
                    payload=CollectionPayload(
                        collection_name=collection_name,
                        content=content,
                    ).model_dump(),
                    vector=self.__embedding(content=content),
                )
            ]
        )
        
        logger.info(
            f"Stored document {content} in collection {collection_name} with result {result}"
        )
        
        existing_collection_names.add_collection_name(
            collection_name=collection_name
        )
        
        return True
    
    async def search_documents(
        self, 
        collection_name: str, 
        query: str,
        top_n: int
    ) -> List[SearchDocument]:
        
        results: List[models.ScoredPoint] = await self.asynchronous_qdrant_client.search(
            collection_name=self.configurations.notes_collections_configurations.notes_collections[0],
            query_vector=self.__embedding(content=query),
            limit=top_n,
            query_filter=models.Filter(
                must=[
                    models.FieldCondition(
                        key="collection_name",
                        match=models.MatchValue(
                            value=collection_name
                        )
                    )
                ]
            )
        )
        
        return [
            SearchDocument(
                id=result.id,
                content=result.payload['content'],
                relevance_score=result.score,
            )
            for result in results
        ]
    
    async def delete_documents(
        self, 
        ids: List[int],
        collection_name: str, 
    ) -> List[SearchDocument]:
        
        result: models.UpdateResult = await self.asynchronous_qdrant_client.delete(
            collection_name=self.configurations.notes_collections_configurations.notes_collections[0],
            points_selector=models.PointIdsList(
                points=ids
            )
        )
        
        if result.status == models.UpdateStatus.COMPLETED:
            return True
        
        else:
            return False
    
    async def list_collection_entries(self, collection_name: str) -> List[SearchDocument]:
        """
        a method to get all contents out of a collection. 
        for now, we only use the first specified collection among the list in 
        the configurations. 
        """
        
        points_count: models.CountResult = await self.asynchronous_qdrant_client.count(
            collection_name=self.configurations.notes_collections_configurations.notes_collections[0],
            count_filter=models.Filter(
                must=[
                    models.FieldCondition(
                        key="collection_name",
                        match=models.MatchValue(
                            value=collection_name
                        )
                    )
                ]
            )
        )
        
        results: Tuple[List[models.ScoredPoint], int] = await self.asynchronous_qdrant_client.scroll(
            collection_name=self.configurations.notes_collections_configurations.notes_collections[0],
            limit=points_count.count,
            with_payload=True,
            with_vectors=False
        )
        
        return [
            ListDocument(
                id=result.id,
                content=result.payload['content']
            )
            for result in results[0]
        ]