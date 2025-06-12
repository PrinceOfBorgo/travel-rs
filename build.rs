use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:warning=Build script is running...");

    // Get the output directory from cargo
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap();
    println!("cargo:warning=Target directory: {}", target_dir.display());

    // Copy config directory
    println!("cargo:warning=Copying config directory...");
    copy_path("config", target_dir.join("config"));

    // Copy locales directory
    println!("cargo:warning=Copying locales directory...");
    copy_path("locales", target_dir.join("locales"));

    // Copy database build script
    println!("cargo:warning=Copying database build script...");
    copy_path(
        "database/build_travelers_db.surql",
        target_dir.join("database/build_travelers_db.surql"),
    );

    // Watch all files recursively
    println!("cargo:warning=Setting up file watchers...");
    watch_directory("config");
    watch_directory("locales");
    watch_directory("database");

    println!("cargo:warning=Build script completed!");
}

/// Copies a file or directory from `src` to `dst`.
/// If `src` is a directory, it will recursively copy all contents.
fn copy_path(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    let src = src.as_ref();
    let dst = dst.as_ref();

    if !src.exists() {
        println!("cargo:warning=Source does not exist: {}", src.display());
        return;
    }

    if src.is_dir() {
        if !dst.exists() {
            println!(
                "cargo:warning=Creating destination directory: {}",
                dst.display()
            );
            fs::create_dir_all(dst).unwrap();
        }
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            copy_path(src_path, dst_path);
        }
    } else {
        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).unwrap();
            }
        }
        println!(
            "cargo:warning=Copying file: {} -> {}",
            src.display(),
            dst.display()
        );
        fs::copy(src, dst).unwrap();
    }
}

/// Watches a directory and all its contents recursively for changes.
/// It will trigger a rebuild if any file changes.
fn watch_directory(dir: impl AsRef<Path>) {
    let dir = dir.as_ref();
    if !dir.exists() {
        return;
    }

    // Watch the directory itself
    println!("cargo:rerun-if-changed={}", dir.display());

    // Recursively watch all files and subdirectories
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            watch_directory(&path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
