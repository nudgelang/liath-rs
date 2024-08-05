use crate::core::NamespaceManager;
use crate::ai::{LLMWrapper, EmbeddingWrapper};
use crate::lua::LuaVM;
use crate::file::FileStorage;
use crate::auth::AuthManager;
use anyhow::{Result, Context};
use tokio::sync::Semaphore;
use std::sync::{Arc, RwLock};
use std::cell::RefCell;
use tracing::{info, error, instrument};
use rlua::{Context as LuaContext, Error as LuaError, Lua, Value as LuaValue};
use candle_core::Device;
use usearch::{MetricKind, ScalarKind};

pub struct QueryExecutor {
    namespace_manager: Arc<RwLock<NamespaceManager>>,
    llm: Arc<RwLock<LLMWrapper>>,
    embedding: Arc<RwLock<EmbeddingWrapper>>,
    lua_vm: Arc<RwLock<LuaVM>>,
    file_storage: Arc<RwLock<FileStorage>>,
    auth_manager: Arc<RwLock<AuthManager>>,
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
            namespace_manager: Arc::new(RwLock::new(namespace_manager)),
            llm: Arc::new(RwLock::new(llm)),
            embedding: Arc::new(RwLock::new(embedding)),
            lua_vm: Arc::new(RwLock::new(lua_vm)),
            file_storage: Arc::new(RwLock::new(file_storage)),
            auth_manager: Arc::new(RwLock::new(auth_manager)),
            llm_semaphore: Arc::new(Semaphore::new(max_concurrent_llm)),
            embedding_semaphore: Arc::new(Semaphore::new(max_concurrent_embedding)),
        }
    }

    #[instrument(skip(self, query))]
    pub async fn execute(&self, query: &str, user_id: &str) -> Result<String> {
        let result = self.lua_vm.read().unwrap().execute_with_context(|lua_ctx| {
            self.register_db_functions(&lua_ctx, user_id.to_string())
                .map_err(|e| LuaError::ExternalError(Arc::new(e)))?;
            lua_ctx.load(query).eval()
        })?;

        // Convert the LuaValue to a String
        match result {
            LuaValue::String(s) => Ok(s.to_str()?.to_owned()),
            LuaValue::Number(n) => Ok(n.to_string()),
            LuaValue::Integer(i) => Ok(i.to_string()),
            LuaValue::Boolean(b) => Ok(b.to_string()),
            LuaValue::Nil => Ok("nil".to_owned()),
            _ => Err(anyhow!("Unexpected Lua return type")),
        }
    }

    fn register_db_functions(&self, lua_ctx: &LuaContext, user_id: String) -> Result<()> {
        let namespace_manager = self.namespace_manager.clone();
        let llm = self.llm.clone();
        let embedding = self.embedding.clone();
        let file_storage = self.file_storage.clone();
        let auth_manager = self.auth_manager.clone();
        let llm_semaphore = self.llm_semaphore.clone();
        let embedding_semaphore = self.embedding_semaphore.clone();
        let lua_vm = self.lua_vm.clone();

        let user_id = RefCell::new(user_id);

        // Namespace operations
        lua_ctx.globals().set("create_namespace", lua_ctx.create_function_mut(move |_, (name, dimensions, metric, scalar): (String, usize, String, String)| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "create_namespace") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let metric = match metric.as_str() {
                "cosine" => MetricKind::Cos,
                "euclidean" => MetricKind::L2sq,
                _ => return Err(LuaError::RuntimeError("Invalid metric kind".to_string())),
            };
            let scalar = match scalar.as_str() {
                "f32" => ScalarKind::F32,
                "f16" => ScalarKind::F16,
                _ => return Err(LuaError::RuntimeError("Invalid scalar kind".to_string())),
            };
            namespace_manager.write().unwrap().create_namespace(&name, dimensions, metric, scalar)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to create namespace: {}", e)))
        })?)?;

        lua_ctx.globals().set("delete_namespace", lua_ctx.create_function_mut(move |_, name: String| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "delete_namespace") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            namespace_manager.write().unwrap().delete_namespace(&name)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to delete namespace: {}", e)))
        })?)?;

        lua_ctx.globals().set("list_namespaces", lua_ctx.create_function_mut(move |lua_ctx, ()| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "list_namespaces") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let namespaces = namespace_manager.read().unwrap().list_namespaces();
            let lua_namespaces = lua_ctx.create_table()?;
            for (i, namespace) in namespaces.iter().enumerate() {
                lua_namespaces.set(i + 1, namespace.clone())?;
            }
            Ok(lua_namespaces)
        })?)?;

        // Database operations
        lua_ctx.globals().set("select", lua_ctx.create_function_mut(move |_, (namespace, key): (String, String)| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "select") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;
            let value = ns.db.get(key.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to retrieve value: {}", e)))?;
            Ok(value.map(|v| String::from_utf8_lossy(&v).into_owned()))
        })?)?;

        lua_ctx.globals().set("insert", lua_ctx.create_function_mut(move |_, (namespace, key, value): (String, String, String)| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "insert") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;
            ns.db.put(key.as_bytes(), value.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to insert value: {}", e)))?;
            Ok(())
        })?)?;

        lua_ctx.globals().set("update", lua_ctx.create_function_mut(move |_, (namespace, key, value): (String, String, String)| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "update") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;
            ns.db.put(key.as_bytes(), value.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to update value: {}", e)))?;
            Ok(())
        })?)?;

        lua_ctx.globals().set("delete", lua_ctx.create_function_mut(move |_, (namespace, key): (String, String)| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "delete") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;
            ns.db.delete(key.as_bytes())
                .map_err(|e| LuaError::RuntimeError(format!("Failed to delete value: {}", e)))?;
            Ok(())
        })?)?;
        // Embedding operations
        lua_ctx.globals().set("generate_embedding", lua_ctx.create_function_mut(move |lua_ctx, texts: Vec<String>| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "generate_embedding") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let permit = embedding_semaphore.try_acquire()
                .map_err(|e| LuaError::RuntimeError(format!("Failed to acquire embedding semaphore: {}", e)))?;
            
            let embedding_results = embedding.read().unwrap().generate(&texts)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to generate embeddings: {}", e)))?;
            drop(permit);
            
            let lua_embeddings = lua_ctx.create_table()?;
            for (i, embedding) in embedding_results.iter().enumerate() {
                let lua_embedding = lua_ctx.create_table()?;
                for (j, value) in embedding.iter().enumerate() {
                    lua_embedding.set(j + 1, *value)?;
                }
                lua_embeddings.set(i + 1, lua_embedding)?;
            }
            Ok(lua_embeddings)
        })?)?;

        // LLM operations
        lua_ctx.globals().set("llm_query", lua_ctx.create_function_mut(move |_, (prompt, sample_len, temp, repeat_penalty, repeat_last_n): (String, usize, f64, f32, usize)| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "llm_query") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let permit = llm_semaphore.try_acquire()
                .map_err(|e| LuaError::RuntimeError(format!("Failed to acquire LLM semaphore: {}", e)))?;
            let llm_result = llm.read().unwrap().generate(&prompt, sample_len, temp, repeat_penalty, repeat_last_n)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to generate LLM response: {}", e)))?;
            drop(permit);
            Ok(llm_result)
        })?)?;

        // File operations
        lua_ctx.globals().set("upload_file", lua_ctx.create_function_mut(move |_, (file_name, content): (String, Vec<u8>)| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "upload_file") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let file_id = file_storage.read().unwrap().store(&content)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to store file: {}", e)))?;
            Ok(file_id)
        })?)?;

        lua_ctx.globals().set("retrieve_file", lua_ctx.create_function_mut(move |lua_ctx, file_id: String| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "retrieve_file") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let content = file_storage.read().unwrap().retrieve(&file_id)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to retrieve file: {}", e)))?;
            let lua_content = lua_ctx.create_string(&content)?;
            Ok(lua_content)
        })?)?;

        // Vector search operations
        lua_ctx.globals().set("similarity_search", lua_ctx.create_function_mut(move |lua_ctx, (namespace, vector, k): (String, Vec<f32>, usize)| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "similarity_search") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let ns = namespace_manager.read().unwrap().get_namespace(&namespace)
                .map_err(|e| LuaError::RuntimeError(format!("Namespace error: {}", e)))?;
            let results = ns.vector_db.search(&vector, k)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to perform similarity search: {}", e)))?;
            
            let lua_results = lua_ctx.create_table()?;
            for (i, (id, distance)) in results.into_iter().enumerate() {
                let result_table = lua_ctx.create_table()?;
                result_table.set("id", id)?;
                result_table.set("distance", distance)?;
                lua_results.set(i + 1, result_table)?;
            }
            Ok(lua_results)
        })?)?;

        // LuaRocks package management
        lua_ctx.globals().set("install_package", lua_ctx.create_function_mut(move |_, package_name: String| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "install_package") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            lua_vm.read().unwrap().install_package(&package_name)
                .map_err(|e| LuaError::RuntimeError(format!("Failed to install package: {}", e)))?;
            Ok(())
        })?)?;

        lua_ctx.globals().set("list_packages", lua_ctx.create_function_mut(move |lua_ctx, ()| {
            let user_id = user_id.borrow().clone();
            if !auth_manager.read().unwrap().is_authorized(&user_id, "list_packages") {
                return Err(LuaError::RuntimeError("Unauthorized".to_string()));
            }
            let packages = lua_vm.read().unwrap().list_installed_packages()
                .map_err(|e| LuaError::RuntimeError(format!("Failed to list packages: {}", e)))?;
            let lua_packages = lua_ctx.create_table()?;
            for (i, package) in packages.iter().enumerate() {
                lua_packages.set(i + 1, package.clone())?;
            }
            Ok(lua_packages)
        })?)?;

        Ok(())
    }
}