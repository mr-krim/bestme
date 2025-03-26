use anyhow::{Result, Error};
use log::{error, LevelFilter};

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
    
    // Run the application
    if let Err(e) = bestme::run() {
        error!("Application error: {}", e);
        // Get the full error chain
        let mut err = e.source();
        while let Some(source) = err {
            error!("Caused by: {}", source);
            err = source.source();
        }
        return Err(e);
    }
    
    Ok(())
} 
