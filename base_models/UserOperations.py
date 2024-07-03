from typing import (
    List,
    Union,
    Literal,
    Optional
)

from pydantic import BaseModel


# a basic class object to hold data to be returned
class BasicResponse(BaseModel):
    
    status: Literal['success', 'error']
    message: str
    data: Optional[dict] = {}


class CreateCollectionRequest(BaseModel):
    
    collection_name: str
    owner_id: str


class SearchRequest(BaseModel):
    
    collection_name: str
    query: str
    top_n: int


class SearchDocument(BaseModel):
    
    id: Union[int | str]
    content: str
    relevance_score: float


class DeleteDocumentsRequest(BaseModel):
    
    collection_name: str
    document_ids: List[int]


class ListDocument(BaseModel):
    
    id: Union[int | str]
    content: str


# put this into the `BasicResponse`'s data key
class SearchResponse(BasicResponse):
    
    documents: List[SearchDocument]


class StoreDocumentRequest(BaseModel):
    
    collection_name: str
    content: str


class CollectionPayload(BaseModel):
    
    numeric_id: int
    collection_name: str
    content: str


class WholeCollectionOperationBaseRequest(BaseModel):
    """
    since when operating on a collection requires a collection name anyway,
    this base request class provides common fields for all operations on a 
    collection
    """
    
    collection_name: str