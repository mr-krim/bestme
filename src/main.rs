use anyhow::Result;
use log::{error, info, LevelFilter};
use std::env;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
    
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let use_gui = args.iter().any(|arg| arg == "--gui");
    let verbose = args.iter().any(|arg| arg == "--verbose");
    
    if verbose {
        // Enable more detailed logging
        env_logger::Builder::new()
            .filter_level(LevelFilter::Debug)
            .init();
        info!("Verbose logging enabled");
    }
    
    // Run the application
    if let Err(e) = bestme::run_with_options(use_gui) {
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
