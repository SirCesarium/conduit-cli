use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Deserialize)]
pub struct NeoForgeMetadata {
    pub dependencies: Option<std::collections::HashMap<String, Vec<Dependency>>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Dependency {
    #[serde(rename = "modId")]
    pub mod_id: String,
    pub r#type: String,
}

pub struct JarInspector;

impl JarInspector {
    pub fn inspect_neoforge<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut toml_file = archive.by_name("META-INF/neoforge.mods.toml")?;
        let mut content = String::new();
        toml_file.read_to_string(&mut content)?;

        let decoded: NeoForgeMetadata = toml::from_str(&content)?;

        let mut required_deps = Vec::new();

        if let Some(all_deps) = decoded.dependencies {
            for (_, deps_list) in all_deps {
                for dep in deps_list {
                    if dep.r#type == "required"
                        && dep.mod_id != "neoforge"
                        && dep.mod_id != "minecraft"
                    {
                        required_deps.push(dep.mod_id);
                    }
                }
            }
        }

        required_deps.sort();
        required_deps.dedup();

        Ok(required_deps)
    }
}
