use binread::BinRead;
use std::{
    cmp::min,
    fs::{self},
    io::{BufReader, Read, Seek, SeekFrom},
};

mod payload;
mod zipfile;

pub struct HttpFile {
    url: String,
    client: reqwest::blocking::Client,
    pos: u64,
    file_size: u64,
}

impl Seek for HttpFile {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        match pos {
            SeekFrom::Start(s) => self.pos = s,
            SeekFrom::End(s) => {
                if self.file_size == 0 {
                    panic!("Seek from file end not supported if file size isn't known");
                }
                self.pos = (self.file_size as i64 + s) as u64;
            }
            SeekFrom::Current(s) => self.pos = (self.pos as i64 + s) as u64,
        }
        Ok(self.pos)
    }
}

impl Read for HttpFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let cap = min(self.file_size - 1, self.pos + buf.len() as u64 - 1);
        let read_size = cap - self.pos + 1;
        // println!("Reading {}-{}, len: {}", self.pos, cap, buf.len());
        let mut resp = self
            .client
            .get(self.url.as_str())
            .header("Range", format!("bytes={}-{}", self.pos, cap))
            .send()
            .unwrap();
        resp.read_exact(&mut buf[..read_size as usize])?;
        self.pos += read_size;
        Ok(buf.len())
    }
}

fn dump_payload<File: Read + Seek>(payload_file: &mut File, show_ops: bool) {
    let payload = payload::UpdateEnginePayload::read(payload_file)
        .expect("Failed to parse update_engine payload");
    assert_eq!(payload.version, 2);
    let mut manifest: payload::update_engine::DeltaArchiveManifest =
        payload.get_manifest().unwrap();
    if !show_ops {
        for part in manifest.partitions.iter_mut() {
            part.operations.clear();
            part.merge_operations.clear();
        }
    }
    println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
}

fn dump_payload_zip<File: Read + Seek>(payload_file: &mut File, show_ops: bool) {
    let mut ziparchive =
        zipfile::ZipArchive::new(payload_file).expect("Failed to open zip archive");
    for entry in ziparchive.into_iter() {
        if entry.get_filename() == "payload.bin" {
            assert!(!entry.is_compressed());
            assert_eq!(entry.get_compressed_size(), entry.get_uncompressed_size());
            let payload = ziparchive.get_compressed_data_file(&entry).unwrap();
            dump_payload(payload, show_ops);
            return;
        }
    }
}

fn main() -> Result<(), i32> {
    let args: Vec<String> = std::env::args().collect();
    if std::env::args().len() != 2 && std::env::args().len() != 3 {
        println!(
            "Usage: {} [-l show operations] <path to payload.bin or OTA.zip>",
            args[0]
        );
        return Err(1);
    }
    let show_ops = args[1] == "-l";
    let path = &args[args.len() - 1];
    if path.starts_with("http://") || path.starts_with("https://") {
        let client = reqwest::blocking::Client::new();
        let resp = client.get(path).send().unwrap();
        let content_length = resp.content_length().unwrap();
        dump_payload_zip(
            &mut BufReader::with_capacity(
                1024 * 64,
                HttpFile {
                    url: path.clone(),
                    client: client,
                    file_size: content_length,
                    pos: 0,
                },
            ),
            show_ops,
        );
        return Ok(());
    }
    let path = std::path::Path::new(path);
    if !std::path::Path::exists(path) {
        println!("{} does not exists", path.display());
        return Err(2);
    }
    let mut file = BufReader::with_capacity(1024 * 64, fs::File::open(&path).unwrap());

    if path.to_str().map_or(false, |f| f.ends_with(".zip")) {
        dump_payload_zip(&mut file, show_ops);
    } else {
        dump_payload(&mut file, show_ops);
    }
    return Ok(());
}
