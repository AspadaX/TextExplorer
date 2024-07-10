mod vectorization;
mod base_models;
mod utilities;
mod models;
mod schema;
mod database;

use std::env;
use actix_web::dev::Path;
use base_models::basics::BasicResponse;
use base_models::document::{GetAllDocumentsResponse, GetDocumentByIDResponse};
use dotenvy::dotenv;
use models::Document;
use core::panic;
use std::time::Duration;
use std::sync::Arc;

use actix_web::{
    web, 
    App, 
    HttpResponse, 
    HttpServer, 
    Responder
};
use actix_web::middleware::Logger;
use log::{error, info};
use actix_cors::Cors;

use utilities::vector_database_interfaces::{
    AsynchronousVectorDatabaseInterface, 
    AsynchronousQdrantVectorDatabaseBackend
};
use utilities::configurations::load_configurations;
use utilities::parsers::{BasicParser, Parser};


#[actix_web::post("/document/create")]
async fn create_document(
    request: web::Json<base_models::document::CreateDocumentRequest>,
    data: web::Data<Arc<vectorization::text::TextVectorization>>, 
    configurations: web::Data<Arc<serde_json::Value>>, 
    vector_database_backend: web::Data<Arc<AsynchronousQdrantVectorDatabaseBackend>>
) -> impl Responder {

    // create a document in the SQL database
    let document = match models::Document::new(
        request.title.clone(), 
        request.full_text.clone()
    ) {
        Ok(result) => result, 
        Err(_)  => return HttpResponse::InternalServerError().finish()
    };

    // parse the document
    let parsed_contents = BasicParser::new(
        request.maximum_text_split_size as usize, 
        request.full_text.clone(), 
        &data
    ); 

    let document_id = document.id;
    // store the document to the vector database
    match vector_database_backend.create_document(
        &document_id, 
        &parsed_contents.text_slices, 
        configurations.get("chunk_factor").unwrap().as_f64().unwrap() as f32,
        &parsed_contents.vectors, 
        &parsed_contents.slice_ids
    ).await {
        Some(error) => {
            error!("{}", error);
            return HttpResponse::InternalServerError().json(
                BasicResponse {
                    status: false,
                    message: error.to_string(),
                    data: None::<String>
                }
            )
        }, 
        None => {}
    };

    // return the result
    let response = base_models::document::CreateDocumentResponse::new(
        document_id,
        base_models::document::DocumentStatus::Ready
    );

    return HttpResponse::Ok().json(response);

}


#[actix_web::post("/document/read/vector_search")]
async fn vector_search_document(
    request: web::Json<base_models::document::SearchDocumentRequest>,
    data: web::Data<Arc<vectorization::text::TextVectorization>>, 
    vector_database_backend: web::Data<Arc<AsynchronousQdrantVectorDatabaseBackend>>
) -> impl Responder {

    // vectorize the input 
    let input_vector = match data.vectorize(&request.input_text, true) {
        Ok(result) => result,
        Err(error) => {
            error!("{:?}", error);
            return HttpResponse::InternalServerError().json(
                BasicResponse {
                    status: false, 
                    message: error.to_string(), 
                    data: None::<String>
                }
            )
        }
    }; 

    // perform a vector search 
    let _query_results = match vector_database_backend.search_document(
        &request.document_id, 
        &input_vector,
        &request.top_n
    ).await {
        Ok(result) => return HttpResponse::Ok().json(
            BasicResponse {
                status: true, 
                message: "Vector search operation succeeded.".to_string(), 
                data: Some(result)
            }
        ), 
        Err(error) => {
            error!("{}", error.to_string());
            return HttpResponse::InternalServerError().json(
                BasicResponse {
                    status: false, 
                    message: error.to_string(), 
                    data: None::<String>
                }
            );
        }
    };

}


