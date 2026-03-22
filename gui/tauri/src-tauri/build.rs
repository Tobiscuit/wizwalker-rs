fn main() {
    if std::process::Command::new("x86_64-w64-mingw32-windres").arg("--version").output().is_ok() {
         tauri_build::build()
    } else {
        println!("cargo:warning=x86_64-w64-mingw32-windres not found, skipping tauri_build::build()");
    }
}
