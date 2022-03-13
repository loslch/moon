use crate::errors::ToolchainError;
use crate::helpers::{get_bin_version, get_path_env_var};
use crate::tool::{PackageManager, Tool};
use crate::Toolchain;
use async_trait::async_trait;
use moon_config::NpmConfig;
use moon_logger::{color, debug, trace};
use moon_utils::is_ci;
use moon_utils::process::{create_command, exec_command, Output};
use std::env::consts;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct NpmTool {
    bin_path: PathBuf,

    install_dir: PathBuf,

    npx_path: PathBuf,

    pub config: NpmConfig,
}

impl NpmTool {
    pub fn new(toolchain: &Toolchain, config: &NpmConfig) -> Result<NpmTool, ToolchainError> {
        let install_dir = toolchain.get_node().get_install_dir().clone();
        let mut bin_path = install_dir.clone();
        let mut npx_path = install_dir.clone();

        if consts::OS == "windows" {
            bin_path.push("npm.cmd");
            npx_path.push("npx.cmd");
        } else {
            bin_path.push("bin/npm");
            npx_path.push("bin/npx");
        }

        debug!(
            target: "moon:toolchain:npm",
            "Creating tool at {}",
            color::file_path(&bin_path)
        );

        Ok(NpmTool {
            bin_path,
            config: config.to_owned(),
            install_dir,
            npx_path,
        })
    }

    pub async fn add_global_dep(&self, name: &str, version: &str) -> Result<(), ToolchainError> {
        let package = format!("{}@{}", name, version);

        exec_command(
            create_command(self.get_bin_path())
                .args(["install", "-g", &package])
                .env("PATH", get_path_env_var(self.get_bin_dir()))
                .current_dir(&self.install_dir),
        )
        .await?;

        Ok(())
    }
}

#[async_trait]
impl Tool for NpmTool {
    fn is_downloaded(&self) -> bool {
        true
    }

    async fn download(&self, _host: Option<&str>) -> Result<(), ToolchainError> {
        trace!(
            target: "moon:toolchain:npm",
            "No download required as it comes bundled with Node.js"
        );

        Ok(()) // This is handled by node
    }

    async fn is_installed(&self) -> Result<bool, ToolchainError> {
        if self.bin_path.exists() {
            let version = self.get_installed_version().await?;

            if self.config.version == "inherit" {
                debug!(
                    target: "moon:toolchain:npm",
                    "Using the version ({}) that came bundled with Node.js",
                    version
                );

                return Ok(true);
            }

            if version == self.config.version {
                debug!(
                    target: "moon:toolchain:npm",
                    "Package has already been installed and is on the correct version",
                );

                return Ok(true);
            }

            debug!(
                target: "moon:toolchain:npm",
                "Package is on the wrong version ({}), attempting to reinstall",
                version
            );
        }

        Ok(false)
    }

    async fn install(&self, toolchain: &Toolchain) -> Result<(), ToolchainError> {
        if self.config.version == "inherit" {
            return Ok(());
        }

        let package = format!("npm@{}", self.config.version);

        if toolchain.get_node().is_corepack_aware() {
            debug!(
                target: "moon:toolchain:npm",
                "Enabling package manager with {}",
                color::shell(&format!("corepack prepare {} --activate", package))
            );

            toolchain
                .get_node()
                .exec_corepack(["prepare", &package, "--activate"])
                .await?;
        } else {
            debug!(
                target: "moon:toolchain:npm",
                "Installing package manager with {}",
                color::shell(&format!("npm install -g {}", package))
            );

            self.add_global_dep("npm", self.config.version.as_str())
                .await?;
        }

        Ok(())
    }

    fn get_bin_path(&self) -> &PathBuf {
        &self.bin_path
    }

    fn get_download_path(&self) -> Option<&PathBuf> {
        None
    }

    fn get_install_dir(&self) -> &PathBuf {
        &self.install_dir
    }

    async fn get_installed_version(&self) -> Result<String, ToolchainError> {
        Ok(get_bin_version(self.get_bin_path()).await?)
    }
}

#[async_trait]
impl PackageManager for NpmTool {
    async fn dedupe_dependencies(&self, toolchain: &Toolchain) -> Result<Output, ToolchainError> {
        Ok(exec_command(
            create_command(self.get_bin_path())
                .args(["dedupe"])
                .current_dir(&toolchain.workspace_root)
                .env("PATH", get_path_env_var(self.get_bin_dir())),
        )
        .await?)
    }

    async fn exec_package(
        &self,
        toolchain: &Toolchain,
        package: &str,
        args: Vec<&str>,
    ) -> Result<Output, ToolchainError> {
        let mut exec_args = vec!["--package", package, "--"];

        exec_args.extend(args);

        Ok(exec_command(
            create_command(&self.npx_path)
                .args(exec_args)
                .current_dir(&toolchain.workspace_root)
                .env("PATH", get_path_env_var(self.get_bin_dir())),
        )
        .await?)
    }

    fn get_lockfile_name(&self) -> String {
        String::from("package-lock.json")
    }

    fn get_workspace_dependency_range(&self) -> String {
        // Doesn't support "workspace:*"
        String::from("*")
    }

    async fn install_dependencies(&self, toolchain: &Toolchain) -> Result<Output, ToolchainError> {
        Ok(exec_command(
            create_command(self.get_bin_path())
                .args([if is_ci() { "ci" } else { "install" }])
                .current_dir(&toolchain.workspace_root)
                .env("PATH", get_path_env_var(self.get_bin_dir())),
        )
        .await?)
    }
}
