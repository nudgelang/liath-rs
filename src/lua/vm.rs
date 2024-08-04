use rlua::{Lua, Context, Result};
use std::sync::Arc;

pub struct LuaVM {
    lua: Lua,
}

impl LuaVM {
    pub fn new() -> Self {
        Self { lua: Lua::new() }
    }

    pub fn execute(&self, code: &str) -> Result<()> {
        self.lua.context(|ctx| {
            ctx.load(code).exec()
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