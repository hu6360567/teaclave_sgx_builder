use std::path::{Path, PathBuf};
use std::ops::Deref;
use serde::Deserialize;
use crate::metadata;
use crate::metadata::Metadata;

#[derive(Debug, Deserialize)]
pub struct TeaclaveSGXSDKMetadata {
    path: PathBuf,
}

impl Deref for TeaclaveSGXSDKMetadata {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl Metadata for TeaclaveSGXSDKMetadata {
    fn canonicalize(&mut self, base: &Path) {
        self.path = metadata::canonicalize_from_base(&self.path, base).expect("fail to canonicalize teaclave sgx sdk path");
    }

    fn set_cargo_instruction(&self) {
        println!("cargo:return-if-changed={}", self.as_os_str().to_str().unwrap())
    }
}


impl TeaclaveSGXSDKMetadata {
    fn common_path(&self) -> PathBuf {
        self.join("common")
    }

    fn edl_path(&self) -> PathBuf {
        self.join("sgx_edl").join("edl")
    }
    pub(crate) fn common_search_path(&self) -> Vec<PathBuf> {
        vec![self.common_path().join("inc"), self.edl_path()]
    }
}