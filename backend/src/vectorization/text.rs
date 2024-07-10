use core::fmt;

use ort::{GraphOptimizationLevel, Session};
use tokenizers::Tokenizer;


// a structure that holds the necessary components for vectorization
pub struct TextVectorization {
    pub tokenizer: Tokenizer, 
    pub model: Session
}

#[derive(Debug)]
pub enum TextVectorizationErrors {
    ExtractedOutputsError
}

impl fmt::Display for TextVectorizationErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TextVectorizationErrors::ExtractedOutputsError => write!(
                f, 
                "Error happens when extracting the outputs"
            )
        }
    }
}

impl std::error::Error for TextVectorizationErrors {}


impl TextVectorization {
    pub fn new(
        model_path: &str, 
        tokenizer_path: &str, 
        optimization_level: GraphOptimizationLevel
    ) -> Result<Self, Box<dyn std::error::Error>> {

        let model: Session = Session::builder()?
            .with_optimization_level(optimization_level)?
            .commit_from_file(model_path)?;

        let tokenizer: Tokenizer = Tokenizer::from_file(tokenizer_path)
            .unwrap_or_else(
                |error| panic!(
                    "Error when loading the tokenizer from path: {}, and the error says: {}", 
                    tokenizer_path, 
                    error
                )
            );

        return Ok(
            TextVectorization {
                tokenizer: tokenizer, 
                model: model
            }
        );
    }

    pub fn vectorize(
        &self, 
        input_text: &str,
        enable_special_tokens: bool
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {

        let encoding: tokenizers::Encoding = match self.tokenizer.encode(
            input_text, 
            enable_special_tokens
        ) {
            Ok(encoded_input_text) => encoded_input_text, 
            Err(error) => return Err(error)
        };

        // the model needs ids and the attention mask to inference vectors
        let ids: &[u32] = encoding.get_ids();
        let attention_mask: &[u32] = encoding.get_attention_mask();

        
        // transform the ids and attention mask to the required shape
        let transformed_ids = std::sync::Arc::new(
            attention_mask
                .iter()
                .map(|i| *i as i64)
                .collect::<Vec<i64>>()
                .into_boxed_slice()
        );
        let transformed_attention_mask = std::sync::Arc::new(
            ids
                .iter()
                .map(|i| *i as i64)
                .collect::<Vec<i64>>()
                .into_boxed_slice()
        );

        let input_ids = (
            vec![1, transformed_ids.len() as i64], 
            std::sync::Arc::clone(&transformed_ids)
        );
        let input_attention_mask = (
            vec![1, transformed_attention_mask.len() as i64], 
            std::sync::Arc::clone(&transformed_attention_mask)
        );

        println!(
            "{:?}",
            input_ids
        );
        println!(
            "{:?}",
            input_attention_mask
        );

        // inference
        let outputs = self.model.run(
            ort::inputs![input_ids, input_attention_mask]?
        )?;

        println!(
            "{:?}", 
            outputs
        );

        // extract the final result
        let extracted_outputs = outputs["dense_vecs"]
            .try_extract_tensor::<f32>()?;

        if let Some(vector) = extracted_outputs
            .rows()
            .into_iter()
            .next() {
                return Ok(
                    vector
                        .iter()
                        .map(|&x| x as f32)
                        .collect()
                );
            } else {
                return Err(
                    Box::new(TextVectorizationErrors::ExtractedOutputsError)
                );
            }
    }
}