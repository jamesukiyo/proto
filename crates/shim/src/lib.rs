#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::*;
#[cfg(windows)]
pub use windows::*;

use std::env;
use std::path::PathBuf;

pub const SHIM_VERSION: u8 = 18;

pub fn locate_proto_exe(bin: &str) -> Option<PathBuf> {
    let bin = get_exe_file_name(bin);
    let mut lookup_dirs = vec![];

    // When in development, ensure we're using the target built proto,
    // and not the proto available globally on `PATH`.
    #[cfg(any(debug_assertions, test))]
    {
        if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
            lookup_dirs.push(PathBuf::from(dir).join("debug"));
        }

        if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
            lookup_dirs.push(
                PathBuf::from(if let Some(index) = dir.find("crates") {
                    &dir[0..index]
                } else {
                    &dir
                })
                .join("target")
                .join("debug"),
            );
        }

        if let Ok(dir) = env::var("GITHUB_WORKSPACE") {
            lookup_dirs.push(PathBuf::from(dir).join("target").join("debug"));
        }

        if let Ok(dir) = env::current_dir() {
            lookup_dirs.push(dir.join("target").join("debug"));
        }
    }

    if let Ok(dir) = env::var("PROTO_HOME") {
        let dir = PathBuf::from(dir);

        if let Ok(version) = env::var("PROTO_VERSION") {
            lookup_dirs.push(dir.join("tools").join("proto").join(version));
        }

        lookup_dirs.push(dir.join("bin"));
    }

    if let Ok(dir) = env::var("PROTO_LOOKUP_DIR") {
        lookup_dirs.push(dir.into());
    }

    // Detect the currently running executable (proto), and then find
    // a proto-shim sibling in the same directory. This assumes both
    // binaries are the same version.
    if let Ok(current) = env::current_exe() {
        if let Some(dir) = current.parent() {
            lookup_dirs.push(dir.to_path_buf());
        }
    }

    // Special case for unit tests and other isolations where
    // PROTO_HOME is set to something random, but the proto
    // binaries still exist in their original location.
    if let Some(dir) = dirs::home_dir() {
        if let Ok(version) = env::var("PROTO_VERSION") {
            lookup_dirs.push(dir.join(".proto").join("tools").join("proto").join(version));
        }

        lookup_dirs.push(dir.join(".proto").join("bin"));
    }

    for lookup_dir in lookup_dirs {
        let file = lookup_dir.join(&bin);

        if file.is_absolute() && file.exists() {
            return Some(file);
        }
    }

    None
}
