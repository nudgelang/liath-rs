use llama_cpp_rs::{LLama, ModelOptions, PredictOptions};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

pub struct LLMWrapper {
    model: Arc<Mutex<LLama>>,
}

impl LLMWrapper {
    pub fn new(model_path: &str) -> Result<Self> {
        let model_options = ModelOptions::default();
        let llama = LLama::new(model_path.into(), &model_options)?;
        Ok(Self {
            model: Arc::new(Mutex::new(llama)),
        })
    }

    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let model = self.model.lock().await;
        let predict_options = PredictOptions::default();
        let result = model.predict(prompt.to_string(), predict_options)?;
        Ok(result)
    }
}