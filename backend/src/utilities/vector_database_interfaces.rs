/*
    the reason for creating an additional layer when interfacing with the
    vector database is that the vector databases are designed with `collections`, 
    which is equivalent to conventional databases' `table`. 
    
    Hence, creating `collections` for each new document will consume way too 
    many resources. To mitigate that, we put all documents in one collection, 
    therefore greatly reduce the resource consumptions. 
*/
use log::{error, debug};
use serde::{Deserialize, Serialize};
use serde_json::json;
use qdrant_client::{
    self, qdrant::{
        value::Kind as QdrantKind, 
        vectors_config::Config, 
        Condition, 
        CreateCollection, 
        DeletePointsBuilder, 
        Distance, 
        Filter, 
        PointStruct,
        ScoredPoint,
        ScrollPoints,
        SearchParams,
        SearchPoints,
        UpsertPointsBuilder,
        VectorParams,
        VectorsConfig
    },
    Payload, 
    Qdrant
};

#[derive(Debug)]
pub enum QueryErrors {
    MissingContent,
    MissingScore, 
    InvalidScore
}

// the struct for holding all kinds of query result from the 
// vector database backends
#[derive(Serialize, Deserialize)]
pub struct QueryResult {
    pub relevance: f32, 
    pub content: Option<String>
}

impl QueryResult {
    async fn from_qdrant_scored_point(
        value: ScoredPoint
    ) -> Result<Self, QueryErrors> {

        if value.score <= 0.0 {
            return Err(QueryErrors::InvalidScore);
        }

        let content = value.payload
            .get("content")
            .ok_or(QueryErrors::MissingContent)
            .and_then(
            |value| 
                match &value.kind {
                    Some(QdrantKind::StringValue(content)) => Ok(
                        Some(
                            content.clone()
                        )
                    ),
                    _ => Err(QueryErrors::MissingContent)
                }
        )?;

        return Ok(
            QueryResult {
                relevance: value.score, 
                content: content
            }
        )
    }
}

pub enum VectorDatabaseInterfaceErrors {
    ConnectionError, 
    CollectionCreationError, 
    TextSlicesMismatchError, 
    SearchError,
    ScrollError,
    DeletionError
}

impl std::fmt::Display for VectorDatabaseInterfaceErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            VectorDatabaseInterfaceErrors::CollectionCreationError => write!(
                f, "Error happens when creating a new collection."
            ),
            VectorDatabaseInterfaceErrors::ConnectionError => write!(
                f, "Error happens when trying to connect to the vector database."
            ),
            VectorDatabaseInterfaceErrors::DeletionError => write!(
                f, "Error happens when deleting a slice."
            ),
            VectorDatabaseInterfaceErrors::SearchError => write!(
                f, "Vector search encounters errors. Connection might be lost."
            ),
            VectorDatabaseInterfaceErrors::TextSlicesMismatchError => write!(
                f, "The number of text slices MUST be the same as the number of vectors!"
            ), 
            VectorDatabaseInterfaceErrors::ScrollError => write!(
                f, "Error happens when trying to scroll the collection"
            )
        }
    }
}
// this trait is required for all vector database backend
pub trait AsynchronousVectorDatabaseInterface {

    // a method to initiate a new instance of the vector database
    // each new method requires the following operations: 
    // 1. instantiate a client
    // 2. create a collection for storing documents
    // 3. return the struct
    async fn new(
        connection_url: &String, 
        documents_collection: &String, 
        vector_dimensionality: &u64
    ) -> Result<Self, VectorDatabaseInterfaceErrors> where Self: Sized;

    // a method to create a new document in the vector database
    async fn create_document(
        &self, 
        document_id: &String, 
        text_slices: &Vec<String>, 
        chunk_factor: f32, 
        vectors: &Vec<Vec<f32>>, 
        slice_ids: &Vec<String>
    ) -> Option<VectorDatabaseInterfaceErrors>;

    // a method to vector search a document
    async fn search_document(
        &self, 
        document_id: &String, 
        input_vector: &Vec<f32>, 
        top_n: &u64
    ) -> Result<Vec<QueryResult>, VectorDatabaseInterfaceErrors>;

    // to avoid complexities, the update method is simply to 
    // delete the specified document, then recreate a new one
    async fn update_document(
        &self, 
        document_id: &String, 
        text_slices: &Vec<String>, 
        chunk_factor: Option<f32>,
        vectors: &Vec<Vec<f32>>, 
        slice_ids: &Vec<String>
    ) -> Option<VectorDatabaseInterfaceErrors>;

    // a method to delete a document entirely from the vector database
    async fn delete_document(
        &self, 
        document_id: &String
    ) -> Option<VectorDatabaseInterfaceErrors>;
}


