pub mod package;
pub mod system;

use package::PackageEvent;
use system::SystemEvent;

pub trait ConduitUI: Send + Sync {
    fn handle_package(&self, event: PackageEvent);
    fn handle_system(&self, event: SystemEvent);
    
    fn log(&self, msg: &str) {
        self.handle_system(system::SystemEvent::Log {
            level: system::NotificationLevel::Info,
            message: msg.to_string(),
        });
    }
}