#[actix_web::post("/document/update")]
async fn update_document(
    request: web::Json<base_models::document::UpdateDocumentRequest>,
    data: web::Data<Arc<vectorization::text::TextVectorization>>, 
    configurations: web::Data<Arc<serde_json::Value>>, 
    vector_database_backend: web::Data<Arc<AsynchronousQdrantVectorDatabaseBackend>>
) -> impl Responder {

    // delete the original document from the sql database first
    match Document::delete(&request.document_id) {
        Some(error) => {
            error!("{}", error);
            return HttpResponse::InternalServerError().json(
                BasicResponse {
                    status: false, 
                    message: error.to_string(), 
                    data: None::<String>
                }
            ) 
        }, 
        None => {}
    };

    // delete the original document from the vector database first
    match vector_database_backend.delete_document(&request.document_id).await {
        Some(error) => {
            error!("{}", error);
            return HttpResponse::InternalServerError().json(
                BasicResponse {
                    status: false, 
                    message: error.to_string(), 
                    data: None::<String>
                }
            ) 
        }, 
        None => {} 
    };

    // create a new document in the SQL database
    let document = match models::Document::new(
        request.updated_parameters.title.clone(), 
        request.updated_parameters.full_text.clone()
    ) {
        Ok(result) => result, 
        Err(_)  => return HttpResponse::InternalServerError().finish()
    };

    // parse the new document
    let parsed_contents = BasicParser::new(
        request.updated_parameters.maximum_text_split_size as usize, 
        request.updated_parameters.full_text.clone(), 
        &data
    ); 

    // store the document to the vector database
    let document_id = document.id;
    match vector_database_backend.create_document(
        &document_id, 
        &parsed_contents.text_slices, 
        configurations.get("chunk_factor").unwrap().as_f64().unwrap() as f32,
        &parsed_contents.vectors, 
        &parsed_contents.slice_ids
    ).await {
        Some(error) => {
            error!("{}", error);
            return HttpResponse::InternalServerError().json(
                BasicResponse {
                    status: false,
                    message: error.to_string(),
                    data: None::<String>
                }
            )
        }, 
        None => {}
    };

    // return the result
    let response = base_models::document::CreateDocumentResponse::new(
        document_id,
        base_models::document::DocumentStatus::Ready
    );

    return HttpResponse::Ok().json(response); 
}


#[actix_web::delete("/document/delete")]
async fn delete_document(
    request: web::Json<base_models::document::DocumentDeleteRequest>,
    vector_database_backend: web::Data<Arc<AsynchronousQdrantVectorDatabaseBackend>>
) -> impl Responder {

    match Document::delete(&request.document_id) {
        Some(error) => {
            error!("{}", error);
            return HttpResponse::InternalServerError().json(
                BasicResponse {
                    status: false, 
                    message: error.to_string(), 
                    data: None::<String>
                }
            ) 
        }, 
        None => {}
    };

    match vector_database_backend.delete_document(&request.document_id).await {
        Some(error) => {
            error!("{}", error);
            return HttpResponse::InternalServerError().json(
                BasicResponse {
                    status: false, 
                    message: error.to_string(), 
                    data: None::<String>
                }
            ) 
        }, 
        None => return HttpResponse::Ok().json(
            BasicResponse {
                status: true, 
                message: "Document was successfully deleted.".to_string(), 
                data: None::<String>
            }
        )
    };

}


#[actix_web::get("/document/read/get_all_documents")]
async fn retrieve_all_documents_title_id() -> impl Responder {
    match Document::get_all_documents() {
        Ok(result) => {
            let mut retrieved_documents: Vec<GetAllDocumentsResponse> = Vec::new();
            for item in result {
                retrieved_documents.push(
                    GetAllDocumentsResponse {
                        id: item.id,
                        title: item.title
                    }
                );
            }
            return HttpResponse::Ok().json(
                BasicResponse {
                    status: true, 
                    message: "All documents retrieved successfully.".to_string(), 
                    data: Some(retrieved_documents)
                }
            );
        }, 
        Err(error) => {
            error!("{}", error);
            return HttpResponse::Ok().json(
                BasicResponse {
                    status: false, 
                    message: error.to_string(),
                    data: None::<String>
                }
            )
        }
    };
}

