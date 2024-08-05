use anyhow::{Result, Context};
use candle_core::{Device, Tensor};
use candle_transformers::models::quantized_llama::ModelWeights;
use candle_transformers::generation::{LogitsProcessor, Sampling};
use tokenizers::Tokenizer;
use std::path::PathBuf;
use std::sync::Arc;
use std::cell::RefCell;
use candle_core::quantized::{ggml_file, gguf_file};

pub struct LLMWrapper {
    model: Arc<RefCell<ModelWeights>>,
    tokenizer: Tokenizer,
    device: Device,
}

impl LLMWrapper {
    pub fn new(model_path: PathBuf, tokenizer_path: PathBuf, device: Device) -> Result<Self> {
        let mut file = std::fs::File::open(&model_path)?;
        let start = std::time::Instant::now();

        let model = match model_path.extension().and_then(|v| v.to_str()) {
            Some("gguf") => {
                let model = gguf_file::Content::read(&mut file).map_err(|e| e.with_path(&model_path))?;
                let mut total_size_in_bytes = 0;
                for (_, tensor) in model.tensor_infos.iter() {
                    let elem_count = tensor.shape.elem_count();
                    total_size_in_bytes +=
                        elem_count * tensor.ggml_dtype.type_size() / tensor.ggml_dtype.block_size();
                }
                println!(
                    "loaded {:?} tensors ({}) in {:.2}s",
                    model.tensor_infos.len(),
                    format_size(total_size_in_bytes),
                    start.elapsed().as_secs_f32(),
                );
                ModelWeights::from_gguf(model, &mut file, &device)?
            }
            Some("ggml" | "bin") | Some(_) | None => {
                let model = ggml_file::Content::read(&mut file, &device)
                    .map_err(|e| e.with_path(&model_path))?;
                let mut total_size_in_bytes = 0;
                for (_, tensor) in model.tensors.iter() {
                    let elem_count = tensor.shape().elem_count();
                    total_size_in_bytes +=
                        elem_count * tensor.dtype().type_size() / tensor.dtype().block_size();
                }
                println!(
                    "loaded {:?} tensors ({}) in {:.2}s",
                    model.tensors.len(),
                    format_size(total_size_in_bytes),
                    start.elapsed().as_secs_f32(),
                );
                println!("params: {:?}", model.hparams);
                ModelWeights::from_ggml(model, 1)?
            }
        };

        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        Ok(Self {
            model: Arc::new(RefCell::new(model)),
            tokenizer,
            device,
        })
    }

    pub fn generate(&self, prompt: &str, sample_len: usize, temp: f64, repeat_penalty: f32, repeat_last_n: usize) -> Result<String> {
        let tokens = self.tokenizer.encode(prompt, true).map_err(|e| anyhow::anyhow!("Tokenizer encode error: {}", e))?;
        let prompt_tokens = tokens.get_ids().to_vec();
        let mut all_tokens = vec![];
        
        let mut logits_processor = {
            let sampling = if temp <= 0. {
                Sampling::ArgMax
            } else {
                Sampling::All { temperature: temp }
            };
            LogitsProcessor::new(299792458, None, None)
        };

        let mut next_token = {
            let input = Tensor::new(prompt_tokens.as_slice(), &self.device)?.unsqueeze(0)?;
            let logits = self.model.borrow_mut().forward(&input, 0)?;
            let logits = logits.squeeze(0)?;
            logits_processor.sample(&logits)?
        };

        all_tokens.push(next_token);

        let eos_token = self.tokenizer.token_to_id("</s>").ok_or_else(|| anyhow::anyhow!("EOS token not found"))?;

        for index in 0..sample_len {
            let input = Tensor::new(&[next_token], &self.device)?.unsqueeze(0)?;
            let logits = self.model.borrow_mut().forward(&input, prompt_tokens.len() + index)?;
            let logits = logits.squeeze(0)?;
            let logits = if repeat_penalty == 1. {
                logits
            } else {
                let start_at = all_tokens.len().saturating_sub(repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    repeat_penalty,
                    &all_tokens[start_at..],
                )?
            };
            next_token = logits_processor.sample(&logits)?;
            all_tokens.push(next_token);
            if next_token == eos_token {
                break;
            }
        }

        let output = self.tokenizer.decode(&all_tokens, true).map_err(|e| anyhow::anyhow!("Decode error: {}", e))?;

        Ok(output)
    }
}

fn format_size(size_in_bytes: usize) -> String {
    if size_in_bytes < 1_000 {
        format!("{}B", size_in_bytes)
    } else if size_in_bytes < 1_000_000 {
        format!("{:.2}KB", size_in_bytes as f64 / 1e3)
    } else if size_in_bytes < 1_000_000_000 {
        format!("{:.2}MB", size_in_bytes as f64 / 1e6)
    } else {
        format!("{:.2}GB", size_in_bytes as f64 / 1e9)
    }
}