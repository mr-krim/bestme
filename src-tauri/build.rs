fn main() {
    // Link against X11 libraries for text injection support
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=X11");
        println!("cargo:rustc-link-lib=Xtst");
    }
    
    tauri_build::build()
} 
