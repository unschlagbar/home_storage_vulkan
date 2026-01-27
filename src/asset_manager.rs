#![allow(unused)]

use std::path::Path;
use std::path::PathBuf;

/// This is responsible for ressource loading
pub struct AssetMananger {}

impl AssetMananger {
    pub fn path(path: &str) -> PathBuf {
        #[cfg(feature = "release")]
        return Path::new(path).to_path_buf();

        #[cfg(not(feature = "release"))]
        return Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    }
}
