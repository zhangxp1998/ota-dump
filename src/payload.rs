use binread::BinRead;
use prost::{DecodeError, Message};

pub mod update_engine {

    pub mod payload_formatter {
        use serde::{Deserializer, Serializer};

        pub fn serialize<S>(data: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match data {
                Some(data) => serializer.serialize_str(hex::encode_upper(data).as_str()),
                None => serializer.serialize_str("\"\""),
            }
        }
        pub fn deserialize<'a, D>(_: D) -> Result<Option<Vec<u8>>, D::Error>
        where
            D: Deserializer<'a>,
        {
            todo!("Deserializing Vec<u8> from hex string is not implemented")
        }
    }
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
