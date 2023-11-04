use std::path::PathBuf;
use crate::metadata::TeaclaveSGXSDKMetadata;

pub(crate) fn common_edl_serach_path(metadata: &TeaclaveSGXSDKMetadata) -> Vec<PathBuf> {
    ["common/inc", "sgx_edl/edl"].into_iter().map(|suffix| metadata.join(suffix).canonicalize().unwrap()).collect()
}