use std::path::{Path, PathBuf};
use std::ffi::{OsStr, OsString};
use std::process::Command;
use serde::Deserialize;
use crate::{BuildContext, message, metadata};
use crate::metadata::Metadata;

enum OutputDir {
    Trusted(PathBuf),
    Untrusted(PathBuf),
}

pub struct SgxEdger8rCommand<'a> {
    build_context: &'a BuildContext,
    output_dir: OutputDir,
}

impl<'a> SgxEdger8rCommand<'a> {
    pub(crate) fn trusted(build_context: &'a BuildContext, output_dir: PathBuf) -> Self {
        let output_dir = OutputDir::Trusted(output_dir);
        Self { build_context, output_dir }
    }

    pub(crate) fn untrusted(build_context: &'a BuildContext, output_dir: PathBuf) -> Self {
        let output_dir = OutputDir::Untrusted(output_dir);
        Self { build_context, output_dir }
    }

    fn generate_command_args(&self) -> Vec<String> {
        let mut args = vec![];

        let edl = &self.build_context.metadata.edl;

        // edl search_path from teaclave sdk
        self.build_context.flags.teaclave_edl_search_path().iter()
            .chain(edl.search_paths.iter())
            .for_each(|search_path| {
                args.push("--search-path".to_string());
                args.push(search_path.to_str().unwrap().to_string())
            });


        match &self.output_dir {
            OutputDir::Trusted(output_dir) => {
                args.push("--trusted".to_string());
                args.push(edl.path.to_str().unwrap().to_string());
                args.push("--trusted-dir".to_string());
                args.push(output_dir.to_str().unwrap().to_string())
            }
            OutputDir::Untrusted(output_dir) => {
                args.push("--untrusted".to_string());
                args.push(edl.path.to_str().unwrap().to_string());
                args.push("--untrusted-dir".to_string());
                args.push(output_dir.to_str().unwrap().to_string())
            }
        }

        args
    }

    pub(crate) fn run(self) {
        let bin = self.build_context.flags.sgx_edger8r();

        let args = self.generate_command_args();

        Command::new(bin)
            .args(args)
            .output().unwrap();
    }
}

#[derive(Debug, Deserialize)]
pub struct EDLMetadata {
    pub path: PathBuf,
    pub search_paths: Vec<PathBuf>,
}

impl EDLMetadata {
    pub fn edl_file_name(&self) -> &OsStr {
        self.path.as_path().file_prefix().unwrap()
    }
    fn file_name_suffix_of(&self, suffix: &str) -> OsString {
        let mut filename = self.edl_file_name().to_os_string();
        filename.push(suffix);

        filename
    }

    pub(crate) fn untrusted_name(&self) -> OsString {
        self.file_name_suffix_of("_u")
    }

    pub(crate) fn trusted_name(&self) -> OsString {
        self.file_name_suffix_of("_t")
    }

    pub(crate) fn trusted_header(&self) -> OsString {
        self.file_name_suffix_of("_t.h")
    }

    pub(crate) fn trusted_source(&self) -> OsString {
        self.file_name_suffix_of("_t.c")
    }

    pub(crate) fn untrusted_header(&self) -> OsString {
        self.file_name_suffix_of("_u.h")
    }

    pub(crate) fn untrusted_source(&self) -> OsString {
        self.file_name_suffix_of("_u.c")
    }
}


impl Metadata for EDLMetadata {
    fn canonicalize(&mut self, base: &Path) {
        self.path = metadata::canonicalize_from_base(&self.path, base).expect("fail to canonicalize edl path");
        self.search_paths = self.search_paths.iter().map(|sp| metadata::canonicalize_from_base(sp, base).expect("fail to canonicalize search path")).collect();
    }

    fn set_cargo_instruction(&self) {
        self.search_paths.iter()
            .chain([&self.path])
            .map(|p| p.to_str().unwrap())
            .for_each(|p| println!("cargo:return-if-changed={}", p))
    }
}
