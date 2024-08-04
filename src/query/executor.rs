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
    lua_vm: Arc<LuaVM>,
    file_storage: FileStorage,
    auth_manager: AuthManager,
    llm_semaphore: Arc<Semaphore>,
    embedding_semaphore: Arc<Semaphore>,
}

impl QueryExecutor {
    // ... (previous methods remain the same)

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
            QueryType::Join => self.execute_join(&args).await,
            QueryType::Aggregate => self.execute_aggregate(&args).await,
            QueryType::InstallPackage => self.execute_install_package(&args).await,
            QueryType::ListPackages => self.execute_list_packages().await,
            QueryType::ExecuteLua => self.execute_lua(&args).await,
        }
    }

    #[instrument(skip(self, args))]
    async fn execute_join(&self, args: &[String]) -> Result<String> {
        if args.len() < 4 {
            return Err(anyhow::anyhow!("Invalid JOIN query. Usage: JOIN <namespace1> <namespace2> <key1> <key2>"));
        }
        let namespace1 = self.namespace_manager.get_namespace(&args[0])
            .context("Namespace1 not found")?;
        let namespace2 = self.namespace_manager.get_namespace(&args[1])
            .context("Namespace2 not found")?;
        
        // Implement join logic here
        // This is a simplified example and might need to be adapted based on your specific use case
        let key1 = args[2].as_bytes();
        let key2 = args[3].as_bytes();
        
        let value1 = namespace1.db.get(key1)
            .context("Failed to retrieve value from namespace1")?;
        let value2 = namespace2.db.get(key2)
            .context("Failed to retrieve value from namespace2")?;
        
        Ok(format!("Joined result: {:?} - {:?}", value1, value2))
    }

    #[instrument(skip(self, args))]
    async fn execute_aggregate(&self, args: &[String]) -> Result<String> {
        if args.len() < 3 {
            return Err(anyhow::anyhow!("Invalid AGGREGATE query. Usage: AGGREGATE <namespace> <operation> <key_prefix>"));
        }
        let namespace = self.namespace_manager.get_namespace(&args[0])
            .context("Namespace not found")?;
        let operation = &args[1];
        let key_prefix = args[2].as_bytes();
        
        // Implement aggregation logic here
        // This is a simplified example and might need to be adapted based on your specific use case
        let mut values = Vec::new();
        namespace.db.prefix_iterator(key_prefix).for_each(|(_, value)| {
            if let Ok(v) = String::from_utf8(value.to_vec()) {
                if let Ok(num) = v.parse::<f64>() {
                    values.push(num);
                }
            }
        });
        
        let result = match operation.as_str() {
            "sum" => values.iter().sum::<f64>(),
            "avg" => values.iter().sum::<f64>() / values.len() as f64,
            "max" => values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            "min" => values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            _ => return Err(anyhow::anyhow!("Unsupported aggregation operation")),
        };
        
        Ok(format!("Aggregation result: {}", result))
    }

    #[instrument(skip(self, args))]
    async fn execute_install_package(&self, args: &[String]) -> Result<String> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("Package name not provided"));
        }
        self.lua_vm.install_package(&args[0])?;
        Ok(format!("Package '{}' installed successfully", args[0]))
    }

    #[instrument(skip(self))]
    async fn execute_list_packages(&self) -> Result<String> {
        let packages = self.lua_vm.list_installed_packages()?;
        Ok(format!("Installed packages: {:?}", packages))
    }

    #[instrument(skip(self, args))]
    async fn execute_lua(&self, args: &[String]) -> Result<String> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("Lua code not provided"));
        }
        let code = args.join(" ");
        let result = self.lua_vm.execute(&code)?;
        Ok(format!("Lua execution result: {}", result))
    }
}