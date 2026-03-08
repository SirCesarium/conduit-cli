use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Deserialize)]
pub struct NeoForgeMetadata {
    pub dependencies: Option<std::collections::HashMap<String, Vec<Dependency>>>,
}

#[derive(Debug, Deserialize)]
pub struct NeoForgeModsList {
    pub mods: Option<Vec<NeoForgeModEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct NeoForgeModEntry {
    #[serde(rename = "modId")]
    pub mod_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Dependency {
    #[serde(rename = "modId")]
    pub mod_id: String,
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct JarJarMetadata {
    pub jars: Vec<JarJarEntry>,
}

#[derive(Debug, Deserialize)]
pub struct JarJarEntry {
    pub identifier: JarJarIdentifier,
}

#[derive(Debug, Deserialize)]
pub struct JarJarIdentifier {
    pub artifact: String,
}

pub struct JarInspector;

impl JarInspector {
    pub fn inspect_neoforge<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        let toml_content = {
            let mut toml_file = archive.by_name("META-INF/neoforge.mods.toml")?;
            let mut content = String::new();
            toml_file.read_to_string(&mut content)?;
            content
        };
        
        let decoded: NeoForgeMetadata = toml::from_str(&toml_content)?;

        let mut embedded_mods = Vec::new();
        if let Ok(mut jarjar_file) = archive.by_name("META-INF/jarjar/metadata.json") {
            let mut jarjar_content = String::new();
            jarjar_file.read_to_string(&mut jarjar_content)?;
            if let Ok(jarjar_data) = serde_json::from_str::<JarJarMetadata>(&jarjar_content) {
                for entry in jarjar_data.jars {
                    let clean_id = entry.identifier.artifact.split('-').next().unwrap_or("").to_string();
                    embedded_mods.push(clean_id.to_lowercase());
                    embedded_mods.push(entry.identifier.artifact.to_lowercase());
                }
            }
        }

        let mut required_deps = Vec::new();

        if let Some(all_deps) = decoded.dependencies {
            for (_, deps_list) in all_deps {
                for dep in deps_list {
                    let dep_id_lower = dep.mod_id.to_lowercase();
                    
                    if dep.r#type == "required"
                        && dep_id_lower != "neoforge"
                        && dep_id_lower != "minecraft"
                        && !embedded_mods.iter().any(|embedded| dep_id_lower.contains(embedded))
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

    pub fn extract_primary_mod_id<P: AsRef<Path>>(
        path: P,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        let toml_content = if let Ok(mut toml_file) = archive.by_name("META-INF/neoforge.mods.toml") {
            let mut content = String::new();
            toml_file.read_to_string(&mut content)?;
            content
        } else if let Ok(mut toml_file) = archive.by_name("META-INF/mods.toml") {
            let mut content = String::new();
            toml_file.read_to_string(&mut content)?;
            content
        } else {
            return Ok(None);
        };

        let decoded: NeoForgeModsList = toml::from_str(&toml_content)?;
        let mod_id = decoded
            .mods
            .unwrap_or_default()
            .into_iter()
            .next()
            .map(|m| m.mod_id);
        Ok(mod_id)
    }
}
