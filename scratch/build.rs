use std::fmt::{Debug, Display};

use teaclave_sgx_builder::{BuildContext, parse};
use teaclave_sgx_builder::metadata::BuildSystem;

fn message<T: Debug>(message: &T) {
    println!("cargo:warning={:?}", message);
}


fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    message(&std::env::var("PROFILE"));

    let build_ctx = parse();

    message(&build_ctx);

    build_ctx.compile_untrusted();
    //
    // let common_flags = build_metadata.common_flags();
    // message(&common_flags);
    //
    // message(&serde_json::to_string(&common_flags).unwrap());
}