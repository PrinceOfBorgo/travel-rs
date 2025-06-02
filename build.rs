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
    copy_dir_recursive("config", target_dir.join("config"));

    // Copy locales directory
    println!("cargo:warning=Copying locales directory...");
    copy_dir_recursive("locales", target_dir.join("locales"));

    // Watch all files recursively
    println!("cargo:warning=Setting up file watchers...");
    watch_directory_recursive("config");
    watch_directory_recursive("locales");

    println!("cargo:warning=Build script completed!");
}

fn copy_dir_recursive(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    let src = src.as_ref();
    let dst = dst.as_ref();

    if !src.exists() {
        println!(
            "cargo:warning=Source directory does not exist: {}",
            src.display()
        );
        return;
    }

    if !dst.exists() {
        println!(
            "cargo:warning=Creating destination directory: {}",
            dst.display()
        );
        fs::create_dir_all(dst).unwrap();
    }

    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let ty = entry.file_type().unwrap();
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(src_path, dst_path);
        } else {
            println!(
                "cargo:warning=Copying file: {} -> {}",
                src_path.display(),
                dst_path.display()
            );
            fs::copy(src_path, dst_path).unwrap();
        }
    }
}

fn watch_directory_recursive(dir: impl AsRef<Path>) {
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
            watch_directory_recursive(&path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
