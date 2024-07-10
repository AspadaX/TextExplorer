/*
    this file contains everything related to requests on `document` 
*/
use core::fmt;
use chrono::prelude::*;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::{
    utilities::vector_database_interfaces::QueryResult,
    models::Document, 
    database::connections::establish_connection
};


// Enum for all errors of `Document`
pub enum DocumentErrors {
    CreateNewDocumentError, 
    DeletionError
}

impl fmt::Display for DocumentErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            DocumentErrors::CreateNewDocumentError => write!(
                f, "Error happens when creating a new document, you may want to check the connection to the database"
            ), 
            DocumentErrors::DeletionError => write!(
                f, "When trying to delete the document, an error has ocurred. "
            )
        }
    }
}

// add additional features to the `Document`
impl Document {
    pub fn new(title: String, full_text: String) -> Result<Self, DocumentErrors> {
        /*
            this method does the following: 
            1. generate a new Document object
            2. create a database entry for the created document
            3. map the new Document to the vector database
            4. return the new Document object for other parts of the program to process
        */
        let an_entry_of_document = Document {
            id: Uuid::new_v4().to_string(), 
            title: title, 
            full_text: full_text, 
            created_at: NaiveDateTime::parse_from_str(
                &Local::now()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(), 
                "%Y-%m-%d %H:%M:%S"
            ).unwrap()
        };

        // operation with the database
        let connection = &mut establish_connection();
        let rows_affected: usize = diesel::insert_into(crate::schema::documents::dsl::documents)
            .values(&an_entry_of_document)
            .execute(connection)
            .expect("Error happened when trying to create a new document");

        if rows_affected > 0 {
            return Ok(an_entry_of_document);
        } else {
            return Err(DocumentErrors::CreateNewDocumentError);
        }
    }

    pub fn delete(id: &str) -> Option<DocumentErrors> {
        let connection = &mut establish_connection();
        let rows_affected: usize = diesel::delete(
            crate::schema::documents::dsl::documents.filter(
                crate::schema::documents::id.eq(id.clone())
            )
        )
            .execute(connection)
            .expect("Error when deleting the Document");

        if rows_affected == 1 {
            return None; 
        } else {
            return Some(DocumentErrors::DeletionError);
        }
    }

    pub fn get_all_documents() -> Result<Vec<Document>, diesel::result::Error> {
        let connection = &mut establish_connection();
        
        return crate::schema::documents::dsl::documents.load::<Document>(
            connection
        );
    }

    pub fn get_document_by_id(document_id: &str) -> Result<Document, diesel::result::Error> {
        let connection = &mut establish_connection();

        return crate::schema::documents::dsl::documents
            .find(document_id)
            .first(connection);
    }
}


/*
    below are for web services
*/


pub enum DocumentStatus {
    Received, 
    Processing,
    Ready, 
    Failed
}

impl fmt::Display for DocumentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            DocumentStatus::Received => write!(f, "received"), 
            DocumentStatus::Processing => write!(f, "processing"), 
            DocumentStatus::Failed => write!(f, "failed"), 
            DocumentStatus::Ready => write!(f, "ready")
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String, 
    pub full_text: String,  // need to verify if the text can be all converted to MD
    #[serde(default = "default_maximum_text_split_size")]
    pub maximum_text_split_size: u64
}

fn default_maximum_text_split_size() -> u64 {
    return 256;
}


#[derive(Serialize, Deserialize)]
pub struct CreateDocumentResponse {
    pub document_id: String, 
    pub status: String
}

impl CreateDocumentResponse {
    pub fn new(document_id: String, status: DocumentStatus) -> Self {
        return Self {
            document_id: document_id, 
            status: status.to_string()
        }; 
    }
}


#[derive(Serialize, Deserialize)]
pub struct SearchDocumentRequest {
    pub document_id: String,
    pub input_text: String,
    pub top_n: u64
}


#[derive(Serialize, Deserialize)]
pub struct SearchDocumentResponse {
    pub query_results: Vec<QueryResult>
}


#[derive(Serialize, Deserialize)]
pub struct DocumentDeleteRequest {
    pub document_id: String
}


/// NOTICE: updating the document does not have a dedicated response model, 
/// instead, we reuse the CreateDocumentResponse. 
#[derive(Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    pub updated_parameters: CreateDocumentRequest,
    pub document_id: String
}


#[derive(Serialize, Deserialize)]
pub struct GetAllDocumentsResponse {
    pub id: String, 
    pub title: String
}


#[derive(Serialize, Deserialize)]
pub struct GetDocumentByIDResponse {
    pub id: String, 
    pub title: String, 
    pub full_text: String
}