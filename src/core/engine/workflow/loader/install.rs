use crate::core::domain::loader::Loader;
use crate::core::engine::resolver::loader::ResolvedLoader;
use crate::core::engine::workflow::Workflow;
use crate::core::schemas::lock::{HashKind, Lockfile};
use crate::core::schemas::manifest::Manifest;
use crate::errors::{ConduitError, ConduitResult};
use std::io::Error;
use std::path::Path;
use std::process::Command;

impl Workflow {
    pub async fn execute_installation(
        &self,
        resolved: &ResolvedLoader,
        hash: &str,
        kind: HashKind,
        loader_type: &Loader,
        mc_version: &str,
    ) -> ConduitResult<()> {
        let is_installer = resolved.file_name.contains("installer");
        let target_name = if is_installer {
            "installer.jar"
        } else {
            "server.jar"
        };
        let target_path = self.project_root.join(target_name);

        if is_installer {
            let source = self.ctx.store.object_path(hash, kind);
            tokio::fs::copy(source, &target_path).await?;

            self.run_java_installer(&target_path, loader_type, mc_version)?;
            let _ = tokio::fs::remove_file(target_path).await;
        } else {
            self.ctx.store.link_object(hash, kind, &target_path).await?;
        }

        self.post_install_cleanup(loader_type).await
    }

    fn run_java_installer(
        &self,
        path: &std::path::Path,
        loader_type: &Loader,
        mc_version: &str,
    ) -> ConduitResult<()> {
        let mut cmd = Command::new("java");
        cmd.arg("-jar").arg(path);

        match loader_type {
            Loader::Fabric => {
                cmd.arg("server")
                    .arg("-mcversion")
                    .arg(mc_version)
                    .arg("-downloadMinecraft");
            }
            _ => {
                cmd.arg("--installServer");
            }
        }

        let status = cmd
            .current_dir(&self.project_root)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| ConduitError::Io(Error::other(e.to_string())))?;

        if !status.success() {
            return Err(ConduitError::Io(Error::other(
                "installer exited with error",
            )));
        }
        Ok(())
    }

    async fn post_install_cleanup(&self, loader_type: &Loader) -> ConduitResult<()> {
        tokio::fs::write(self.project_root.join("eula.txt"), "eula=true").await?;

        if let Loader::Forge { version } = loader_type {
            let mc_version = version.split('-').next().unwrap_or("");
            let is_old = mc_version
                .split('.')
                .nth(1)
                .and_then(|v| v.parse::<u32>().ok())
                .is_some_and(|v| v <= 16);

            if is_old {
                let mut entries = tokio::fs::read_dir(&self.project_root).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if name.contains("forge")
                        && Path::new(&name)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
                        && !name.contains("installer")
                    {
                        let _ =
                            tokio::fs::rename(entry.path(), self.project_root.join("server.jar"))
                                .await;
                        break;
                    }
                }
            }
        }

        let files_to_delete = [
            "installer.jar.log",
            "run.sh",
            "run.bat",
            "user_jvm_args.txt",
        ];
        for file in files_to_delete {
            let path = self.project_root.join(file);
            if path.exists() {
                let _ = tokio::fs::remove_file(path).await;
            }
        }
        Ok(())
    }

    pub fn ensure_loader_presence(
        &self,
        lock: &Lockfile,
        manifest: &Manifest,
    ) -> ConduitResult<bool> {
        if lock.instance.loader != manifest.project.loader
            || lock.instance.minecraft_version != manifest.project.minecraft
        {
            return Ok(false);
        }

        match &lock.instance.loader {
            Loader::Neoforge { .. } => {
                let libs = self.project_root.join("libraries");
                Ok(libs.exists())
            }
            Loader::Forge { version } => {
                if Self::is_modern_forge(version) {
                    let libs = self.project_root.join("libraries");
                    Ok(libs.exists())
                } else {
                    Ok(self.project_root.join("server.jar").exists())
                }
            }
            Loader::Fabric => {
                let has_launch_jar = self.project_root.join("fabric-server-launch.jar").exists();
                let has_server_jar = self.project_root.join("server.jar").exists();
                Ok(has_launch_jar || has_server_jar)
            }
            _ => Ok(self.project_root.join("server.jar").exists()),
        }
    }
}
