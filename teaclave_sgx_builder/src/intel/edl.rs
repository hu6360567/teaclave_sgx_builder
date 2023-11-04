use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::Command;
use crate::HostArch;
use crate::metadata::{EDLMetadata, IntelSGXSDKMetadata};


pub struct SgxEdger8r {
    bin: PathBuf,
}

impl From<&IntelSGXSDKMetadata> for SgxEdger8r {
    fn from(value: &IntelSGXSDKMetadata) -> Self {
        Self { bin: value.bin_path().join("sgx_edger8r").canonicalize().expect("fail to canonicalize sgx_edger8r") }
    }
}

enum OutputDir {
    Trusted(PathBuf),
    Untrusted(PathBuf),
}

struct SgxEdger8rCommandContext {
    edl: EDLMetadata,
    output_dir: OutputDir,
}

impl SgxEdger8rCommandContext {
    fn generate_command_args(&self) -> Vec<String> {
        let mut args = vec![];

        for search_path in &self.edl.search_paths {
            args.push("--search-path".to_string());
            args.push(search_path.to_str().unwrap().to_string())
        }

        match &self.output_dir {
            OutputDir::Trusted(output_dir) => {
                args.push("--trusted".to_string());
                args.push(self.edl.path.to_str().unwrap().to_string());
                args.push("--trusted-dir".to_string());
                args.push(output_dir.to_str().unwrap().to_string())
            }
            OutputDir::Untrusted(output_dir) => {
                args.push("--untrusted".to_string());
                args.push(self.edl.path.to_str().unwrap().to_string());
                args.push("--untrusted-dir".to_string());
                args.push(output_dir.to_str().unwrap().to_string())
            }
        }

        args
    }
    pub fn generate_files(&self, sgx_edger8r: &SgxEdger8r) {
        Command::new(sgx_edger8r.bin.as_os_str())
            .args(self.generate_command_args())
            .output()
            .expect("fail to generate run sgx_edger8r");
    }

    fn generated_edl_source(&self) -> PathBuf {
        match &self.output_dir {
            OutputDir::Trusted(output_dir) =>
                output_dir.join(self.edl.trusted_source()),
            OutputDir::Untrusted(output_dir) =>
                output_dir.join(self.edl.untrusted_source())
        }
    }
}
