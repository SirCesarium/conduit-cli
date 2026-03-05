use conduit_cli::core::paths::CorePaths;  
use conduit_cli::core::loader_installer::install_loader;  
use crate::ui::CliUi;  
use std::error::Error;  
  
pub async fn run() -> Result<(), Box<dyn Error>> {  
    let paths = CorePaths::from_project_dir(".")?;  
    let mut ui = CliUi::new();  
      
    install_loader(&paths, &mut ui).await  
}
