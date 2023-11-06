use std::ops::Deref;
use std::path::{Path, PathBuf};
use serde::Deserialize;
use crate::{Arch, metadata};
use crate::metadata::Metadata;

pub mod edl;
#[derive(Debug, Deserialize)]
pub struct IntelSGXSDKMetadata {
    path: PathBuf,
}

impl Default for IntelSGXSDKMetadata {
    fn default() -> Self {
        Self { path: PathBuf::from("/opt/intel/sgxsdk") }
    }
}

impl Deref for IntelSGXSDKMetadata {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl Metadata for IntelSGXSDKMetadata {
    fn canonicalize(&mut self, base: &Path) {
        self.path = metadata::canonicalize_from_base(&self.path, base).expect("fail to canonicalize intel sgx sdk path");
    }

    fn set_cargo_instruction(&self) {
        println!("cargo:return-if-changed={}", self.as_os_str().to_str().unwrap())
    }
}

impl IntelSGXSDKMetadata {
    pub fn bin_path(&self) -> PathBuf {
        match Arch::default() {
            Arch::X86 => self.join("bin").join("x86"),
            Arch::X86_64 => self.join("bin").join("x64"),
        }
    }
}
