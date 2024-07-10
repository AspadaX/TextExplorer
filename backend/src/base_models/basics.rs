use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub struct BasicResponse<T> {
    pub status: bool, 
    pub message: String, 
    pub data: Option<T>
}