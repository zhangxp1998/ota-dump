use binread::BinRead;
use prost::{DecodeError, Message};

pub mod update_engine {
    include!("../target/protobuf_out/chromeos_update_engine.rs");
}

#[derive(BinRead)]
#[br(magic = b"CrAU", big)]
#[derive(Debug)]
pub struct UpdateEnginePayload {
    pub version: u64,
    manifest_size: u64,
    metadata_signature_size: u32,
    #[br(count=manifest_size)]
    manifest: Vec<u8>,
}

impl UpdateEnginePayload {
    pub fn get_manifest(&self) -> Result<update_engine::DeltaArchiveManifest, DecodeError> {
        return update_engine::DeltaArchiveManifest::decode(&self.manifest[..]);
    }
}
