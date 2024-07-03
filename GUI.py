from nicegui import ui

from utilities.QdrantVectorDatabaseOperationsSynchronous import (
    QdrantVectorDatabaseOperations
)
from ui_controls.UIControllers import (
    UIController
)
from ui_controls.MainPage import (
    MainPage
)
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
    qdrant_client=synchronous_qdrant_client,
    tokenizer=tokenizer,
    onnx_embedding_model=onnx_session
)

# check what collections that we currently have
collection_names = CollectionNames()
collection_names.load_collection_names()

# initiate a stateful ui controller
ui_controller = UIController(
    operations_object=operations_object, 
    collection_names=collection_names
)


with ui.row():
    
    # initiate the main page
    main_page = MainPage(
        ui_controller=ui_controller,
        collection_names=collection_names, 
        operations_object=operations_object
    )
    
ui.run(
    host="0.0.0.0",
    port=10000,
    title="Text Explorer"
)