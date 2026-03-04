#[derive(Debug, Clone)]
pub enum ExtraDepsPolicy {
    Skip,
    AutoExactMatch,
    Callback,
}

#[derive(Debug, Clone)]
pub struct ExtraDepCandidate {
    pub title: String,
    pub slug: String,
    pub is_exact_match: bool,
}

#[derive(Debug, Clone)]
pub struct ExtraDepRequest {
    pub tech_id: String,
    pub parent_slug: String,
    pub parent_filename: String,
    pub candidates: Vec<ExtraDepCandidate>,
}

#[derive(Debug, Clone)]
pub enum ExtraDepDecision {
    Skip,
    InstallSlug(String),
}

pub trait ExtraDepChooser {
    fn choose_extra_dep(&mut self, _request: ExtraDepRequest) -> ExtraDepDecision {
        ExtraDepDecision::Skip
    }
}

pub trait InstallerUi: crate::core::events::CoreCallbacks + ExtraDepChooser {}

impl<T> InstallerUi for T where T: crate::core::events::CoreCallbacks + ExtraDepChooser {}
