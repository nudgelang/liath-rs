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
        let result: String = self.lua_vm.execute_with_context(|lua_ctx| {
            self.register_db_functions(lua_ctx, user_id)?;
            lua_ctx.load(query).eval()
        })?;

        Ok(result)
    }

    fn register_db_functions(&self, lua_ctx: LuaContext, user_id: &str) -> Result<()> {
        let namespace_manager = self.namespace_manager.clone();
        let llm = self.llm.clone();
        let embedding = self.embedding.clone();
        let file_storage = self.file_storage.clone();
        let auth_manager = self.auth_manager.clone();
        let llm_semaphore = self.llm_semaphore.clone();
        let embedding_semaphore = self.embedding_semaphore.clone();

        // Database operations
        lua_ctx.globals().set("select", lua_ctx.create_function(move |_, (namespace, key): (String, String)| {
            if !auth_manager.is_authorized(user_id, "select") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.get_namespace(&namespace).ok_or_else(|| rlua::Error::RuntimeError("Namespace not found".to_string()))?;
            let value = ns.db.get(key.as_bytes()).map_err(|e| rlua::Error::RuntimeError(format!("Failed to retrieve value: {}", e)))?;
            Ok(value.map(|v| String::from_utf8_lossy(&v).into_owned()))
        })?)?;

        lua_ctx.globals().set("insert", lua_ctx.create_function(move |_, (namespace, key, value): (String, String, String)| {
            if !auth_manager.is_authorized(user_id, "insert") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.get_namespace(&namespace).ok_or_else(|| rlua::Error::RuntimeError("Namespace not found".to_string()))?;
            ns.db.put(key.as_bytes(), value.as_bytes()).map_err(|e| rlua::Error::RuntimeError(format!("Failed to insert value: {}", e)))?;
            Ok(())
        })?)?;

        lua_ctx.globals().set("update", lua_ctx.create_function(move |_, (namespace, key, value): (String, String, String)| {
            if !auth_manager.is_authorized(user_id, "update") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.get_namespace(&namespace).ok_or_else(|| rlua::Error::RuntimeError("Namespace not found".to_string()))?;
            ns.db.put(key.as_bytes(), value.as_bytes()).map_err(|e| rlua::Error::RuntimeError(format!("Failed to update value: {}", e)))?;
            Ok(())
        })?)?;

        lua_ctx.globals().set("delete", lua_ctx.create_function(move |_, (namespace, key): (String, String)| {
            if !auth_manager.is_authorized(user_id, "delete") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.get_namespace(&namespace).ok_or_else(|| rlua::Error::RuntimeError("Namespace not found".to_string()))?;
            ns.db.delete(key.as_bytes()).map_err(|e| rlua::Error::RuntimeError(format!("Failed to delete value: {}", e)))?;
            Ok(())
        })?)?;

        // Namespace operations
        lua_ctx.globals().set("create_namespace", lua_ctx.create_function(move |_, name: String| {
            if !auth_manager.is_authorized(user_id, "create_namespace") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            namespace_manager.create_namespace(&name).map_err(|e| rlua::Error::RuntimeError(format!("Failed to create namespace: {}", e)))?;
            Ok(())
        })?)?;

        lua_ctx.globals().set("delete_namespace", lua_ctx.create_function(move |_, name: String| {
            if !auth_manager.is_authorized(user_id, "delete_namespace") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            namespace_manager.delete_namespace(&name).map_err(|e| rlua::Error::RuntimeError(format!("Failed to delete namespace: {}", e)))?;
            Ok(())
        })?)?;

        // Embedding operations
        lua_ctx.globals().set("generate_embedding", lua_ctx.create_function(move |_, text: String| {
            if !auth_manager.is_authorized(user_id, "generate_embedding") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            let permit = embedding_semaphore.try_acquire().map_err(|e| rlua::Error::RuntimeError(format!("Failed to acquire embedding semaphore: {}", e)))?;
            let embedding_result = embedding.generate(vec![&text]).map_err(|e| rlua::Error::RuntimeError(format!("Failed to generate embedding: {}", e)))?;
            drop(permit);
            Ok(embedding_result)
        })?)?;

        // LLM operations
        lua_ctx.globals().set("llm_query", lua_ctx.create_function(move |_, (prompt, sample_len, temp, repeat_penalty, repeat_last_n): (String, usize, f64, f32, usize)| {
            if !auth_manager.is_authorized(user_id, "llm_query") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            let permit = llm_semaphore.try_acquire().map_err(|e| rlua::Error::RuntimeError(format!("Failed to acquire LLM semaphore: {}", e)))?;
            let llm_result = llm.generate(&prompt, sample_len, temp, repeat_penalty, repeat_last_n)
                .map_err(|e| rlua::Error::RuntimeError(format!("Failed to generate LLM response: {}", e)))?;
            drop(permit);
            Ok(llm_result)
        })?)?;

        // File operations
        lua_ctx.globals().set("upload_file", lua_ctx.create_function(move |_, (file_name, content): (String, Vec<u8>)| {
            if !auth_manager.is_authorized(user_id, "upload_file") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            let file_id = file_storage.store(&content).map_err(|e| rlua::Error::RuntimeError(format!("Failed to store file: {}", e)))?;
            Ok(file_id)
        })?)?;

        lua_ctx.globals().set("retrieve_file", lua_ctx.create_function(move |_, file_id: String| {
            if !auth_manager.is_authorized(user_id, "retrieve_file") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            let content = file_storage.retrieve(&file_id).map_err(|e| rlua::Error::RuntimeError(format!("Failed to retrieve file: {}", e)))?;
            Ok(content)
        })?)?;

        // Vector search operations
        lua_ctx.globals().set("similarity_search", lua_ctx.create_function(move |_, (namespace, vector, k): (String, Vec<f32>, usize)| {
            if !auth_manager.is_authorized(user_id, "similarity_search") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.get_namespace(&namespace).ok_or_else(|| rlua::Error::RuntimeError("Namespace not found".to_string()))?;
            let results = ns.vector_db.search(&vector, k).map_err(|e| rlua::Error::RuntimeError(format!("Failed to perform similarity search: {}", e)))?;
            Ok(results)
        })?)?;

        // LuaRocks package management
        lua_ctx.globals().set("install_package", lua_ctx.create_function(move |_, package_name: String| {
            if !auth_manager.is_authorized(user_id, "install_package") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            self.lua_vm.install_package(&package_name).map_err(|e| rlua::Error::RuntimeError(format!("Failed to install package: {}", e)))?;
            Ok(())
        })?)?;

        lua_ctx.globals().set("list_packages", lua_ctx.create_function(move |_, ()| {
            if !auth_manager.is_authorized(user_id, "list_packages") {
                return Err(rlua::Error::RuntimeError("Unauthorized".to_string()));
            }
            let packages = self.lua_vm.list_installed_packages().map_err(|e| rlua::Error::RuntimeError(format!("Failed to list packages: {}", e)))?;
            Ok(packages)
        })?)?;

        Ok(())
    }
}