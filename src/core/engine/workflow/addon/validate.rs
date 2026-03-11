use crate::{
    core::domain::{addon::AddonType, loader::Loader},
    core::engine::workflow::Workflow,
    core::schemas::manifest::Manifest,
    errors::{ConduitError, ConduitResult},
};

impl Workflow {
    pub fn validate_compatibility(
        &self,
        target: &AddonType,
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
                    return Err(ConduitError::Deserialize(
                        "Datapacks require MC 1.13+".to_string(),
                    ));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
