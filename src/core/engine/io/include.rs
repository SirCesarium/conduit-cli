use crate::core::schemas::include;
use crate::errors::ConduitResult;
use ignore::WalkBuilder;
use serde::{Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};
use tokio::fs;

pub trait IncludeFile: Serialize + DeserializeOwned + Sync {
    fn from_patterns(patterns: Vec<String>) -> Self;
    fn get_patterns(&self) -> &[String];

    fn load<P: AsRef<Path> + Send>(
        path: P,
    ) -> impl std::future::Future<Output = ConduitResult<Self>> + Send {
        async {
            let content = fs::read_to_string(path).await?;
            let patterns = content
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect();

            Ok(Self::from_patterns(patterns))
        }
    }

    fn save<P: AsRef<Path> + Send>(
        &self,
        path: P,
    ) -> impl std::future::Future<Output = ConduitResult<()>> + Send {
        async move {
            let content = self.get_patterns().join("\n");
            fs::write(path, content).await?;
            Ok(())
        }
    }

    fn scan<P: AsRef<Path>>(&self, root: P) -> Vec<PathBuf> {
        let root_path = root.as_ref();
        let patterns = self.get_patterns();

        if patterns.is_empty() {
            return Vec::new();
        }

        let compiled: Vec<_> = patterns
            .iter()
            .map(|p| {
                let p = p.replace('\\', "/");
                if p.ends_with('/') {
                    format!("{p}**/*")
                } else if !p.contains('*') && !p.contains('.') {
                    format!("{p}/**/*")
                } else {
                    p
                }
            })
            .filter_map(|p| glob::Pattern::new(&p).ok())
            .collect();

        let mut included = Vec::new();

        let walker = WalkBuilder::new(root_path)
            .hidden(false)
            .git_ignore(false)
            .ignore(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();

            if let Ok(relative_path) = path.strip_prefix(root_path) {
                let path_str = relative_path.to_string_lossy().replace('\\', "/");

                if path_str.is_empty() || path_str == "." {
                    continue;
                }

                if compiled.iter().any(|g| g.matches(&path_str)) {
                    included.push(path.to_path_buf());
                }
            }
        }
        included
    }
}

impl IncludeFile for include::ConduitInclude {
    fn from_patterns(patterns: Vec<String>) -> Self {
        Self { paths: patterns }
    }

    fn get_patterns(&self) -> &[String] {
        &self.paths
    }
}
