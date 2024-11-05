use std::io::Read;

use flate2::read::{GzDecoder, ZlibDecoder};
use ic_canister_log::log;

use crate::{logs::DEBUG, rpc_client::RpcError};

/// Decompresses response body if it is compressed.
pub fn decompress_if_needed(body: Vec<u8>) -> Result<Vec<u8>, RpcError> {
    let bytes = body.as_slice();

    if let Some(compression) = detect_compression(bytes) {
        log!(DEBUG, "Decompressing response with compression type: {:?}", compression);
        match compression {
            CompressionType::Zlib | CompressionType::Deflate => {
                let mut d = ZlibDecoder::new(body.as_slice());
                let mut decompressed = Vec::new();
                d.read_to_end(&mut decompressed)
                    .map_err(|e| RpcError::ParseError(format!("Deflate decompression failed: {}", e)))?;
                Ok(decompressed)
            }
            CompressionType::Gzip => {
                let mut d = GzDecoder::new(body.as_slice());
                let mut decompressed = Vec::new();
                d.read_to_end(&mut decompressed)
                    .map_err(|e| RpcError::ParseError(format!("Gzip decompression failed: {}", e)))?;
                Ok(decompressed)
            }
            _ => Err(RpcError::ParseError(format!(
                "Unsupported compression type: {:?}",
                compression
            ))),
        }
    } else {
        Ok(body)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[non_exhaustive]
pub enum CompressionType {
    Deflate,
    Zlib,
    Gzip,
    BZIP2,
    Zstd,
    XZ,
    LZ4,
    Brotli,
    Snappy,
}

fn detect_compression(bytes: &[u8]) -> Option<CompressionType> {
    if bytes[0] == 0x78 && bytes[1] == 0x9c {
        Some(CompressionType::Deflate)
    } else if bytes[0] == 0x78 && bytes[1] == 0x01 {
        Some(CompressionType::Zlib)
    } else if bytes[0] == 0x1f && bytes[1] == 0x8b {
        Some(CompressionType::Gzip)
    } else if bytes[0] == 0x42 && bytes[1] == 0x5a {
        Some(CompressionType::BZIP2)
    } else if bytes[0] == 0x28 && bytes[1] == 0xb5 && bytes[2] == 0x2f && bytes[3] == 0xfd {
        Some(CompressionType::Zstd)
    } else if bytes[0] == 0xfd
        && bytes[1] == 0x37
        && bytes[2] == 0x7a
        && bytes[3] == 0x58
        && bytes[4] == 0x5a
        && bytes[5] == 0x00
    {
        Some(CompressionType::XZ)
    } else if bytes[0] == 0x04 && bytes[1] == 0x22 && bytes[2] == 0x4d && bytes[3] == 0x18 {
        Some(CompressionType::LZ4)
    } else if bytes[0] == 0xce && bytes[1] == 0x00 && bytes[2] == 0x00 && bytes[3] == 0x00 {
        Some(CompressionType::Brotli)
    } else if bytes[0] == 0xff && bytes[1] == 0x06 && bytes[2] == 0x00 && bytes[3] == 0x00 {
        Some(CompressionType::Snappy)
    } else {
        None
    }
}