#[actix_web::get("/document/read/{document_id}")]
async fn get_document_by_id_api(path: actix_web::web::Path<String>) -> impl Responder {
    let document_id = path.into_inner();
    match Document::get_document_by_id(&document_id) {
        Ok(result) => return HttpResponse::Ok().json(
                BasicResponse {
                    status: true, 
                    message: "Document retrieved successfully.".to_string(), 
                    data: Some(
                        GetDocumentByIDResponse {
                            id: result.id, 
                            title: result.title, 
                            full_text: result.full_text
                        }
                    )
                }
            ),
        Err(error) => {
            error!("{}", error);
            return HttpResponse::Ok().json(
                BasicResponse {
                    status: false, 
                    message: error.to_string(),
                    data: None::<String>
                }
            )
        }
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(
        env_logger::Env::new()
            .default_filter_or("info")
    );

    // load local configuration
    dotenv().ok();
    let configurations = match load_configurations(
        &env::var("LOCAL_CONFIGURATION_FILE").unwrap()
    ) {
        Ok(result) => result, 
        Err(_) => panic!(
            "Failed to load the local configuration file. Please double check. Abort startup..."
        )
    };

    // have this for main() to access
    let configurations_for_main = Arc::new(
        configurations
    );

    let configurations_for_routes = configurations_for_main
        .clone();
    
    let embedding_models_group = match vectorization::text::TextVectorization::new(
        &configurations_for_main.get("embedding_model_path").unwrap().as_str().unwrap(), 
        &configurations_for_main.get("embedding_tokenizer_path").unwrap().as_str().unwrap(), 
        ort::GraphOptimizationLevel::Level3
    ) {
        Ok(embedding_models_group) => Arc::new(
            embedding_models_group
        ),
        Err(error) => panic!(
            "{:?}",
            error
        )
    };

    // initialize a vector database backend
    let vector_database_backend = match AsynchronousQdrantVectorDatabaseBackend::new(
        &configurations_for_main.get("vector_database_url").unwrap().as_str().unwrap().to_string(), 
        &configurations_for_main.get("documents_collection").unwrap().as_str().unwrap().to_string(), 
        &configurations_for_main.get("vector_dimensionality").unwrap().as_u64().unwrap()
    ).await {
        Ok(result) => Arc::new(result),
        Err(error) => panic!(
            "{}", error.to_string()
        )
    };

    info!(
        "Host from configurations: {}",
        configurations_for_main.get("host").unwrap().to_string()
    );

    info!(
        "Port from configurations: {}",
        configurations_for_main.get("port").unwrap().as_u64().unwrap() as u16
    );

    HttpServer::new(
        move || {
            App::new()
                .wrap(Logger::default())
                .wrap(Cors::permissive())
                .app_data(
                    web::Data::new(embedding_models_group.clone())
                )
                .app_data(
                    web::Data::new(
                        configurations_for_routes.clone()
                    )
                )
                .app_data(
                    web::Data::new(
                        vector_database_backend.clone()
                    )
                )
                .service(create_document)
                .service(vector_search_document)
                .service(update_document)
                .service(delete_document)
                .service(retrieve_all_documents_title_id)
                .service(get_document_by_id_api)
        }
    )
        .client_request_timeout(
            Duration::from_secs(0)
        )
        .client_disconnect_timeout(
            Duration::from_secs(0)
        )
        .max_connection_rate(256)
        .bind(
            (
                configurations_for_main.get("host")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
                configurations_for_main.get("port")
                    .unwrap()
                    .as_u64()
                    .unwrap() as u16
            )
        )?
        .run()
        .await
}