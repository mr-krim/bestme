fn main() {
    // Link against X11 libraries for text injection support on Linux
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=X11");
        println!("cargo:rustc-link-lib=Xtst");
        println!("cargo:rustc-link-lib=Xfixes");
    }
}