use crate::core::NamespaceManager;
use crate::ai::{LLMWrapper, EmbeddingWrapper};
use crate::lua::LuaVM;
use crate::file::FileStorage;
use crate::auth::AuthManager;
use anyhow::{Result, Context};
use tokio::sync::Semaphore;
use std::sync::Arc;
use tracing::{info, error, instrument};
use rlua::{Lua, Context as LuaContext, Value as LuaValue};
use candle_core::Device;

pub struct QueryExecutor {
    namespace_manager: Arc<NamespaceManager>,
    llm: Arc<LLMWrapper>,
    embedding: Arc<EmbeddingWrapper>,
    lua_vm: Arc<LuaVM>,
    file_storage: Arc<FileStorage>,
    auth_manager: Arc<AuthManager>,
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
            namespace_manager: Arc::new(namespace_manager),
            llm: Arc::new(llm),
            embedding: Arc::new(embedding),
            lua_vm: Arc::new(lua_vm),
            file_storage: Arc::new(file_storage),
            auth_manager: Arc::new(auth_manager),
            llm_semaphore: Arc::new(Semaphore::new(max_concurrent_llm)),
            embedding_semaphore: Arc::new(Semaphore::new(max_concurrent_embedding)),
        }
    }

    #[instrument(skip(self, query))]
    pub async fn execute(&self, query: &str, user_id: &str) -> Result<String> {
        let result = self.lua_vm.execute_with_context(|lua_ctx| {
            self.register_db_functions(lua_ctx)?;
            lua_ctx.load(query).eval()
        })?;

        Ok(self.format_lua_result(result))
    }

    fn register_db_functions(&self, lua_ctx: LuaContext, user_id: &str) -> Result<()> {
        let namespace_manager = self.namespace_manager.clone();
        let llm = self.llm.clone();
        let embedding = self.embedding.clone();
        let file_storage = self.file_storage.clone();
        let auth_manager = self.auth_manager.clone();
        let llm_semaphore = self.llm_semaphore.clone();
        let embedding_semaphore = self.embedding_semaphore.clone();

        lua_ctx.globals().set("select", lua_ctx.create_function(move |_, (namespace, key): (String, String)| {
            let ns = namespace_manager.get_namespace(&namespace).context("Namespace not found")?;
            let value = ns.db.get(key.as_bytes()).context("Failed to retrieve value")?;
            Ok(value.map(|v| String::from_utf8_lossy(&v).into_owned()))
        })?)?;

        lua_ctx.globals().set("insert", lua_ctx.create_function(move |_, (namespace, key, value): (String, String, String)| {
            let ns = namespace_manager.get_namespace(&namespace).context("Namespace not found")?;
            ns.db.put(key.as_bytes(), value.as_bytes()).context("Failed to insert value")?;
            Ok(())
        })?)?;

        // Add more functions for update, delete, create_namespace, delete_namespace, etc.

        lua_ctx.globals().set("generate_embedding", lua_ctx.create_function(move |_, text: String| {
            let permit = embedding_semaphore.try_acquire().context("Failed to acquire embedding semaphore")?;
            let embedding_result = embedding.generate(&text).context("Failed to generate embedding")?;
            drop(permit);
            Ok(embedding_result)
        })?)?;

        let llm = self.llm.clone();
        let llm_semaphore = self.llm_semaphore.clone();

        lua_ctx.globals().set("llm_query", lua_ctx.create_function(move |_, (prompt, max_tokens): (String, usize)| {
            let permit = llm_semaphore.try_acquire().context("Failed to acquire LLM semaphore")?;
            let llm_result = tokio::runtime::Runtime::new()?.block_on(llm.generate(&prompt, max_tokens))?;
            drop(permit);
            Ok(llm_result)
        })?)?;

        // Add more functions for file operations, similarity search, etc.

        Ok(())
    }

    fn format_lua_result(&self, result: LuaValue) -> String {
        match result {
            LuaValue::Nil => "nil".to_string(),
            LuaValue::Boolean(b) => b.to_string(),
            LuaValue::Integer(i) => i.to_string(),
            LuaValue::Number(n) => n.to_string(),
            LuaValue::String(s) => s.to_str().unwrap_or("").to_string(),
            LuaValue::Table(t) => format!("{:?}", t),
            _ => format!("{:?}", result),
        }
    }
}