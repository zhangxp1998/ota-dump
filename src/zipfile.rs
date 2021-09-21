use binread::{io::Cursor, BinRead, BinReaderExt};

use std::cmp::min;
use std::io::{Seek, SeekFrom};

#[derive(BinRead)]
#[br(magic = b"\x50\x4b\x05\x06", little)]
#[derive(Debug)]
struct EndOfCentralDirectory {
    disk: u16,
    central_directory_disk: u16,
    num_central_directory_records: u16,
    total_central_directory_recrods: u16,
    central_directory_size: u32,
    central_directory_offset: u32,
    comment_length: u16,
    #[br(count=comment_length)]
    comment: Vec<u8>,
}

#[derive(BinRead)]
#[br(magic = b"\x50\x4b\x01\x02", little)]
#[derive(Debug)]
pub struct CentralDirectoryRecord {
    src_version: u16,
    extract_version: u16,
    flags: u16,
    compression: u16,
    mod_time: u16,
    mod_date: u16,
    crc: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    filename_len: u16,
    extra_len: u16,
    comment_len: u16,
    disk: u16,
    file_attr: u16,
    external_file_attr: u32,
    local_file_header_offset: u32,
    #[br(count=filename_len)]
    filename: Vec<u8>,
    #[br(count=extra_len)]
    extra: Vec<u8>,
    #[br(count=comment_len)]
    comment: Vec<u8>,
}

impl CentralDirectoryRecord {
    pub fn get_filename(&self) -> &str {
        return std::str::from_utf8(&self.filename).expect("Invalid filename");
    }
}

#[derive(BinRead)]
#[br(magic = b"\x50\x4b\x03\x04", little)]
#[derive(Debug)]
pub struct LocalFileHeader {
    extract_version: u16,
    flags: u16,
    compression: u16,
    mod_time: u16,
    mod_date: u16,
    crc: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    filename_len: u16,
    extra_len: u16,
    #[br(count=filename_len)]
    filename: Vec<u8>,
    #[br(count=extra_len)]
    extra: Vec<u8>,
}

#[derive(Debug)]
pub struct ZipEntry {
    central_directory_record: CentralDirectoryRecord,
    local_file_header: LocalFileHeader,
}

impl ZipEntry {
    pub fn get_filename(&self) -> String {
        return String::from_utf8(self.local_file_header.filename.clone())
            .expect("Invalid filename");
    }
    pub fn get_uncompressed_size(&self) -> usize {
        return self.local_file_header.uncompressed_size as usize;
    }
    pub fn get_compressed_size(&self) -> usize {
        return self.local_file_header.compressed_size as usize;
    }
    pub fn get_compressed_data_offset(&self) -> usize {
        // Local file header has size 30, + filename length + extra length
        return (self.central_directory_record.local_file_header_offset as usize
            + 30
            + self.local_file_header.extra_len as usize
            + self.local_file_header.filename_len as usize) as usize;
    }
    pub fn is_compressed(&self) -> bool {
        self.local_file_header.compression != 0
    }
}

pub struct ZipArchive<'a> {
    data: &'a [u8],
    eocd: EndOfCentralDirectory,
}
fn locate_end_of_central_directory(bytes: &[u8]) -> Result<EndOfCentralDirectory, binread::Error> {
    // Only search for End of Central Directory in last 64K of zip archive
    let chunk_size = min(bytes.len(), 64 * 1024);
    let chunk: &[u8] = &bytes[bytes.len() - chunk_size..];
    let eocd_magic: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
    let eocd_offset = match chunk
        .windows(eocd_magic.len())
        .position(|window| window == eocd_magic)
    {
        Some(offset) => offset,
        None => {
            return Err(binread::Error::AssertFail {
                pos: 0,
                message: "Failed to find end of central directory record".to_string(),
            })
        }
    };
    let mut reader = Cursor::new(&chunk[eocd_offset..]);
    return Ok(reader.read_ne()?);
}

impl ZipArchive<'_> {
    pub fn new(data: &[u8]) -> Result<ZipArchive, binread::Error> {
        let eocd = locate_end_of_central_directory(data)?;
        return Ok(ZipArchive { data, eocd });
    }
    pub fn get_compressed_data(&self, entry: &ZipEntry) -> &[u8] {
        &self.data[entry.get_compressed_data_offset()
            ..entry.get_compressed_data_offset() + entry.get_compressed_size()]
    }

    pub fn into_iter(&self) -> ZipEntryIterator {
        let eocd = &self.eocd;
        let cd_offset = eocd.central_directory_offset as usize;
        let cd_size = eocd.central_directory_size as usize;
        ZipEntryIterator {
            reader: Cursor::new(&self.data[cd_offset..cd_offset + cd_size]),
            file_header_reader: Cursor::new(self.data),
            total_central_directory_recrods: eocd.total_central_directory_recrods as usize,
            idx: 0,
        }
    }
}

pub struct ZipEntryIterator<'a> {
    reader: Cursor<&'a [u8]>,
    file_header_reader: Cursor<&'a [u8]>,
    total_central_directory_recrods: usize,
    idx: usize,
}

impl Iterator for ZipEntryIterator<'_> {
    type Item = ZipEntry;
    fn next(&mut self) -> Option<ZipEntry> {
        if self.idx >= self.total_central_directory_recrods {
            return None;
        }
        let cd = self.reader.read_ne();
        if cd.is_err() {
            println!(
                "Failed to parse {}th central directory record {:?}",
                self.idx, cd
            );
            return None;
        }
        let cd: CentralDirectoryRecord = cd.unwrap();
        let res = self
            .file_header_reader
            .seek(SeekFrom::Start(cd.local_file_header_offset as u64));
        if res.is_err() {
            println!(
                "Failed to seek to local file header for {}, offset {}, {:?}",
                cd.get_filename(),
                cd.local_file_header_offset,
                res
            );
            return None;
        }
        let local_file_header = self.file_header_reader.read_ne();
        if local_file_header.is_err() {
            println!(
                "Failed to parse local file header: {}, {:?}",
                cd.get_filename(),
                local_file_header
            );
            return None;
        }
        let local_file_header = local_file_header.unwrap();
        let zip_record = ZipEntry {
            central_directory_record: cd,
            local_file_header,
        };
        self.idx += 1;
        return Some(zip_record);
    }
}
