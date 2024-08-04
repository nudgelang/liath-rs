use crate::core::NamespaceManager;
use crate::ai::{LLMWrapper, EmbeddingWrapper};
use crate::lua::LuaVM;
use crate::file::{FileStorage, FileProcessor};
use crate::query::parser::{QueryParser, QueryType};
use crate::auth::AuthManager;
use anyhow::{Result, Context};
use tokio::sync::Semaphore;
use std::sync::Arc;
use tracing::{info, error, instrument};

pub struct QueryExecutor {
    namespace_manager: NamespaceManager,
    llm: Arc<LLMWrapper>,
    embedding: Arc<EmbeddingWrapper>,
    lua_vm: LuaVM,
    file_storage: FileStorage,
    auth_manager: AuthManager,
    llm_semaphore: Arc<Semaphore>,
    embedding_semaphore: Arc<Semaphore>,
}

impl QueryExecutor {
    pub fn new(
        namespace_manager: NamespaceManager,
        llm: LLMWrapper,
        embedding: EmbeddingWrapper,
        lua_vm: LuaVM,
        file_storage: FileStorage,
        auth_manager: AuthManager,
        max_concurrent_llm: usize,
        max_concurrent_embedding: usize,
    ) -> Self {
        Self {
            namespace_manager,
            llm: Arc::new(llm),
            embedding: Arc::new(embedding),
            lua_vm,
            file_storage,
            auth_manager,
            llm_semaphore: Arc::new(Semaphore::new(max_concurrent_llm)),
            embedding_semaphore: Arc::new(Semaphore::new(max_concurrent_embedding)),
        }
    }

    #[instrument(skip(self, query))]
    pub async fn execute(&self, query: &str, user_id: &str) -> Result<String> {
        let (query_type, args) = QueryParser::parse(query).context("Failed to parse query")?;

        if !self.auth_manager.is_authorized(user_id, &query_type) {
            return Err(anyhow::anyhow!("User not authorized for this operation"));
        }

        match query_type {
            QueryType::Select => self.execute_select(&args).await,
            QueryType::Insert => self.execute_insert(&args).await,
            QueryType::Update => self.execute_update(&args).await,
            QueryType::Delete => self.execute_delete(&args).await,
            QueryType::CreateNamespace => self.execute_create_namespace(&args).await,
            QueryType::DeleteNamespace => self.execute_delete_namespace(&args).await,
            QueryType::UploadFile => self.execute_upload_file(&args).await,
            QueryType::ProcessFile => self.execute_process_file(&args).await,
            QueryType::GenerateEmbedding => self.execute_generate_embedding(&args).await,
            QueryType::SimilaritySearch => self.execute_similarity_search(&args).await,
            QueryType::LLMQuery => self.execute_llm_query(&args).await,
        }
    }

    #[instrument(skip(self, args))]
    async fn execute_select(&self, args: &[String]) -> Result<String> {
        if args.len() < 2 {
            return Err(anyhow::anyhow!("Invalid SELECT query. Usage: SELECT <namespace> <key>"));
        }
        let namespace = self.namespace_manager.get_namespace(&args[0])
            .context("Namespace not found")?;
        let value = namespace.db.get(args[1].as_bytes())
            .context("Failed to retrieve value")?;
        
        Ok(format!("Value: {:?}", value.map(String::from_utf8_lossy)))
    }

    #[instrument(skip(self, args))]
    async fn execute_insert(&self, args: &[String]) -> Result<String> {
        if args.len() < 3 {
            return Err(anyhow::anyhow!("Invalid INSERT query. Usage: INSERT <namespace> <key> <value>"));
        }
        let namespace = self.namespace_manager.get_namespace(&args[0])
            .context("Namespace not found")?;
        namespace.db.put(args[1].as_bytes(), args[2].as_bytes())
            .context("Failed to insert value")?;
        
        Ok("Value inserted successfully".to_string())
    }

