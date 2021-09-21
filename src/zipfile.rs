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
}
impl ZipArchive<'_> {
    pub fn new(data: &[u8]) -> ZipArchive {
        return ZipArchive { data };
    }
    pub fn get_compressed_data(&self, entry: &ZipEntry) -> &[u8] {
        &self.data[entry.get_compressed_data_offset()
            ..entry.get_compressed_data_offset() + entry.get_compressed_size()]
    }

    pub fn get_zip_entries(&self) -> Result<Vec<ZipEntry>, binread::Error> {
        let bytes = self.data;
        // Only search for End of Central Directory in last 64K of zip archive
        let chunk_size = min(bytes.len(), 64 * 1024);
        let chunk: &[u8] = &bytes[bytes.len() - chunk_size..];
        let eocd_magic: &[u8; 4] = &[0x50, 0x4b, 0x05, 0x06];
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
        let eocd: EndOfCentralDirectory = reader.read_ne()?;
        let cd_offset: usize = eocd.central_directory_offset as usize;
        let cd_size: usize = eocd.central_directory_size as usize;
        let mut reader = Cursor::new(&bytes[cd_offset..cd_offset + cd_size]);
        let mut file_header_reader = Cursor::new(bytes);
        let mut records = Vec::<ZipEntry>::new();
        records.reserve(eocd.total_central_directory_recrods as usize);
        for _ in 0..eocd.total_central_directory_recrods {
            let cd: CentralDirectoryRecord = reader.read_ne()?;
            file_header_reader.seek(SeekFrom::Start(cd.local_file_header_offset as u64))?;
            let local_file_header: LocalFileHeader = file_header_reader.read_ne()?;
            let zip_record = ZipEntry {
                central_directory_record: cd,
                local_file_header,
            };
            records.push(zip_record);
        }
        Ok(records)
    }
}
