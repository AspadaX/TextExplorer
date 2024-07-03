from nicegui import ui

from utilities.QdrantVectorDatabaseOperationsSynchronous import (
    QdrantVectorDatabaseOperations
)
from ui_controls.UIControllers import UIController
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
from ui_controls.UIControllers import (
    UIController
)


class MainPage:
    
    def __init__(
        self, 
        ui_controller: UIController, 
        collection_names: CollectionNames, 
        operations_object: QdrantVectorDatabaseOperations
    ) -> None:
    
        # automate dark mode switch
        self.dark = ui.dark_mode(
            value=None
        )
        self.ui_controller = ui_controller
        self.collection_names = collection_names
        self.operations_object = operations_object
        
        # UIs
        with ui.card():
            self.textarea = ui.textarea(
                placeholder='Enter your text here...', 
                value=ui_controller.active_document_state
            ).props('clearable').on(
                type='keydown.enter', 
                handler=lambda: ui_controller.rewrite_document(
                    document_text=self.textarea.value,
                    collection_names=self.collection_names
                )
            )
        
        with ui.card():
            self.documents_table: ui.item = self.ui_controller.update_ui_list(
                textarea=self.textarea
            )
            
            new_document_name = ui.input(
                label='Name', 
                placeholder='Input your new document name here...'
            )
            
            ui.button(
                text='New Document', 
                on_click=lambda: self.ui_controller.new_document(
                    collection_name=new_document_name.value,
                    collection_names=self.collection_names,
                    content=self.textarea.value, 
                    textarea=self.textarea
                )
            )
        
        with ui.card():
            self.search_result = ui.table(
                columns=[
                    {
                        "name": "relevance_score", 
                        "label": "Relevance", 
                        "field": "relevance_score", 
                        "align": "left",
                        "sortable": True
                    },
                    {
                        "name": "content", 
                        "label": "Content", 
                        "field": "content", 
                        "align": "left"
                    }
                ],
                rows=[], 
                row_key='id',
            )
            
            self.search_bar = ui.input(
                label='SearchBar', 
                placeholder='Type what you want to know about...'
            ).on(
                type='keydown.enter', 
                handler=lambda: ui_controller.search_similar_contents(
                    input_text=self.search_bar.value,
                    table=self.search_result
                )
            )
    
    def display_documents_table(self, textarea: ui.textarea) -> None:
        with ui.card():
            with ui.list().props("bordered separator") as documents_list:
                documents_list.clear()  # Clear existing content in the list
                ui.item_label('Documents').props('header').classes('text-bold')
                ui.separator()
                
                for item in self.operations_object.retrieve_collection_names():
                    with ui.item(
                        on_click=lambda item=item: self.ui_controller.get_all_documents(
                            collection_name=item, 
                            textarea=textarea
                        )
                    ):
                        with ui.item_section():
                            ui.item_label(item)
                        with ui.item_section().props('side'):
                            # Assuming deletion needs to refresh the list too
                            ui.button(
                                icon='delete',
                                color='red',
                                on_click=lambda item=item: self.ui_controller.delete_collection(
                                    collection_name=item,
                                    textarea=textarea
                                )
                            )
            
            new_document_name = ui.input(
                label='Name', 
                placeholder='Input your new document name here...'
            )
            
            ui.button(
                text='New Document', 
                on_click=lambda: self.ui_controller.new_document(
                    collection_name=new_document_name.value,
                    collection_names=self.collection_names,
                    content=textarea.value, 
                    textarea=textarea
                )
            )