use std::process::Command;
use anyhow::{Result, Context};
use std::path::PathBuf;

pub struct LuaRocks {
    luarocks_path: PathBuf,
}

impl LuaRocks {
    pub fn new(luarocks_path: PathBuf) -> Self {
        Self { luarocks_path }
    }

    pub fn install_package(&self, package_name: &str) -> Result<()> {
        let output = Command::new(&self.luarocks_path)
            .arg("install")
            .arg(package_name)
            .output()
            .context("Failed to execute LuaRocks")?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install package: {}", error_message));
        }

        Ok(())
    }

    pub fn list_installed_packages(&self) -> Result<Vec<String>> {
        let output = Command::new(&self.luarocks_path)
            .arg("list")
            .output()
            .context("Failed to execute LuaRocks")?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to list packages: {}", error_message));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.lines().map(|line| line.to_string()).collect())
    }
}