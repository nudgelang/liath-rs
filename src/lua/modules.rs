use rlua::{Context, Result};
use reqwest;
use scraper;
use serde_json;
use ring;

pub fn load_http_module(ctx: Context) -> Result<()> {
    let http = ctx.create_table()?;
    http.set("get", ctx.create_function(|_, url: String| {
        let body = reqwest::blocking::get(&url)?.text()?;
        Ok(body)
    })?)?;
    http.set("post", ctx.create_function(|_, (url, body): (String, String)| {
        let response = reqwest::blocking::Client::new()
            .post(&url)
            .body(body)
            .send()?
            .text()?;
        Ok(response)
    })?)?;
    ctx.globals().set("http", http)?;
    Ok(())
}

pub fn load_html_module(ctx: Context) -> Result<()> {
    let html = ctx.create_table()?;
    html.set("parse", ctx.create_function(|_, html: String| {
        let document = scraper::Html::parse_document(&html);
        let title = document.select(&scraper::Selector::parse("title").unwrap())
            .next()
            .and_then(|e| e.text().next())
            .unwrap_or("");
        Ok(title.to_string())
    })?)?;
    ctx.globals().set("html", html)?;
    Ok(())
}

pub fn load_json_module(ctx: Context) -> Result<()> {
    let json = ctx.create_table()?;
    json.set("parse", ctx.create_function(|_, json: String| {
        let value: serde_json::Value = serde_json::from_str(&json)?;
        Ok(value.to_string())
    })?)?;
    json.set("stringify", ctx.create_function(|_, value: rlua::Value| {
        let json = serde_json::to_string(&value)?;
        Ok(json)
    })?)?;
    ctx.globals().set("json", json)?;
    Ok(())
}

pub fn load_crypto_module(ctx: Context) -> Result<()> {
    let crypto = ctx.create_table()?;
    crypto.set("sha256", ctx.create_function(|_, data: String| {
        use ring::digest::{Context, SHA256};
        let mut context = Context::new(&SHA256);
        context.update(data.as_bytes());
        let digest = context.finish();
        Ok(hex::encode(digest.as_ref()))
    })?)?;
    ctx.globals().set("crypto", crypto)?;
    Ok(())
}

pub fn load_all_modules(ctx: Context) -> Result<()> {
    load_http_module(ctx)?;
    load_html_module(ctx)?;
    load_json_module(ctx)?;
    load_crypto_module(ctx)?;
    Ok(())
}