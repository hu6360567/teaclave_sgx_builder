#![feature(path_file_prefix)]

use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::PathBuf;
use std::process::Command;

use cargo_metadata::MetadataCommand;

use intel::edl::EDLMetadata;
use intel::IntelSGXSDKMetadata;
use metadata::SGX_METADATA_KEY;
use teaclave::TeaclaveSGXSDKMetadata;

use crate::flags::BuildFlags;
use crate::intel::edl::SgxEdger8rCommand;
use crate::metadata::{Mode, PrepareBuildMetadata};

pub mod flags;
pub mod metadata;

pub mod teaclave;

pub mod intel;

enum Arch {
    X86,
    X86_64,
}

impl Default for Arch {
    fn default() -> Self {
        let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| std::env::consts::ARCH.to_string());
        match target_arch.as_str() {
            "x86" => Arch::X86,
            "x86_64" => Arch::X86_64,
            _ => panic!("unsupported arch"),
        }
    }
}

pub fn message<T: Debug + ?Sized>(message: &T) {
    println!("cargo:warning={:?}", message);
}

#[derive(Debug)]
pub struct BuildMetadata {
    pub intel_sgx_sdk: IntelSGXSDKMetadata,
    pub teaclave_sgx_sdk: TeaclaveSGXSDKMetadata,
    pub edl: EDLMetadata,
    pub mode: Mode,
}

pub fn parse() -> BuildContext {
    let cargo_metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .unwrap();

    let workspace_metadata = cargo_metadata.workspace_metadata.as_object().expect("fail to parse workspace metadata as object").get(SGX_METADATA_KEY);

    let workspace_build_metadata = PrepareBuildMetadata::from_cargo_metadata(workspace_metadata, cargo_metadata.workspace_root.as_std_path());


    let package_name = std::env::var("CARGO_PKG_NAME").expect("fail to get CARGO_PACKAGE_NAME");
    let package = cargo_metadata.workspace_packages().into_iter().filter(|p| p.name == package_name).next().expect("current package not found");
    let package_path = {
        let mut manifest = package.manifest_path.canonicalize().expect("fail to canonicalize package manifest path");
        manifest.pop();
        manifest
    };


    let package_metadata = package.metadata.as_object().expect("fail to parse current package metadata as object").get(SGX_METADATA_KEY);
    let package_build_metadata = PrepareBuildMetadata::from_cargo_metadata(package_metadata, package_path.as_path());

    PrepareBuildMetadata::extract(package_build_metadata, workspace_build_metadata).build().into()
}


#[derive(Debug)]
pub struct BuildContext {
    metadata: BuildMetadata,
    flags: BuildFlags,
    out_dir: PathBuf,
}

impl From<BuildMetadata> for BuildContext {
    fn from(metadata: BuildMetadata) -> Self {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let out_dir = PathBuf::from(out_dir).canonicalize().unwrap();

        let flags = BuildFlags::new(&metadata, out_dir.as_path());

        Self { metadata, flags, out_dir }
    }
}

const EDL_OUTPUT_PATH: &'static str = "_teaclave_sgx_builder_edl_output";

impl BuildContext {
    fn edl_output_path(&self) -> PathBuf {
        self.out_dir.join(EDL_OUTPUT_PATH)
    }
    fn generate_untrusted_edl(&self) {
        let output_path = self.edl_output_path();

        message(&format!("generating edl codes to {}", output_path.as_os_str().to_str().unwrap()));

        let command_context = SgxEdger8rCommand::untrusted(&self, output_path);

        command_context.run();
    }
    fn compile_untrusted_edl(&self) {
        let output_path = self.edl_output_path();
        let untrusted_lib_name = self.metadata.edl.untrusted_name();


        let untrusted_source = output_path.join(self.metadata.edl.untrusted_source()).canonicalize().unwrap();


        // let untrusted_object = output_path.join(format!("{}.o", untrusted_lib_name.to_str().unwrap()));
        let untrusted_static = output_path.join(format!("lib{}.a", untrusted_lib_name.to_str().unwrap()));

        message(&format!("compiling to {}", untrusted_static.as_os_str().to_str().unwrap()));


        self.flags.app_c_flag().split(" ")
            .filter(|&flag| !flag.is_empty())
            .fold(cc::Build::new(), |mut build, flag| {
                build.flag(flag);
                build
            })
            // .ar_flag("rcsD") //FIXME:cannot set ar option with cc
            .include(&output_path)
            .file(&untrusted_source)
            .out_dir(&output_path)
            .compile(untrusted_lib_name.to_str().unwrap());

        println!("cargo:rustc-link-search=native={}", output_path.to_str().unwrap());
        println!("cargo:rustc-link-lib=static={}", untrusted_lib_name.to_str().unwrap());
    }

    fn set_untrusted_link(&self) {
        let intel_sdk_lib = self.metadata.intel_sgx_sdk.join("lib64").canonicalize().unwrap();
        let sgx_mode = self.flags.sgx_mode();

        println!("cargo:rustc-link-search=native={}", intel_sdk_lib.to_str().unwrap());
        match sgx_mode {
            "SIM" | "SW" => println!("cargo:rustc-link-lib=dylib=sgx_urts_sim"),
            "HYPER" => println!("cargo:rustc-link-lib=dylib=sgx_urts_hyper"),
            "HW" => println!("cargo:rustc-link-lib=dylib=sgx_urts"),
            _ => println!("cargo:rustc-link-lib=dylib=sgx_urts"),
        }
    }

    pub fn compile_untrusted(&self) {
        self.generate_untrusted_edl();
        self.compile_untrusted_edl();
        self.set_untrusted_link();
    }
}