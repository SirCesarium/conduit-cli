use std::{path::PathBuf, sync::Arc};

use crate::{api::ConduitAPI, core::{ConduitContext, resolver::Resolver}, domain::loader::Loader};

mod api;
mod core;
mod domain;
mod schemas;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store_path = PathBuf::from("./.conduit_store");
    let ctx = Arc::new(ConduitContext::new(store_path));
    let resolver = Resolver::new(ctx.clone());

    let loaders_to_test = vec![
        Loader::Paper { version: "1.21.1".to_string() },
        Loader::Fabric { version: "1.21.1".to_string() },
        Loader::Vanilla { version: "1.21.1".to_string() },
        Loader::Neoforge { version: "1.21.1".to_string() },
        Loader::Fabric { version: "1.21.1".to_string() },
        Loader::Purpur { version: "1.21.1".to_string() },
        Loader::Forge { version: "1.21.1".to_string() },
    ];

    for loader in loaders_to_test {
        println!("Resolving {:?}...", loader);
        
        match resolver.resolve_loader(&loader).await {
            Ok(resolved) => {
                println!("  File: {}", resolved.file_name);
                println!("  URL:  {}", resolved.url);
                println!("  Hash: {}", if resolved.hash.is_empty() { "None" } else { &resolved.hash });
            }
            Err(e) => eprintln!("  Error resolving loader: {}", e),
        }
        println!("---");
    }

    Ok(())
}

