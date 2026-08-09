use std::io::Write;

const BROTLI_BUFFER_BYTES: usize = 16 * 1024;
const BROTLI_QUALITY: u32 = 6;
const BROTLI_LGWIN: u32 = 22;
const ZSTD_LEVEL: i32 = 3;

/// Encodes decoded response bytes using one HTTP Content-Encoding.
///
/// The returned bytes are used only to derive Resource Timing sizes. Runtime
/// script execution continues to consume the decoded source, like a browser
/// after its network stack has removed Content-Encoding.
pub(crate) fn encode_http_body(decoded: &[u8], encoding: &str) -> Result<Vec<u8>, String> {
    match encoding {
        "" | "identity" => Ok(decoded.to_vec()),
        "gzip" => {
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder
                .write_all(decoded)
                .map_err(|error| format!("gzip encoding failed: {error}"))?;
            encoder
                .finish()
                .map_err(|error| format!("gzip encoding failed: {error}"))
        }
        // HTTP `deflate` is RFC 1950 zlib framing around RFC 1951 DEFLATE.
        "deflate" => {
            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder
                .write_all(decoded)
                .map_err(|error| format!("deflate encoding failed: {error}"))?;
            encoder
                .finish()
                .map_err(|error| format!("deflate encoding failed: {error}"))
        }
        "br" => {
            let mut encoded = Vec::new();
            {
                let mut encoder = brotli::CompressorWriter::new(
                    &mut encoded,
                    BROTLI_BUFFER_BYTES,
                    BROTLI_QUALITY,
                    BROTLI_LGWIN,
                );
                encoder
                    .write_all(decoded)
                    .map_err(|error| format!("brotli encoding failed: {error}"))?;
            }
            Ok(encoded)
        }
        "zstd" => zstd::bulk::compress(decoded, ZSTD_LEVEL)
            .map_err(|error| format!("zstd encoding failed: {error}")),
        _ => Err(format!("unsupported HTTP content encoding {encoding:?}")),
    }
}

pub(crate) fn encoded_http_body_size(decoded: &[u8], encoding: &str) -> Result<usize, String> {
    if encoding.is_empty() || encoding == "identity" {
        return Ok(decoded.len());
    }
    encode_http_body(decoded, encoding).map(|encoded| encoded.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn source() -> Vec<u8> {
        "const payload = 'Edge resource timing compression';\n"
            .repeat(256)
            .into_bytes()
    }

    #[test]
    fn all_http_encodings_round_trip_real_bytes() {
        let decoded = source();
        for encoding in ["gzip", "deflate", "br", "zstd"] {
            let encoded = encode_http_body(&decoded, encoding).expect("encode body");
            assert!(encoded.len() < decoded.len(), "{encoding} did not compress");
            assert_eq!(
                encoded.len(),
                encoded_http_body_size(&decoded, encoding).expect("encoded size")
            );

            let mut round_trip = Vec::new();
            match encoding {
                "gzip" => flate2::read::GzDecoder::new(encoded.as_slice())
                    .read_to_end(&mut round_trip)
                    .expect("decode gzip"),
                "deflate" => flate2::read::ZlibDecoder::new(encoded.as_slice())
                    .read_to_end(&mut round_trip)
                    .expect("decode deflate"),
                "br" => brotli::Decompressor::new(encoded.as_slice(), BROTLI_BUFFER_BYTES)
                    .read_to_end(&mut round_trip)
                    .expect("decode brotli"),
                "zstd" => zstd::stream::read::Decoder::new(encoded.as_slice())
                    .expect("create zstd decoder")
                    .read_to_end(&mut round_trip)
                    .expect("decode zstd"),
                _ => unreachable!(),
            };
            assert_eq!(round_trip, decoded, "{encoding} round trip");
        }
    }

    #[test]
    fn identity_is_a_zero_work_size_path() {
        let decoded = source();
        assert_eq!(encoded_http_body_size(&decoded, ""), Ok(decoded.len()));
        assert_eq!(
            encoded_http_body_size(&decoded, "identity"),
            Ok(decoded.len())
        );
    }
}