pub struct AsynchronousQdrantVectorDatabaseBackend {
    pub client: Qdrant,
    pub documents_collection_name: String
}


#[derive(Serialize, Deserialize, Debug)]
pub struct DocumentSlice {  
    document_id: String, 
    content: String, 
    chunk_factor: f32
}


impl AsynchronousVectorDatabaseInterface for AsynchronousQdrantVectorDatabaseBackend {

    /// instantiate a new vector database backend object for Qdrant. 
    /// it takes a url of the vector database, a collection name, and 
    /// a vector dimensionality number. 
    /// the collection is the same as a table in a traditional SQL database, 
    /// therefore, for saving resources, we will not create a collection for 
    /// each Document. 
    async fn new(
        connection_url: &String, 
        documents_collection: &String, 
        vector_dimensionality: &u64
    ) -> Result<Self, VectorDatabaseInterfaceErrors> {
        
        // initialize a qdrant client
        // it is worth noting that the Rust version of Qdrant client uses gRPC
        // connection only as of July, 2024. 
        let client = qdrant_client::Qdrant::from_url(
            connection_url
        )
            .build()
            .map_err(|_| VectorDatabaseInterfaceErrors::ConnectionError)?;

        // check if the collection exists first. 
        // try to avoid the situation in which Qdrant panics and abort the 
        // entire operation. This is a subtle difference between the Python
        // client and the Rust one. 
        let collection_exists = client.collection_exists(
            documents_collection
        ).await.unwrap();

        // create a collection for storing documents
        // it checks if the collection has already existed, otherwise
        // it creates a new one. 
        if collection_exists {
            return Ok(
                AsynchronousQdrantVectorDatabaseBackend {
                    client: client, 
                    documents_collection_name: documents_collection.to_string()
                }
            ) 
        } else {
            match client.create_collection(
                CreateCollection {
                    collection_name: documents_collection.to_string(),
                    vectors_config: Some(
                        VectorsConfig {
                            config: Some(
                                Config::Params(
                                    VectorParams {
                                        size: *vector_dimensionality,
                                        distance: Distance::Cosine.into(),
                                        ..Default::default()
                                    }
                                )
                            ),
                        }
                    ),
                    ..Default::default()
                }
            ).await {
                Ok(_) => return Ok(
                    AsynchronousQdrantVectorDatabaseBackend {
                        client: client, 
                        documents_collection_name: documents_collection.to_string()
                    }
                ), 
                Err(error) => {
                    error!("{:?}", error);
                    return Err(
                        VectorDatabaseInterfaceErrors::CollectionCreationError
                    ); // as a side note: we use a unified Error for the vector
                    // database interfaces, to unify the experiences of various
                    // backend. 
                }
            };
        }
        
    }

    /// create a new Document entry in the vector database
    /// the `document_id` should be aligned with the `id` in the SQL database
    /// , otherwise there will be a mismatch between the ids on both end. i
    /// 
    /// the `vectors` and `slice_ids` MUST be in the same length, as the former
    /// is the vector for each slice of text, and the latter is sliced texts. 
    async fn create_document(
        &self, 
        document_id: &String, 
        text_slices: &Vec<String>, 
        chunk_factor: f32, 
        vectors: &Vec<Vec<f32>>, 
        slice_ids: &Vec<String>
    ) -> Option<VectorDatabaseInterfaceErrors> {

        // it will be used later for computing the chunk size
        // the so called `chunk_size` is another name of `batch_size` given by
        // Qdrant. 
        // since we don't know how big a Document will be, we employ a chunk 
        // factor for calculating the batch size during runtime.
        // the chunk factor is specified in `configuration.json`.  
        let document_size: usize;

        // validate if the `text_slice` and `vectors` are in the same length, 
        // otherwise, it would a logic error if the number of text slices and 
        // the number of corresponding vectors are not identical. 
        if text_slices.len() != vectors.len() {
            return Some(
                VectorDatabaseInterfaceErrors::TextSlicesMismatchError
            );
        } else {
            document_size = text_slices.len();
        }

        // iterate through the `text_slices` to create points for insertion. 
        let mut points: Vec<PointStruct> = Vec::new();

        for (index, text_slice) in text_slices.iter().enumerate() {
            points.push(
                PointStruct::new(
                    slice_ids[index].clone(), 
                    vectors[index].clone(), 
                    Payload::try_from(
                        json!( // the additional information we want to store. 
                            DocumentSlice {
                                document_id: document_id.clone(), 
                                content: text_slice.clone(),
                                chunk_factor: chunk_factor.clone()
                            }
                        )
                    ).unwrap()
                )
            );
        }

        // the builder is required by Qdrant, before insertion. 
        let request = UpsertPointsBuilder::new(
            self.documents_collection_name.clone(), 
            points
        );

        // a simple equation to calculate the proper batch size when sending data
        // to the vector database. 
        let chunk_size: usize = ((document_size as f32 / chunk_factor).ceil() as usize).max(1);
        
        // insert the points with the chunk method provided by Qdrant
        match self.client
            .upsert_points_chunked(request, chunk_size)
            .await {
                Ok(_) => {
                    return None;
                }, 
                Err(_) => return Some(
                    VectorDatabaseInterfaceErrors::CollectionCreationError
                )
            };
    }

