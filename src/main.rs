use binread::io::Cursor;
use binread::BinRead;
use std::fs;

mod payload;
mod zipfile;

pub mod update_engine {
    include!(concat!(env!("OUT_DIR"), "/chromeos_update_engine.rs"));
}

fn dump_payload(payload: &[u8]) {
    let mut reader = Cursor::new(payload);
    let payload = payload::UpdateEnginePayload::read(&mut reader).unwrap();
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
    let file = fs::File::open(&path).unwrap();
    let mmap = unsafe { memmap::Mmap::map(&file).unwrap() };
    let data = mmap.as_ref();

    if path.to_str().map_or(false, |f| f.ends_with(".zip")) {
        let ziparchive = zipfile::ZipArchive::new(data);
        let records = ziparchive.get_zip_entries().unwrap();
        for entry in records {
            if entry.get_filename() == "payload.bin" {
                assert!(!entry.is_compressed());
                assert_eq!(entry.get_compressed_size(), entry.get_uncompressed_size());
                let payload = ziparchive.get_compressed_data(&entry);
                dump_payload(payload);
            }
        }
    } else {
        dump_payload(data);
    }
    return Ok(());
}