    #[instrument(skip(self, args))]
    async fn execute_update(&self, args: &[String]) -> Result<String> {
        if args.len() < 3 {
            return Err(anyhow::anyhow!("Invalid UPDATE query. Usage: UPDATE <namespace> <key> <value>"));
        }
        let namespace = self.namespace_manager.get_namespace(&args[0])
            .context("Namespace not found")?;
        namespace.db.put(args[1].as_bytes(), args[2].as_bytes())
            .context("Failed to update value")?;
        
        Ok("Value updated successfully".to_string())
    }

    #[instrument(skip(self, args))]
    async fn execute_delete(&self, args: &[String]) -> Result<String> {
        if args.len() < 2 {
            return Err(anyhow::anyhow!("Invalid DELETE query. Usage: DELETE <namespace> <key>"));
        }
        let namespace = self.namespace_manager.get_namespace(&args[0])
            .context("Namespace not found")?;
        namespace.db.delete(args[1].as_bytes())
            .context("Failed to delete value")?;
        
        Ok("Value deleted successfully".to_string())
    }

    #[instrument(skip(self, args))]
    async fn execute_create_namespace(&self, args: &[String]) -> Result<String> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("Namespace name not provided"));
        }
        self.namespace_manager.create_namespace(&args[0])
            .context("Failed to create namespace")?;
        Ok(format!("Namespace '{}' created successfully", args[0]))
    }

    #[instrument(skip(self, args))]
    async fn execute_delete_namespace(&self, args: &[String]) -> Result<String> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("Namespace name not provided"));
        }
        self.namespace_manager.delete_namespace(&args[0])
            .context("Failed to delete namespace")?;
        Ok(format!("Namespace '{}' deleted successfully", args[0]))
    }

    #[instrument(skip(self, args))]
    async fn execute_upload_file(&self, args: &[String]) -> Result<String> {
        if args.len() < 2 {
            return Err(anyhow::anyhow!("File path and content not provided"));
        }
        let file_id = self.file_storage.store(args[1].as_bytes())
            .context("Failed to store file")?;
        Ok(format!("File uploaded successfully. File ID: {}", file_id))
    }

    #[instrument(skip(self, args))]
    async fn execute_process_file(&self, args: &[String]) -> Result<String> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("File ID not provided"));
        }
        let file_content = self.file_storage.retrieve(&args[0])
            .context("Failed to retrieve file")?;
        let extracted_text = FileProcessor::extract_text(file_content.as_slice())
            .context("Failed to extract text from file")?;
        Ok(format!("Extracted text: {}", extracted_text))
    }

    #[instrument(skip(self, args))]
    async fn execute_generate_embedding(&self, args: &[String]) -> Result<String> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("Text for embedding not provided"));
        }
        let permit = self.embedding_semaphore.acquire().await
            .context("Failed to acquire embedding semaphore")?;
        let embedding = self.embedding.generate(&args[0])
            .context("Failed to generate embedding")?;
        drop(permit);
        Ok(format!("Embedding generated: {:?}", embedding))
    }

    #[instrument(skip(self, args))]
    async fn execute_similarity_search(&self, args: &[String]) -> Result<String> {
        if args.len() < 3 {
            return Err(anyhow::anyhow!("Invalid similarity search query. Usage: SIMILARITY_SEARCH <namespace> <vector> <k>"));
        }
        let namespace = self.namespace_manager.get_namespace(&args[0])
            .context("Namespace not found")?;
        let vector: Vec<f32> = serde_json::from_str(&args[1])
            .context("Failed to parse vector")?;
        let k: usize = args[2].parse()
            .context("Failed to parse k")?;
        
        let results = namespace.vector_db.search(&vector, k)
            .context("Failed to perform similarity search")?;
        
        Ok(format!("Similarity search results: {:?}", results))
    }

    #[instrument(skip(self, args))]
    async fn execute_llm_query(&self, args: &[String]) -> Result<String> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("Query for LLM not provided"));
        }
        let permit = self.llm_semaphore.acquire().await
            .context("Failed to acquire LLM semaphore")?;
        let response = self.llm.generate(&args[0]).await
            .context("Failed to generate LLM response")?;
        drop(permit);
        Ok(format!("LLM response: {}", response))
    }
}