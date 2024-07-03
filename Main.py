# TODO: Indexing api: chunk the input article then vectorize them respectively. 


from contextlib import asynccontextmanager
from typing import (
    AsyncGenerator, 
    List
)

import tqdm
from semantic_text_splitter import TextSplitter
from fastapi import FastAPI
from fastapi.logger import logger
from fastapi.responses import JSONResponse
from fastapi.middleware.cors import CORSMiddleware

from utilities.Lifespan import (
    AsyncQdrantClient,
    load_configurations,
    initialize_qdrant_client,
    initialize_embedding_model,
    CollectionNames
)
from utilities.Lifespan import (
    Configurations, 
    CollectionNames
)
from utilities.QdrantVectorDatabaseOperations import (
    QdrantVectorDatabaseOperations
)
from base_models.ServiceManagement import Configurations
from base_models.UserOperations import (
    CreateCollectionRequest,
    BasicResponse,
    SearchDocument,
    SearchRequest,
    SearchResponse,
    StoreDocumentRequest,
    WholeCollectionOperationBaseRequest,
    ListDocument,
    DeleteDocumentsRequest
)

@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator:
    yield
    
    collection_names.write_file()

app = FastAPI(
    logger=logger, 
    lifespan=lifespan
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

"""
API Routes
"""

@app.post("/user_operations/create_collection")
async def create_collection(parameters: CreateCollectionRequest) -> JSONResponse:
    result, collection_id = await operations_object.create_collection(
        collection_name=parameters.collection_name,
        owner_id=parameters.owner_id
    )
    
    if result:
        return JSONResponse(
            status_code=200,
            content=BasicResponse(
                status='success',
                message='Collection created successfully',
                data={
                    "collection_id": collection_id
                }
            ).model_dump()
        )
    
    else:
        return JSONResponse(
            status_code=400,
            content=BasicResponse(
                status='error',
                message='Collection creation failed',
                data={}
            ).model_dump()
        )

@app.put("/user_operations/search")
async def search(parameters: SearchRequest) -> JSONResponse:
    return JSONResponse(
        status_code=200,
        content=BasicResponse(
            status='success',
            message='Search completed successfully',
            data={
                "documents": [
                    document.model_dump()
                    for document in await operations_object.search_documents(
                        collection_name=parameters.collection_name,
                        query=parameters.query,
                        top_n=parameters.top_n
                    )
                ]
            }
        ).model_dump()
    )

@app.post("/user_operations/store_document")
async def store_document(parameters: StoreDocumentRequest) -> JSONResponse:
    # TODO: link this route with creating collections
    if await operations_object.store_documents(
        existing_collection_names=collection_names,
        collection_name=parameters.collection_name,
        content=parameters.content
    ):
        return JSONResponse(
            status_code=200,
            content=BasicResponse(
                status='success',
                message='Document stored successfully',
            ).model_dump()
        )
    
@app.post("/user_operations/comprehensive_store_document")
async def comprehensive_store_document(parameters: StoreDocumentRequest) -> JSONResponse:
    
    # initialize a text splitter
    splitter = TextSplitter(
        capacity=configurations.notes_collections_configurations.text_split_maximum_size
    )
    
    # split texts
    splitted_texts: List[str] = splitter.chunks(
        text=parameters.content
    )
    
    results: List[bool] = []
    for text in tqdm.tqdm(splitted_texts):
        
        result: bool = await operations_object.store_documents(
            existing_collection_names=collection_names,
            collection_name=parameters.collection_name,
            content=text
        )
        
        results.append(result)
    
    if all(results):
        return JSONResponse(
            status_code=200,
            content=BasicResponse(
                status='success',
                message='Document stored successfully',
            ).model_dump()
        )
    
    else:
        return JSONResponse(
            status_code=400,
            content=BasicResponse(
                status='error',
                message='Document storage failed',
            ).model_dump()   
        )
    
@app.delete("/user_operations/delete_collection")
async def delete_collection(parameters: WholeCollectionOperationBaseRequest) -> JSONResponse:
    if await operations_object.delete_collection(
        collection_name=parameters.collection_name,
        existing_collection_names=collection_names
    ):
        return JSONResponse(
            status_code=200,
            content=BasicResponse(
                status='success',
                message='Document deleted successfully'
            ).model_dump()
        )
    else:
        return JSONResponse(
            status_code=400,
            content=BasicResponse(
                status='error',
                message='Document deletion failed'
            ).model_dump()
        )

@app.delete("/user_operations/delete_documents")
async def delete_documents(parameters: DeleteDocumentsRequest) -> JSONResponse:
    if await operations_object.delete_documents(
        ids=parameters.document_ids,
        collection_name=parameters.collection_name
    ):
        return JSONResponse(
            status_code=200,
            content=BasicResponse(
                status='success',
                message='Document deleted successfully'
            ).model_dump()
        )
    else:
        return JSONResponse(
            status_code=400,
            content=BasicResponse(
                status='error',
                message='Document deletion failed'
            ).model_dump()
        )

@app.post("/user_operations/update_document")
async def update_document() -> JSONResponse:
    ...

@app.put("/user_operations/list_documents")
async def list_documents(parameters: WholeCollectionOperationBaseRequest) -> JSONResponse:
    return JSONResponse(
        status_code=200,
        content=BasicResponse(
            status='success',
            message='Collection contents retrieved successfully.',
            data={
                "documents": [
                    document.model_dump()
                    for document in await operations_object.list_collection_entries(
                        collection_name=parameters.collection_name
                    )
                ]
            }
        ).model_dump()
    )

@app.get("/user_operations/list_collections")
async def list_collections() -> JSONResponse:
    return JSONResponse(
        status_code=200,
        content=BasicResponse(
            status='success',
            message='Collection list retrieved successfully.',
            data=collection_names.file_dictionary
        ).model_dump()
    )


if __name__ == "__main__":
    
    import uvicorn
    
    # load configurations
    configurations: Configurations = load_configurations()
    
    # initialize the qdrant client and the specified collections
    # docker run -p 6333:6333 -p 6334:6334 -v /home/xinyubao/qdrant/qdrant_storage:/qdrant/storage:z qdrant/qdrant
    asynchronous_qdrant_client, synchronous_qdrant_client = initialize_qdrant_client(
        configurations=configurations
    )
    
    # load embedding model and tokenizer
    onnx_session, tokenizer = initialize_embedding_model(
        configurations=configurations
    )
    
    # initialize the vector database operations object
    operations_object = QdrantVectorDatabaseOperations(
        configurations=configurations,
        asynchronous_qdrant_client=asynchronous_qdrant_client,
        tokenizer=tokenizer,
        onnx_embedding_model=onnx_session
    )
    
    # check what collections that we currently have
    collection_names = CollectionNames()
    collection_names.load_collection_names()
    
    uvicorn.run(
        app=app, 
        host=configurations.host, 
        port=configurations.port,
        log_level=configurations.log_level
    )