    async fn delete_document(
        &self, 
        document_id: &String
    ) -> Option<VectorDatabaseInterfaceErrors> {
        
        let request = DeletePointsBuilder::new(
            self.documents_collection_name.clone()
        )
            .points(
                Filter::must(
                    [
                        Condition::matches(
                            "document_id", 
                            document_id.clone()
                        )
                    ]
                )
            ).wait(true);

        match self.client
            .delete_points(request)
            .await {
                Ok(_) => None, 
                Err(error) => {
                    error!(
                        "{}", error.to_string()
                    );
                    return Some(
                        VectorDatabaseInterfaceErrors::DeletionError
                    );
                }
            }
        
    }

    async fn search_document(
        &self, 
        document_id: &String, 
        input_vector: &Vec<f32>, 
        top_n: &u64
    ) -> Result<Vec<QueryResult>, VectorDatabaseInterfaceErrors> {

        let response = match self.client
            .search_points(
                SearchPoints {
                    collection_name: self.documents_collection_name.to_string(),
                    filter: Some(
                        Filter::must(
                            [
                                Condition::matches(
                                    "document_id",
                                    document_id.to_string(),
                                )
                            ]
                        )
                    ),
                    params: Some(
                        SearchParams {
                            hnsw_ef: Some(128),
                            exact: Some(false),
                            ..Default::default()
                        }
                    ),
                    vector: input_vector.clone(),
                    limit: *top_n, 
                    with_payload: Some(true.into()),
                    ..Default::default()
                }
            )
                .await {
                    Ok(result) => result,
                    Err(error) => {
                        error!("{:?}", error);
                        return Err(
                            VectorDatabaseInterfaceErrors::SearchError
                        );
                    }
                };
        
        let mut results: Vec<QueryResult> = Vec::new();
        
        for item in response.result {
            debug!("{:?}", item);
            results.push(
                match QueryResult::from_qdrant_scored_point(item)
                    .await {
                        Ok(result) => result, 
                        Err(error) => {
                            error!("{:?}", error);
                            return Err(
                                VectorDatabaseInterfaceErrors::SearchError
                            );
                        }
                    }
            )
        }
        
        return Ok(results);

    }

    async fn update_document(
        &self, 
        document_id: &String, 
        text_slices: &Vec<String>, 
        chunk_factor: Option<f32>,
        vectors: &Vec<Vec<f32>>, 
        slice_ids: &Vec<String>
    ) -> Option<VectorDatabaseInterfaceErrors> {
        
        // here we check if the `chunk_factor` is supplied, otherwise, we use
        // the default value in the document stored in the vector database. 
        // the `chunk_factor_checked` is for storing the final value. 
        let chunk_factor_checked: f32;

        if let None = chunk_factor {
            // we first need to retrieve the 
            let result = match self.client.scroll(
                ScrollPoints {
                    collection_name: self.documents_collection_name.to_string(),
                    filter: Some(
                        Filter::must(
                            [
                                Condition::matches(
                                    "document_id", 
                                    document_id.clone()
                                )
                            ]
                        )
                    ),
                    // limit: Some(u32::MAX), /// a convenient way to get all points
                    limit: Some(1),
                    with_payload: Some(true.into()), 
                    with_vectors: Some(false.into()), 
                    ..Default::default()
                }
            )
                .await {
                    Ok(result) => result.result, 
                    Err(_) => {
                        error!(
                            "Checking number of slices of document {} failed.", 
                            *document_id
                        );

                        return Some(
                            VectorDatabaseInterfaceErrors::ScrollError
                        );
                    }
                };
            
            // retrieve the `chunk_factor` assigned when the document
            // was created. 
            chunk_factor_checked = result[0]
                .payload["chunk_factor"]
                .as_double()
                .unwrap() as f32;
        } else {
            chunk_factor_checked = chunk_factor.unwrap();
        }

        // implementing the logics specified in the trait... 
        // we first delete the document from the vector database. 
        match self.delete_document(document_id).await {
            None => {},
            Some(error) => {
                error!("{}", error);
                return Some(VectorDatabaseInterfaceErrors::DeletionError);
            }
        };

        self.create_document(
            document_id, 
            text_slices, 
            chunk_factor_checked, 
            vectors, 
            slice_ids
        ).await;
        
        return None;
    }
}