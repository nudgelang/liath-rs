use anyhow::Result;
use candle_core::{Device, Tensor};
use candle_transformers::models::quantized_llama::ModelWeights;
use candle_transformers::generation::{LogitsProcessor, Sampling};
use tokenizers::Tokenizer;
use std::path::PathBuf;
use std::sync::Arc;

pub struct LLMWrapper {
    model: Arc<ModelWeights>,
    tokenizer: Tokenizer,
    device: Device,
}

impl LLMWrapper {
    pub fn new(model_path: PathBuf, tokenizer_path: PathBuf, device: Device) -> Result<Self> {
        let mut file = std::fs::File::open(&model_path)?;
        let model = ModelWeights::from_gguf_file(&mut file, &device)?;
        let tokenizer = Tokenizer::from_file(tokenizer_path)?;

        Ok(Self {
            model: Arc::new(model),
            tokenizer,
            device,
        })
    }

    pub async fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        let tokens = self.tokenizer.encode(prompt, true)?;
        let input = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;

        let mut logits_processor = LogitsProcessor::new(299792458, 0.8, None, None); // You can adjust these parameters

        let mut generated_tokens = Vec::new();
        let mut next_token = 0;

        for index in 0..max_tokens {
            let input = if index == 0 {
                input.clone()
            } else {
                Tensor::new(&[next_token], &self.device)?.unsqueeze(0)?
            };

            let logits = self.model.forward(&input, index)?;
            let logits = logits.squeeze(0)?;
            next_token = logits_processor.sample(&logits)?;
            generated_tokens.push(next_token);

            if next_token == self.tokenizer.token_to_id("</s>").unwrap_or(0) {
                break;
            }
        }

        let output = self.tokenizer.decode(&generated_tokens, true)?;
        Ok(output)
    }
}