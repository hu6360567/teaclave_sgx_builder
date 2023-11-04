use std::ffi::{OsStr, OsString};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use serde::Deserialize;
use serde_json::Value;
use crate::{BuildMetadata, HostArch};
use crate::teaclave::common_edl_serach_path;

pub(crate) const SGX_METADATA_KEY: &'static str = "sgx";

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

impl IntelSGXSDKMetadata {
    fn canonicalize(&mut self, base: &Path) {
        self.path = canonicalize_from_base(&self.path, base).expect("fail to canonicalize intel sgx sdk path");
    }

    pub fn bin_path(&self) -> PathBuf {
        match HostArch::default() {
            HostArch::X86 => self.join("bin").join("x86"),
            HostArch::X86_64 => self.join("bin").join("x64"),
        }
    }
}

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

impl TeaclaveSGXSDKMetadata {
    fn canonicalize(&mut self, base: &Path) {
        self.path = canonicalize_from_base(&self.path, base).expect("fail to canonicalize teaclave sgx sdk path");}
}

#[derive(Debug, Deserialize)]
pub struct EDLMetadata {
    pub path: PathBuf,
    pub search_paths: Vec<PathBuf>,
}

impl EDLMetadata {
    fn edl_file_name(&self) -> &OsStr {
        self.path.as_path().file_name().unwrap()
    }
    fn file_name_suffix_of(&self, suffix: &str) -> OsString {
        let mut filename = self.edl_file_name().to_os_string();
        filename.push(suffix);

        filename
    }

    pub fn trusted_header(&self) -> OsString {
        self.file_name_suffix_of("_t.h")
    }

    pub fn trusted_source(&self) -> OsString {
        self.file_name_suffix_of("_t.c")
    }

    pub fn untrusted_header(&self) -> OsString {
        self.file_name_suffix_of("_u.h")
    }

    pub fn untrusted_source(&self) -> OsString {
        self.file_name_suffix_of("_u.c")
    }
}

fn canonicalize_from_base(p: &PathBuf, base: &Path) -> std::io::Result<PathBuf> {
    if p.is_absolute() { std::fs::canonicalize(p.clone()) } else {
        std::fs::canonicalize(base.join(p))
    }
}

impl EDLMetadata {
    fn canonicalize(&mut self, base: &Path) {
        self.path = canonicalize_from_base(&self.path, base).expect("fail to canonicalize edl path");
        self.search_paths = self.search_paths.iter().map(|sp| canonicalize_from_base(sp, base).expect("fail to canonicalize search path")).collect();
    }
}


#[derive(Default, Debug, Deserialize)]
pub struct OptionBuildMetadata {
    pub intel_sgx_sdk: Option<IntelSGXSDKMetadata>,
    pub teaclave_sgx_sdk: Option<TeaclaveSGXSDKMetadata>,
    pub edl: Option<EDLMetadata>,
}


impl OptionBuildMetadata {
    pub(crate) fn from_cargo_metadata(value: Option<&Value>, base: &Path) -> OptionBuildMetadata {
        let mut obm = match value {
            None => OptionBuildMetadata::default(),
            Some(value) => serde_json::from_value(value.clone()).expect("fail to parse metadata")
        };

        obm.canonicalize(base);

        obm
    }

    fn canonicalize(&mut self, base: &Path) {
        if let Some(intel_sgx_sdk) = &mut self.intel_sgx_sdk {
            intel_sgx_sdk.canonicalize(base)
        }

        if let Some(teaclave_sgx_sdk) = &mut self.teaclave_sgx_sdk {
            teaclave_sgx_sdk.canonicalize(base)
        }

        if let Some(edl) = &mut self.edl {
            edl.canonicalize(base)
        }
    }


    pub(crate) fn extract(mut first: Self, second: Self) -> Self {
        if let Some(sdk) = second.intel_sgx_sdk {
            first.intel_sgx_sdk.get_or_insert(sdk);
        }

        if let Some(sdk) = second.teaclave_sgx_sdk {
            first.teaclave_sgx_sdk.get_or_insert(sdk);
        }

        if let Some(edl) = second.edl {
            first.edl.get_or_insert(edl);
        }

        first
    }

    pub(crate) fn build(self) -> BuildMetadata {
        let intel_sgx_sdk = self.intel_sgx_sdk.unwrap_or_else(IntelSGXSDKMetadata::default);
        let teaclave_sgx_sdk = self.teaclave_sgx_sdk.expect("teaclave not set in metadata");
        let mut edl = self.edl.expect("edl not set in metadata");

        // add common search path into edl
        edl.search_paths.append(&mut common_edl_serach_path(&teaclave_sgx_sdk));


        BuildMetadata {
            intel_sgx_sdk,
            teaclave_sgx_sdk,
            edl,
        }
    }
}