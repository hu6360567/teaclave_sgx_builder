use std::fmt::{Debug, Display};

use teaclave_sgx_builder::parse;

fn message<T: Debug>(message: &T) {
    println!("cargo:warning={:?}", message);
}


fn main() {
    let build_metadata = parse();

    message(&build_metadata);

    let common_flags = build_metadata.common_flags();
    message(&common_flags);

    message(&serde_json::to_string(&common_flags).unwrap());
}