use crate::error::CompareError;
use std::path::Path;

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
const STOP_PLATE_KEYWORD: &str = "StopPlateMetadata";

pub fn extract_stop_plate_metadata(bytes: &[u8]) -> Result<String, CompareError> {
    if bytes.len() < PNG_SIGNATURE.len() || &bytes[..PNG_SIGNATURE.len()] != PNG_SIGNATURE {
        return Err(CompareError::InvalidPngSignature);
    }

    let mut offset = PNG_SIGNATURE.len();
    let mut metadata = None;
    let mut saw_iend = false;
    while offset < bytes.len() {
        let Some(length_bytes) = bytes.get(offset..offset + 4) else {
            return Err(CompareError::TruncatedChunk);
        };
        let length = u32::from_be_bytes(length_bytes.try_into().expect("length slice")) as usize;
        offset += 4;

        let Some(chunk_type) = bytes.get(offset..offset + 4) else {
            return Err(CompareError::TruncatedChunk);
        };
        offset += 4;

        let Some(data_end) = offset.checked_add(length) else {
            return Err(CompareError::TruncatedChunk);
        };
        let Some(chunk_data) = bytes.get(offset..data_end) else {
            return Err(CompareError::TruncatedChunk);
        };
        offset = data_end;

        let Some(crc_end) = offset.checked_add(4) else {
            return Err(CompareError::TruncatedChunk);
        };
        let Some(expected_crc_bytes) = bytes.get(offset..crc_end) else {
            return Err(CompareError::TruncatedChunk);
        };
        let expected_crc = u32::from_be_bytes(expected_crc_bytes.try_into().expect("crc slice"));
        if chunk_crc(chunk_type, chunk_data) != expected_crc {
            return Err(CompareError::TruncatedChunk);
        }
        offset = crc_end;

        if chunk_type == b"iTXt" {
            if metadata.is_none() {
                metadata = parse_stop_plate_itxt(chunk_data)?;
            }
        }
        if chunk_type == b"IEND" {
            saw_iend = true;
            break;
        }
    }

    if saw_iend {
        metadata.ok_or(CompareError::MissingStopPlateMetadata)
    } else {
        Err(CompareError::TruncatedChunk)
    }
}

pub fn extract_stop_plate_metadata_from_file(path: &Path) -> Result<String, CompareError> {
    let bytes = std::fs::read(path).map_err(|err| CompareError::FileRead {
        path: path.to_path_buf(),
        reason: err.to_string(),
    })?;

    extract_stop_plate_metadata(&bytes)
}

fn parse_stop_plate_itxt(data: &[u8]) -> Result<Option<String>, CompareError> {
    let Some(keyword_end) = data.iter().position(|&byte| byte == 0) else {
        return Ok(None);
    };

    let keyword = std::str::from_utf8(&data[..keyword_end]).map_err(|err| {
        CompareError::InvalidInternationalText(format!("keyword is not valid UTF-8: {err}"))
    });
    let Ok(keyword) = keyword else {
        return Ok(None);
    };

    if keyword != STOP_PLATE_KEYWORD {
        return Ok(None);
    }

    let mut offset = keyword_end + 1;
    let Some(&compression_flag) = data.get(offset) else {
        return Err(CompareError::InvalidInternationalText(
            "missing compression flag".into(),
        ));
    };
    offset += 1;

    let Some(&compression_method) = data.get(offset) else {
        return Err(CompareError::InvalidInternationalText(
            "missing compression method".into(),
        ));
    };
    offset += 1;

    if compression_flag == 1 {
        return Err(CompareError::UnsupportedCompressedText);
    }
    if compression_flag > 1 {
        return Err(CompareError::InvalidInternationalText(format!(
            "invalid compression flag: {compression_flag}"
        )));
    }
    if compression_method != 0 {
        return Err(CompareError::InvalidInternationalText(format!(
            "invalid compression method: {compression_method}"
        )));
    }

    let Some(language_end) = data[offset..].iter().position(|&byte| byte == 0) else {
        return Err(CompareError::InvalidInternationalText(
            "missing language tag terminator".into(),
        ));
    };
    offset += language_end + 1;

    let Some(translated_end) = data[offset..].iter().position(|&byte| byte == 0) else {
        return Err(CompareError::InvalidInternationalText(
            "missing translated keyword terminator".into(),
        ));
    };
    offset += translated_end + 1;

    let text = std::str::from_utf8(&data[offset..])
        .map_err(|err| CompareError::MetadataUtf8(err.to_string()))?;

    Ok(Some(text.to_owned()))
}

