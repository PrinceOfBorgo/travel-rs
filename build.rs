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

    // Validate that DEPLOYMENT.md migration table is aligned with CHANGELOG
    validate_migration_table();

    // Watch all files recursively
    println!("cargo:warning=Setting up file watchers...");
    watch_directory("config");
    watch_directory("locales");
    watch_directory("database");
    println!("cargo:rerun-if-changed=CHANGELOG.md");
    println!("cargo:rerun-if-changed=DEPLOYMENT.md");
    println!("cargo:rerun-if-changed=Cargo.toml");

    println!("cargo:warning=Build script completed!");
}

/// Validates that every migration script referenced in each CHANGELOG version
/// section:
/// 1. exists on disk in `database/migrations/`
/// 2. has a corresponding row in the DEPLOYMENT.md Migration Reference table
fn validate_migration_table() {
    let changelog = fs::read_to_string("CHANGELOG.md").unwrap();
    let deployment = fs::read_to_string("DEPLOYMENT.md").unwrap();

    // Find all version section headers: `## [x.y.z...]`
    let mut errors = Vec::new();
    let mut search_from = 0;
    while let Some(header_start) = changelog[search_from..].find("\n## [").or_else(|| {
        // Handle the very first line (no leading newline)
        if search_from == 0 && changelog.starts_with("## [") {
            Some(0)
        } else {
            None
        }
    }) {
        let abs_start = if search_from == 0 && changelog.starts_with("## [") {
            0
        } else {
            search_from + header_start + 1 // skip the \n
        };

        // Extract version string from `## [<version>]`
        let after_bracket = abs_start + "## [".len();
        let Some(close) = changelog[after_bracket..].find(']') else {
            break;
        };
        let version = &changelog[after_bracket..after_bracket + close];

        // Extract the section body (until next `## [` or end of file)
        let body_start = after_bracket + close;
        let section = match changelog[body_start..].find("\n## [") {
            Some(end) => &changelog[body_start..body_start + end],
            None => &changelog[body_start..],
        };

        validate_version_section(version, section, &deployment, &mut errors);

        search_from = body_start + 1;
    }

    if !errors.is_empty() {
        panic!("\n\n{}\n", errors.join("\n\n"));
    }
}

/// Validates a single CHANGELOG version section.
fn validate_version_section(
    version: &str,
    section: &str,
    deployment: &str,
    errors: &mut Vec<String>,
) {
    // Collect migration script references from the section.
    //
    // CHANGELOG entries typically use markdown links:
    //   [`009_foo.surql`](database/migrations/009_foo.surql)
    // but may also use bare or backtick-wrapped filenames.
    //
    // We extract two things:
    //   - **link targets** (paths inside `(…)`) — used for file-existence checks
    //   - **display names** (filenames from any context) — used for DEPLOYMENT.md
    //     table validation
    //
    // We scan for `.surql` anchors and walk backwards to the nearest markdown
    // delimiter to capture the full token, including spaces or other characters.
    let mut script_names: Vec<&str> = Vec::new();
    let mut link_targets: Vec<&str> = Vec::new();
    {
        let mut remaining = section;
        while let Some(pos) = remaining.find(".surql") {
            let end = pos + ".surql".len();
            let before = &remaining[..pos];
            let boundary = before.rfind(['`', '[', ']', '(', ')', '\n', '\r']);
            let start = boundary.map_or(0, |i| i + 1);
            let boundary_char = boundary.map(|i| before.as_bytes()[i]);

            let candidate = &remaining[start..end];

            if boundary_char == Some(b'(') {
                // Inside a markdown link URL — this is a real path
                link_targets.push(candidate.trim());
            } else {
                // Display name (backtick-wrapped, bare, etc.) — for DEPLOYMENT.md check
                let filename = candidate.rsplit('/').next().unwrap_or(candidate).trim();
                if filename.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    script_names.push(filename);
                }
            }
            remaining = &remaining[end..];
        }
        script_names.sort_unstable();
        script_names.dedup();
        link_targets.sort_unstable();
        link_targets.dedup();
    }

    if script_names.is_empty() {
        return;
    }

    println!(
        "cargo:warning=Migration scripts referenced in CHANGELOG for v{version}: {}",
        script_names.join(", ")
    );

    // 1. Verify each link target exists on disk (these are the real file paths)
    for target in &link_targets {
        if !Path::new(target).exists() {
            let filename = target.rsplit('/').next().unwrap_or(target);
            errors.push(format!(
                "CHANGELOG for v{version} references migration script `{filename}` \
                 but the file `{target}` does not exist."
            ));
        }
    }
    // Also verify any bare filenames (not from link targets) exist
    for script in &script_names {
        let path = format!("database/migrations/{script}");
        let already_checked = link_targets
            .iter()
            .any(|t| t.rsplit('/').next().unwrap_or(t) == *script);
        if !already_checked && !Path::new(&path).exists() {
            errors.push(format!(
                "CHANGELOG for v{version} references migration script `{script}` \
                 but the file `{path}` does not exist."
            ));
        }
    }

    // 2. Verify DEPLOYMENT.md has a row for this version containing all scripts
    let base_version = version.split('-').next().unwrap_or(version);
    let row = deployment.lines().find(|line| {
        let trimmed = line.trim();
        trimmed.starts_with('|')
            && trimmed.split('|').nth(1).is_some_and(|cell| {
                let cell = cell.trim();
                cell == format!("v{base_version}") || cell == format!("v{version}")
            })
    });

    let Some(row) = row else {
        errors.push(format!(
            "DEPLOYMENT.md migration reference table is missing an entry for v{base_version}.\n\
             The CHANGELOG references migration scripts for this release: {}\n\
             Add a row to the Migration Reference table in DEPLOYMENT.md:\n\n  \
             | v{base_version} | {} | <notes> |",
            script_names.join(", "),
            script_names
                .iter()
                .map(|s| format!("`{s}`"))
                .collect::<Vec<_>>()
                .join(", "),
        ));
        return;
    };

    let missing: Vec<&str> = script_names
        .iter()
        .filter(|s| !row.contains(*s))
        .copied()
        .collect();

    if !missing.is_empty() {
        errors.push(format!(
            "DEPLOYMENT.md migration table row for v{base_version} is incomplete.\n\
             The following migration scripts are in the CHANGELOG but missing from the row:\n  \
             {}\n\
             Update the v{base_version} row in DEPLOYMENT.md to include all migration scripts.",
            missing.join("\n  "),
        ));
    }

    println!("cargo:warning=DEPLOYMENT.md migration table for v{base_version} is complete. ✓");
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
        if let Some(parent) = dst.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).unwrap();
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
