use std::collections::HashMap;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use serde::Deserialize;
use serde_json::Value;
use crate::BuildMetadata;
use crate::intel::edl::EDLMetadata;
use crate::intel::IntelSGXSDKMetadata;
use crate::teaclave::TeaclaveSGXSDKMetadata;

pub(crate) const SGX_METADATA_KEY: &'static str = "sgx";

pub trait Metadata {
    fn canonicalize(&mut self, base: &Path);

    fn set_cargo_instruction(&self);
}

pub(crate) fn canonicalize_from_base(p: &PathBuf, base: &Path) -> std::io::Result<PathBuf> {
    if p.is_absolute() { std::fs::canonicalize(p.clone()) } else {
        std::fs::canonicalize(base.join(p))
    }
}


#[derive(Debug, Deserialize, Clone)]
pub struct Mode {
    pub(crate) hardware: bool,
    pub(crate) debug: bool,
}

impl Mode {
    fn default(profile: &str) -> Self {
        match profile {
            "dev" | "debug" => Self::default_dev(),
            "release" => Self::default_release(),
            "test" => Self::default_test(),
            "bench" => Self::default_bench(),
            _ => {
                panic!("no default mode");
            }
        }
    }
    fn default_dev() -> Self {
        Self {
            hardware: false,
            debug: true,
        }
    }

    fn default_release() -> Self {
        Self {
            hardware: true,
            debug: false,
        }
    }

    fn default_test() -> Self {
        Self::default_dev()
    }

    fn default_bench() -> Self {
        Self::default_release()
    }
}

#[derive(Debug, Deserialize)]
pub struct ModeMetadata(HashMap<String, Mode>);

impl Deref for ModeMetadata {
    type Target = HashMap<String, Mode>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildSystem {
    NoStd,
    Std,
    Xargo,
}


#[derive(Default, Debug, Deserialize)]
pub struct PrepareBuildMetadata {
    pub intel_sgx_sdk: Option<IntelSGXSDKMetadata>,
    pub teaclave_sgx_sdk: Option<TeaclaveSGXSDKMetadata>,
    pub edl: Option<EDLMetadata>,
    pub mode: Option<ModeMetadata>,
}


impl PrepareBuildMetadata {
    pub(crate) fn from_cargo_metadata(value: Option<&Value>, base: &Path) -> PrepareBuildMetadata {
        let mut obm = match value {
            None => PrepareBuildMetadata::default(),
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

        if let Some(mode) = second.mode {
            first.mode.get_or_insert(mode);
        }

        first
    }

    pub(crate) fn build(self) -> BuildMetadata {
        let intel_sgx_sdk = self.intel_sgx_sdk.unwrap_or_else(IntelSGXSDKMetadata::default);
        intel_sgx_sdk.set_cargo_instruction();

        let teaclave_sgx_sdk = self.teaclave_sgx_sdk.expect("teaclave not set in metadata");
        teaclave_sgx_sdk.set_cargo_instruction();

        let edl = self.edl.expect("edl not set in metadata");
        edl.set_cargo_instruction();

        // get profile and its build mode
        let build_profile = std::env::var("PROFILE").unwrap();
        let build_profile = if build_profile == "debug" { "dev".to_string() } else { build_profile };

        let mode = if let Some(mode) = self.mode.and_then(|ref m| m.get(&build_profile).map(|m| m.clone())) {
            mode
        } else {
            Mode::default(&build_profile)
        };


        BuildMetadata {
            intel_sgx_sdk,
            teaclave_sgx_sdk,
            edl,
            mode,
        }
    }
}