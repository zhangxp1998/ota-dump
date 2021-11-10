use binread::BinRead;
use std::{
    fs::{self},
    io::{Read, Seek},
};

mod payload;
mod zipfile;

fn dump_payload<File: Read + Seek>(payload_file: &mut File) {
    let payload = payload::UpdateEnginePayload::read(payload_file)
        .expect("Failed to parse update_engine payload");
    assert_eq!(payload.version, 2);
    let manifest = payload.get_manifest().unwrap();
    println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
}

fn main() -> Result<(), i32> {
    let args: Vec<String> = std::env::args().collect();
    if std::env::args().len() != 2 {
        println!("Usage: {} <path to payload.bin or OTA.zip>", args[0]);
        return Err(1);
    }
    let path = &args[1];
    let path = std::path::Path::new(path);
    if !std::path::Path::exists(path) {
        println!("{} does not exists", path.display());
        return Err(2);
    }
    let mut file = fs::File::open(&path).unwrap();

    if path.to_str().map_or(false, |f| f.ends_with(".zip")) {
        let mut ziparchive = zipfile::ZipArchive::new(file).expect("Failed to open zip archive");
        for entry in ziparchive.into_iter() {
            if entry.get_filename() == "payload.bin" {
                assert!(!entry.is_compressed());
                assert_eq!(entry.get_compressed_size(), entry.get_uncompressed_size());
                let payload = ziparchive.get_compressed_data_file(&entry).unwrap();
                dump_payload(payload);
                return Ok(());
            }
        }
    } else {
        dump_payload(&mut file);
    }
    return Ok(());
}
