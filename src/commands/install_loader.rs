use conduit_cli::core::error::CoreResult;
use conduit_cli::core::paths::CorePaths;  
use conduit_cli::core::loader_installer::install_loader;  
use crate::ui::CliUi;  
  
pub async fn run() -> CoreResult<()> {  
    let paths = CorePaths::from_project_dir(".")?;  
    let mut ui = CliUi::new();  
      
    install_loader(&paths, &mut ui).await  
}
