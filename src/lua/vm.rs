use rlua::{Lua, Context, Result};
use std::sync::Arc;
use crate::lua::modules::load_all_modules;

pub struct LuaVM {
    lua: Lua,
}

impl LuaVM {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        lua.context(|ctx| {
            load_all_modules(ctx)?;
            Ok(())
        })?;
        Ok(Self { lua })
    }

    pub fn execute(&self, code: &str) -> Result<String> {
        self.lua.context(|ctx| {
            ctx.load(code).eval()
        })
    }

    pub fn register_function<F>(&self, name: &str, func: F) -> Result<()>
    where
        F: 'static + Send + Sync + Fn(Context, rlua::Value) -> Result<rlua::Value>,
    {
        self.lua.context(|ctx| {
            let globals = ctx.globals();
            globals.set(name, ctx.create_function(func)?)
        })
    }
}