use crate::error::CompareError;

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
const STOP_PLATE_KEYWORD: &str = "StopPlateMetadata";

pub fn extract_stop_plate_metadata(bytes: &[u8]) -> Result<String, CompareError> {
    if bytes.len() < PNG_SIGNATURE.len() || &bytes[..PNG_SIGNATURE.len()] != PNG_SIGNATURE {
        return Err(CompareError::InvalidPngSignature);
    }

    let mut offset = PNG_SIGNATURE.len();
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
        if bytes.get(offset..crc_end).is_none() {
            return Err(CompareError::TruncatedChunk);
        }
        offset = crc_end;

        if chunk_type == b"iTXt" {
            if let Some(metadata) = parse_stop_plate_itxt(chunk_data)? {
                return Ok(metadata);
            }
        }
    }

    Err(CompareError::MissingStopPlateMetadata)
}

fn parse_stop_plate_itxt(data: &[u8]) -> Result<Option<String>, CompareError> {
    let Some(keyword_end) = data.iter().position(|&byte| byte == 0) else {
        return Err(CompareError::InvalidInternationalText(
            "missing keyword terminator".into(),
        ));
    };

    let keyword = std::str::from_utf8(&data[..keyword_end]).map_err(|err| {
        CompareError::InvalidInternationalText(format!("keyword is not valid UTF-8: {err}"))
    })?;

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

    if keyword != STOP_PLATE_KEYWORD {
        return Ok(None);
    }

    let text = std::str::from_utf8(&data[offset..])
        .map_err(|err| CompareError::MetadataUtf8(err.to_string()))?;

    Ok(Some(text.to_owned()))
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

    fn png_with_chunks(chunks: Vec<Vec<u8>>) -> Vec<u8> {
        let mut bytes = Vec::from(b"\x89PNG\r\n\x1a\n".as_slice());
        for chunk in chunks {
            bytes.extend(chunk);
        }
        bytes
    }

    fn chunk(kind: [u8; 4], data: Vec<u8>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&kind);
        bytes.extend_from_slice(&data);
        bytes.extend_from_slice(&0u32.to_be_bytes());
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
}
