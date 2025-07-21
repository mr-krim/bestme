use bestme::app::App;
use bestme::config::ConfigManager;
use log::info;

/// Main entry point
fn main() {
    // Initialize logging with environment variables
    // Set RUST_LOG=debug to enable debug logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .format_module_path(true)
        .init();
    
    info!("Initializing BestMe application");
    
    // Check for GUI mode command-line argument
    let args: Vec<String> = std::env::args().collect();
    let force_gui = args.len() > 1 && (args[1] == "--gui" || args[1] == "-g");
    
    // Create and run app
    match ConfigManager::new() {
        Ok(config_manager) => {
            match App::new(config_manager) {
                Ok(mut app) => {
                    // Force GUI mode if requested
                    if force_gui {
                        info!("Forcing GUI mode from command-line argument");
                        app.set_gui_mode(true);
                    }
                    
                    if let Err(err) = app.run() {
                        eprintln!("Error running application: {}", err);
                        std::process::exit(1);
                    }
                },
                Err(err) => {
                    eprintln!("Error initializing application: {}", err);
                    std::process::exit(1);
                }
            }
        },
        Err(err) => {
            eprintln!("Error initializing config manager: {}", err);
            std::process::exit(1);
        }
    }
} 
