use std::collections::HashMap;
use std::fmt::Debug;
use cargo_metadata::MetadataCommand;
use metadata::SGX_METADATA_KEY;
use crate::flags::common_flags;
use crate::metadata::{EDLMetadata, IntelSGXSDKMetadata, OptionBuildMetadata, TeaclaveSGXSDKMetadata};

pub mod flags;
pub mod metadata;

pub mod teaclave;

pub mod intel;

enum HostArch {
    X86,
    X86_64,
}

impl Default for HostArch {
    fn default() -> Self {
        let arch = std::env::consts::ARCH;
        match arch {
            "x86" => HostArch::X86,
            "x86_64" => HostArch::X86_64,
            _ => panic!("unsupported arch"),
        }
    }
}

pub fn message<T: Debug>(message: &T) {
    println!("cargo:warning={:?}", message);
}

#[derive(Debug)]
pub struct BuildMetadata {
    pub intel_sgx_sdk: IntelSGXSDKMetadata,
    pub teaclave_sgx_sdk: TeaclaveSGXSDKMetadata,
    pub edl: EDLMetadata,
}

impl BuildMetadata {
    pub fn common_flags(&self) -> HashMap<String, String> {
        common_flags(&self.teaclave_sgx_sdk)
    }
}

pub fn parse() -> BuildMetadata {
    let cargo_metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .unwrap();

    let workspace_metadata = cargo_metadata.workspace_metadata.as_object().expect("fail to parse workspace metadata as object").get(SGX_METADATA_KEY);

    let workspace_build_metadata = OptionBuildMetadata::from_cargo_metadata(workspace_metadata, cargo_metadata.workspace_root.as_std_path());


    let package_name = std::env::var("CARGO_PKG_NAME").expect("fail to get CARGO_PACKAGE_NAME");
    let package = cargo_metadata.workspace_packages().into_iter().filter(|p| p.name == package_name).next().expect("current package not found");
    let package_path = {
        let mut manifest = package.manifest_path.canonicalize().expect("fail to canonicalize package manifest path");
        manifest.pop();
        manifest
    };


    let package_metadata = package.metadata.as_object().expect("fail to parse current package metadata as object").get(SGX_METADATA_KEY);
    let package_build_metadata = OptionBuildMetadata::from_cargo_metadata(package_metadata, package_path.as_path());

    OptionBuildMetadata::extract(package_build_metadata, workspace_build_metadata).build()
}
