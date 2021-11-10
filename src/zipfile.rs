use binread::{io::Cursor, BinRead, BinReaderExt};

use std::cmp::min;
use std::io::{Read, Seek, SeekFrom};

#[derive(BinRead)]
#[br(magic = b"\x50\x4b\x05\x06", little)]
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

pub struct ZipArchive<File: Read + Seek> {
    data: File,
    eocd: EndOfCentralDirectory,
    central_directory: Vec<CentralDirectoryRecord>,
}

fn stream_len(file: &mut dyn Seek) -> Result<u64, std::io::Error> {
    let cur_pos = file.stream_position()?;
    file.seek(SeekFrom::End(0))?;
    let res = file.stream_position()?;
    file.seek(SeekFrom::Start(cur_pos))?;
    Ok(res)
}

fn locate_end_of_central_directory<File: Read + Seek>(
    file: &mut File,
) -> Result<EndOfCentralDirectory, binread::Error> {
    // Only search for End of Central Directory in last 64K of zip archive
    let file_size = stream_len(file)?;
    let chunk_size = min(file_size, 64 * 1024);
    file.seek(SeekFrom::End(-(chunk_size as i64)))?;
    let mut chunk: Vec<u8> = vec![0; chunk_size as usize];

    file.read_exact(&mut chunk[..])
        .expect(format!("Failed to read last {} bytes", chunk_size).as_str());
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
    return reader.read_ne();
}

impl<File: Read + Seek> ZipArchive<File> {
    pub fn new(mut data: File) -> Result<ZipArchive<File>, binread::Error> {
        let eocd = locate_end_of_central_directory(&mut data)?;
        data.seek(SeekFrom::Start(eocd.central_directory_offset as u64))
            .unwrap();
        let mut central_directory_data = vec![0 as u8; eocd.central_directory_size as usize];
        data.read_exact(&mut central_directory_data[..]).unwrap();
        let mut central_directory_reader = Cursor::new(central_directory_data);
        let mut central_directory_records: Vec<CentralDirectoryRecord> = vec![];
        for _ in 0..eocd.num_central_directory_records {
            let cd: CentralDirectoryRecord = central_directory_reader.read_ne()?;
            central_directory_records.push(cd);
        }
        return Ok(ZipArchive {
            data,
            eocd,
            central_directory: central_directory_records,
        });
    }
    pub fn get_compressed_data_file(
        &mut self,
        entry: &ZipEntry,
    ) -> Result<&mut File, std::io::Error> {
        self.data
            .seek(SeekFrom::Start(entry.get_compressed_data_offset() as u64))?;
        Ok(&mut self.data)
    }

    pub fn get_compressed_data(&mut self, entry: &ZipEntry) -> Vec<u8> {
        let mut buf = vec![0 as u8; entry.get_compressed_size()];
        let reader = self.get_compressed_data_file(entry).unwrap();
        reader.read(&mut buf[..]).unwrap();
        buf
    }

    pub fn into_iter(&mut self) -> ZipEntryIterator<File> {
        self.data
            .seek(SeekFrom::Start(self.eocd.central_directory_offset as u64))
            .unwrap();
        ZipEntryIterator {
            reader: &mut self.data,
            eocd: &self.central_directory,
            idx: 0,
        }
    }
}

pub struct ZipEntryIterator<'a, File: Read + Seek> {
    reader: &'a mut File,
    eocd: &'a Vec<CentralDirectoryRecord>,
    idx: usize,
}

impl<'a, T: Read + Seek> Iterator for ZipEntryIterator<'a, T> {
    type Item = ZipEntry;
    fn next(&mut self) -> Option<ZipEntry> {
        if self.idx >= self.eocd.len() {
            return None;
        }
        let cd = self.eocd[self.idx].clone();
        let res = self
            .reader
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
        let local_file_header = self.reader.read_ne();
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
