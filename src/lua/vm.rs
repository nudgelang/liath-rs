use rlua::{Lua, Context, Result, Value, FromLua};
use std::sync::Arc;
use crate::lua::luarocks::LuaRocks;
use std::path::PathBuf;

pub struct LuaVM {
    lua: Lua,
    luarocks: Arc<LuaRocks>,
}

impl LuaVM {
    pub fn new(luarocks_path: PathBuf) -> Result<Self> {
        let lua = Lua::new();
        let luarocks = Arc::new(LuaRocks::new(luarocks_path));

        Ok(Self { lua, luarocks })
    }

    pub fn execute<T: for<'lua> FromLua<'lua>>(&self, code: &str) -> Result<T> {
        self.lua.context(|ctx| {
            let result: Value = ctx.load(code).eval()?;
            T::from_lua(result, ctx)
        })
    }

    pub fn execute_with_context<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(Context) -> Result<R>,
    {
        self.lua.context(f)
    }

    pub fn register_function<F>(&self, name: &str, func: F) -> Result<()>
    where
        F: for<'lua> Fn(Context<'lua>, Value<'lua>) -> Result<Value<'lua>> + 'static + Send + Sync,
    {
        self.lua.context(|ctx| {
            let globals = ctx.globals();
            globals.set(name, ctx.create_function(func)?)
        })
    }

    pub fn install_package(&self, package_name: &str) -> anyhow::Result<()> {
        self.luarocks.install_package(package_name)
    }

    pub fn list_installed_packages(&self) -> anyhow::Result<Vec<String>> {
        self.luarocks.list_installed_packages()
    }
}