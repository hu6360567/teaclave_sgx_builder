use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::OpenOptions;
use std::hash::Hash;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;

use crate::BuildMetadata;
use crate::metadata::Mode;

const TMP_MAKE_DIR: &'static str = "_teaclave_sgx_builder_tmp_make";

const MAKE_FILE: &'static str = r#"
include ${_TEACLAVE_SGX_SDK_ROOT}/buildenv.mk

ifeq ($(shell getconf LONG_BIT), 32)
	SGX_ARCH := x86
else ifeq ($(findstring -m32, $(CXXFLAGS)), -m32)
	SGX_ARCH := x86
endif

ifeq ($(SGX_ARCH), x86)
	SGX_COMMON_CFLAGS := -m32
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib
	SGX_BIN_PATH := $(SGX_SDK)/bin/x86
else
	SGX_COMMON_CFLAGS := -m64
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib64
	SGX_BIN_PATH := $(SGX_SDK)/bin/x64
endif

ifeq ($(SGX_DEBUG), 1)
	SGX_COMMON_CFLAGS += -O0 -g
else
	SGX_COMMON_CFLAGS += -O2
endif

SGX_EDGER8R := $(SGX_BIN_PATH)/sgx_edger8r
ifneq ($(SGX_MODE), HYPER)
	SGX_ENCLAVE_SIGNER := $(SGX_BIN_PATH)/sgx_sign
else
	SGX_ENCLAVE_SIGNER := $(SGX_BIN_PATH)/sgx_sign_hyper
	SGX_EDGER8R_MODE := --sgx-mode $(SGX_MODE)
endif

TEACLAVE_EDL_PATH := $(_TEACLAVE_SGX_SDK_ROOT)/sgx_edl/edl
TEACLAVE_COMMON_PATH := $(_TEACLAVE_SGX_SDK_ROOT)/common

App_Include_Paths := -I$(SGX_SDK)/include -I$(TEACLAVE_COMMON_PATH)/inc -I$(TEACLAVE_EDL_PATH)
App_C_Flags := $(CFLAGS) $(SGX_COMMON_CFLAGS) -fPIC -Wno-attributes $(App_Include_Paths)



all:
	@$(foreach v, $(.VARIABLES), $(info $(v) = $($(v))))
	@echo ""
"#;


#[derive(Debug)]
pub struct BuildFlags {
    pub common_flags: HashMap<String, String>,
}

impl BuildFlags {
    fn sgx_mode_var(mode: &Mode) -> &OsStr {
        match mode.hardware {
            true => OsStr::new("HW"),
            false => OsStr::new("SIM")
        }
    }

    fn sgx_debug_var(mode: &Mode) -> &OsStr {
        match mode.debug {
            true => OsStr::new("1"),
            false => OsStr::new("0")
        }
    }
    fn common_flags(build_metadata: &BuildMetadata, out_dir: &Path) -> HashMap<String, String> {

        // create a folder called tmp_make
        let make_dir = PathBuf::from(&out_dir).join(TMP_MAKE_DIR);
        fs::create_dir_all(&make_dir).unwrap();

        let makeflile = make_dir.join("Makefile");

        OpenOptions::new()
            .create(true)
            .write(true)
            .open(&makeflile).expect("fail to open Makefile")
            .write_all(MAKE_FILE.as_bytes()).expect("fail to write Makefile");

        let make_output = Command::new("make")
            .current_dir(&make_dir)
            .env("_TEACLAVE_SGX_SDK_ROOT", build_metadata.teaclave_sgx_sdk.as_os_str())
            .env("SGX_SDK", build_metadata.intel_sgx_sdk.as_os_str())
            .env("SGX_MODE", Self::sgx_mode_var(&build_metadata.mode))
            .env("SGX_DEBUG", Self::sgx_debug_var(&build_metadata.mode))
            .output().expect("fail to execute make")
            .stdout;

        let make_output = String::from_utf8(make_output).expect("fail to parse make output");


        let re = Regex::new(r"(?<k>\S*) = (?<v>.*)").unwrap();

        re.captures_iter(make_output.trim())
            .map(|c| (c.name("k"), c.name("v")))
            .filter(|(k, v)| k.is_some() && v.is_some())
            .map(|(k, v)| (k.unwrap().as_str().to_string(), v.unwrap().as_str().to_string()))
            .collect::<HashMap<_, _>>()
    }
    pub fn new(build_metadata: &BuildMetadata, out_dir: &Path) -> Self {
        let common_flags = Self::common_flags(build_metadata, out_dir);

        Self { common_flags }
    }

    pub(crate) fn sgx_edger8r(&self) -> PathBuf {
        PathBuf::from(self.common_flags.get("SGX_EDGER8R").unwrap())
    }

    fn teaclave_edl_path(&self) -> PathBuf {
        PathBuf::from(self.common_flags.get("TEACLAVE_EDL_PATH").unwrap())
    }

    fn teaclave_common_path(&self) -> PathBuf {
        PathBuf::from(self.common_flags.get("TEACLAVE_COMMON_PATH").unwrap())
    }

    pub(crate) fn teaclave_edl_search_path(&self) -> Vec<PathBuf> {
        vec![
            self.teaclave_edl_path(),
            self.teaclave_common_path().join("inc"),
        ]
    }

    pub(crate) fn app_c_flag(&self) -> &str {
        self.common_flags.get("App_C_Flags").unwrap()
    }
    pub(crate) fn sgx_mode(&self) -> &str {
        self.common_flags.get("SGX_MODE").unwrap()
    }
}

