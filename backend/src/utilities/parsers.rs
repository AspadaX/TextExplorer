use text_splitter::TextSplitter;
use indicatif::ProgressBar;

use crate::vectorization::text::TextVectorization;

/// the parser here is to provide a package that processes inputs
/// to be ready for storing into the vector database
pub struct BasicParser {
    pub vectors: Vec<Vec<f32>>,
    pub slice_ids: Vec<String>, 
    pub text_slices: Vec<String>
}

pub trait Parser {
    /// the new method returns a usable struct for storing items into 
    /// the vector database
    fn new(
        maximum_text_split_size: usize, 
        full_text: String, 
        embedding_model: &TextVectorization
    ) -> Self;
}


impl Parser for BasicParser {
    fn new(
        maximum_text_split_size: usize, 
        full_text: String, 
        embedding_model: &TextVectorization
    ) -> Self {
        // parse the document
        let text_splitter = TextSplitter::new(
            maximum_text_split_size as usize
        );
        let chunks: Vec<String> = text_splitter.chunks(
            &full_text
        )
            .into_iter()
            .map(|chunk| chunk.to_string())
            .collect();

        // create variables for storing the document in the vector database
        let mut vectors: Vec<Vec<f32>> = Vec::new();
        let mut slice_ids: Vec<String> = Vec::new();
        let mut text_slices: Vec<String> = Vec::new();

        // set a progress bar
        let progress_bar = ProgressBar::new(
            chunks.len() as u64
        );
        // vectorize
        for chunk in chunks {
            progress_bar.inc(1);
            vectors.push(
                embedding_model.vectorize(&chunk, true).unwrap()
            );
            slice_ids.push(
                uuid::Uuid::new_v4().to_string()
            );
            text_slices.push(
                chunk.to_string()
            );
        }
        progress_bar.finish();

        return BasicParser {
            vectors: vectors, 
            slice_ids: slice_ids, 
            text_slices: text_slices
        }
    }
}