#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CompressionMethod {
    /// Store the file as is
    Stored,
    /// Compress the file using Deflate
    Deflated,
    /// Compress the file using BZIP2
    Bzip2,
    /// Unsupported compression method
    Unsupported(u16),
}

impl CompressionMethod {
    pub const STORE: Self = CompressionMethod::Stored;
    pub const SHRINK: Self = CompressionMethod::Unsupported(1);
    pub const REDUCE_1: Self = CompressionMethod::Unsupported(2);
    pub const REDUCE_2: Self = CompressionMethod::Unsupported(3);
    pub const REDUCE_3: Self = CompressionMethod::Unsupported(4);
    pub const REDUCE_4: Self = CompressionMethod::Unsupported(5);
    pub const IMPLODE: Self = CompressionMethod::Unsupported(6);
    pub const DEFLATE: Self = CompressionMethod::Deflated;
    pub const DEFLATE64: Self = CompressionMethod::Unsupported(9);
    pub const PKWARE_IMPLODE: Self = CompressionMethod::Unsupported(10);
    pub const BZIP2: Self = CompressionMethod::Bzip2;
    pub const LZMA: Self = CompressionMethod::Unsupported(14);
    pub const IBM_ZOS_CMPSC: Self = CompressionMethod::Unsupported(16);
    pub const IBM_TERSE: Self = CompressionMethod::Unsupported(18);
    pub const ZSTD_DEPRECATED: Self = CompressionMethod::Unsupported(20);
    pub const ZSTD: Self = CompressionMethod::Unsupported(93);
    pub const MP3: Self = CompressionMethod::Unsupported(94);
    pub const XZ: Self = CompressionMethod::Unsupported(95);
    pub const JPEG: Self = CompressionMethod::Unsupported(96);
    pub const WAVPACK: Self = CompressionMethod::Unsupported(97);
    pub const PPMD: Self = CompressionMethod::Unsupported(98);
}
