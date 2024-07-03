import asyncio
from typing import (
    List,
    Optional
)

from nicegui import ui

from utilities.QdrantVectorDatabaseOperationsSynchronous import (
    QdrantVectorDatabaseOperations
)
from utilities.Lifespan import (
    CollectionNames
)


def async_to_sync(awaitable) -> any:
    loop = asyncio.get_event_loop()
    return loop.run_until_complete(awaitable)


class UIController:
    
    def __init__(
        self, 
        operations_object: QdrantVectorDatabaseOperations, 
        collection_names: CollectionNames
    ) -> None:
        self.operations_object = operations_object
        
        self.active_document: str = ""
        self.active_document_state: str = ""
        self.search_results: List[dict] = []
        
        self.collection_names: CollectionNames = collection_names
        self.documents_list: ui.list = ui.list().props("bordered separator")
    
    def get_all_documents(
        self, 
        collection_name: str, 
        textarea: ui.textarea
    ) -> None:
        
        # reset the document state first
        self.active_document_state = ""
        
        values = self.operations_object.list_collection_entries(
            collection_name=collection_name
        )
        
        for value in values:
            self.active_document_state += value.content
        
        textarea.value = ""
        textarea.update()
        
        textarea.value = self.active_document_state
        textarea.update()
        
        self.active_document = collection_name
    
    def search_similar_contents(
        self,
        input_text: str, 
        table: ui.table, 
        collection_name: Optional[str] = None,
    ) -> None:
        
        if collection_name == None:
            collection_name = self.active_document
        
        results = self.operations_object.search_documents(
            collection_name=collection_name,
            query=input_text, 
            top_n=10
        )
        
        # reset the rows of the table
        table.rows = []
        
        for result in results:
            table.rows.append(
                result.model_dump()
            )
        
        table.update()
    
    def rewrite_document(
        self, 
        document_text: str, 
        collection_names: CollectionNames
    ) -> None:
        
        self.operations_object.delete_collection(
            collection_name=self.active_document, 
            existing_collection_names=collection_names
        )
        
        self.operations_object.comprehensive_document_storage(
            existing_collection_names=collection_names,
            collection_name=self.active_document,
            content=document_text
        )
    
    def new_document(
        self, 
        collection_name: str, 
        collection_names: CollectionNames, 
        textarea: ui.textarea,
        content: Optional[str] = "input your content here...", 
    ) -> None:
        
        self.operations_object.comprehensive_document_storage(
            existing_collection_names=collection_names,
            collection_name=collection_name,
            content=content
        )
        
        self.update_ui_list(
            textarea=textarea
        )
    
    def delete_collection(
        self, 
        collection_name: str, 
        textarea: ui.textarea
    ) -> None:
        self.operations_object.delete_collection(
            collection_name=collection_name,
            existing_collection_names=self.collection_names
        )
        
        self.update_ui_list(
            textarea=textarea
        )
    
    def update_ui_list(
        self,
        textarea: ui.textarea,
    ) -> None:
        self.documents_list.clear()  # Clear existing content in the list
        
        with self.documents_list:
            ui.item_label('Documents').props('header').classes('text-bold')
            ui.separator()
            
            for item in self.operations_object.retrieve_collection_names():
                with ui.item(
                    on_click=lambda item=item: self.get_all_documents(
                        collection_name=item, 
                        textarea=textarea
                    )
                ) as table:
                    with ui.item_section():
                        ui.item_label(item)
                    with ui.item_section().props('side'):
                        # Assuming deletion needs to refresh the list too
                        ui.button(
                            icon='delete',
                            color='red',
                            on_click=lambda item=item: self.delete_collection(
                                collection_name=item,
                                textarea=textarea
                            )
                        )