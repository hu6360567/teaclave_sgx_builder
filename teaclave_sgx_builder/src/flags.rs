use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use regex::Regex;
use crate::message;
use crate::metadata::TeaclaveSGXSDKMetadata;

const MAKE_FILE: &'static str = r#"
include ${TEACLAVE_ROOT}/buildenv.mk

all:
	@$(foreach v, $(.VARIABLES), $(info $(v) = $($(v))))
	@echo ""
"#;

pub(crate) fn common_flags(teaclave: &TeaclaveSGXSDKMetadata) -> HashMap<String, String> {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // create a folder called tmp_make
    let make_dir = PathBuf::from(&out_dir).join("tmp_make");
    fs::create_dir_all(&make_dir).unwrap();

    let makeflile = make_dir.join("Makefile");

    OpenOptions::new()
        .create(true)
        .write(true)
        .open(&makeflile).expect("fail to open Makefile")
        .write_all(MAKE_FILE.as_bytes()).expect("fail to write Makefile");

    let make_output = Command::new("make")
        .current_dir(&make_dir)
        .env("TEACLAVE_ROOT", teaclave.as_os_str())
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