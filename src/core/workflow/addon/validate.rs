use crate::{
    core::workflow::Workflow,
    domain::{addon::AddonType, loader::Loader},
    errors::{ConduitError, ConduitResult},
    schemas::manifest::Manifest,
};

impl Workflow {
    pub async fn validate_compatibility(
        &self,
        target: AddonType,
        manifest: &Manifest,
    ) -> ConduitResult<()> {
        let loader = &manifest.project.loader;
        let mc_version = &manifest.project.minecraft;

        match (&target, loader) {
            (AddonType::Mod, Loader::Vanilla | Loader::Paper | Loader::Purpur) => Err(
                ConduitError::Deserialize(format!("Cannot install Mods on a {loader:?} server")),
            ),
            (AddonType::Plugin, l) if !matches!(l, Loader::Paper | Loader::Purpur) => Err(
                ConduitError::Deserialize(format!("Plugins require Paper/Purpur (current: {l:?})")),
            ),
            (AddonType::Datapack, _) => {
                let minor = mc_version
                    .split('.')
                    .nth(1)
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0);
                if minor < 13 {
                    return Err(ConduitError::Deserialize(format!(
                        "Datapacks require MC 1.13+"
                    )));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