fn chunk_crc(chunk_type: &[u8], chunk_data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in chunk_type.iter().copied().chain(chunk_data.iter().copied()) {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = if crc & 1 == 1 { 0xedb8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_stop_plate_metadata_text() {
        let png = png_with_chunks(vec![stop_plate_itxt(r#"{"plate":"ABC123"}"#)]);

        let metadata = extract_stop_plate_metadata(&png).expect("metadata should be extracted");

        assert_eq!(metadata, r#"{"plate":"ABC123"}"#);
    }

    #[test]
    fn returns_missing_metadata_when_keyword_is_absent() {
        let png = png_with_chunks(vec![chunk(
            *b"iTXt",
            b"OtherKeyword\0\0\0\0\0{\"plate\":\"ABC123\"}".to_vec(),
        )]);

        let error = extract_stop_plate_metadata(&png).expect_err("metadata should be missing");

        assert!(matches!(error, CompareError::MissingStopPlateMetadata));
    }

    #[test]
    fn rejects_invalid_png_signature() {
        let error = extract_stop_plate_metadata(b"not a png").expect_err("signature should fail");

        assert!(matches!(error, CompareError::InvalidPngSignature));
    }

    #[test]
    fn rejects_compressed_itxt_metadata() {
        let png = png_with_chunks(vec![chunk(
            *b"iTXt",
            b"StopPlateMetadata\0\x01\0\0\0{\"plate\":\"ABC123\"}".to_vec(),
        )]);

        let error =
            extract_stop_plate_metadata(&png).expect_err("compressed metadata should be rejected");

        assert!(matches!(error, CompareError::UnsupportedCompressedText));
    }

    #[test]
    fn skips_invalid_non_target_itxt_before_stop_plate_metadata() {
        let png = png_with_chunks(vec![
            chunk(*b"iTXt", b"\xff\xfe\xfd".to_vec()),
            stop_plate_itxt(r#"{"plate":"ABC123"}"#),
        ]);

        let metadata = extract_stop_plate_metadata(&png).expect("later metadata should be found");

        assert_eq!(metadata, r#"{"plate":"ABC123"}"#);
    }

    #[test]
    fn ignores_trailer_bytes_after_iend() {
        let mut png = png_with_chunks(Vec::new());
        png.extend_from_slice(b"trailing junk that is not a chunk");

        let error = extract_stop_plate_metadata(&png)
            .expect_err("trailer bytes after IEND should be ignored");

        assert!(matches!(error, CompareError::MissingStopPlateMetadata));
    }

    #[test]
    fn rejects_truncated_png_even_when_stop_plate_metadata_appears_early() {
        let mut png = Vec::from(b"\x89PNG\r\n\x1a\n".as_slice());
        png.extend(stop_plate_itxt(r#"{"plate":"ABC123"}"#));

        let error = extract_stop_plate_metadata(&png)
            .expect_err("metadata should not succeed before IEND is validated");

        assert!(matches!(error, CompareError::TruncatedChunk));
    }

    #[test]
    fn rejects_png_without_terminal_iend_chunk() {
        let mut png = Vec::from(b"\x89PNG\r\n\x1a\n".as_slice());
        png.extend(chunk(*b"IDAT", Vec::new()));

        let error = extract_stop_plate_metadata(&png)
            .expect_err("missing IEND should be treated as malformed");

        assert!(matches!(error, CompareError::TruncatedChunk));
    }

    #[test]
    fn rejects_png_with_bad_crc_even_when_structure_and_metadata_are_valid() {
        let mut png = png_with_chunks(vec![stop_plate_itxt(r#"{"plate":"ABC123"}"#)]);
        corrupt_crc(&mut png, 0);

        let error =
            extract_stop_plate_metadata(&png).expect_err("bad CRC should make the PNG malformed");

        assert!(matches!(error, CompareError::TruncatedChunk));
    }

    fn png_with_chunks(chunks: Vec<Vec<u8>>) -> Vec<u8> {
        let mut bytes = Vec::from(b"\x89PNG\r\n\x1a\n".as_slice());
        for chunk in chunks {
            bytes.extend(chunk);
        }
        bytes.extend(chunk(*b"IEND", Vec::new()));
        bytes
    }

    fn chunk(kind: [u8; 4], data: Vec<u8>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&kind);
        bytes.extend_from_slice(&data);
        bytes.extend_from_slice(&chunk_crc(&kind, &data).to_be_bytes());
        bytes
    }

    fn stop_plate_itxt(json: &str) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"StopPlateMetadata");
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);
        data.extend_from_slice(json.as_bytes());
        chunk(*b"iTXt", data)
    }

    fn corrupt_crc(png: &mut [u8], chunk_index: usize) {
        let mut offset = PNG_SIGNATURE.len();
        for index in 0..=chunk_index {
            let length = u32::from_be_bytes(
                png[offset..offset + 4]
                    .try_into()
                    .expect("chunk length should exist"),
            ) as usize;
            let crc_offset = offset + 8 + length;
            if index == chunk_index {
                png[crc_offset..crc_offset + 4].copy_from_slice(&1u32.to_be_bytes());
                return;
            }
            offset = crc_offset + 4;
        }

        panic!("chunk index out of range: {chunk_index}");
    }
}
