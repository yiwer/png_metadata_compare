use png_metadata_compare::batch_report::UnmatchedSide;
use png_metadata_compare::inspection::{
    DirectorySummary, PairInspection, SideInspection, inspect_pair, inspect_single_side,
    scan_directory_summary,
};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopSide {
    Left,
    Right,
}

impl From<DesktopSide> for UnmatchedSide {
    fn from(value: DesktopSide) -> Self {
        match value {
            DesktopSide::Left => UnmatchedSide::Left,
            DesktopSide::Right => UnmatchedSide::Right,
        }
    }
}

#[tauri::command]
pub fn compare_single(left_path: String, right_path: String) -> Result<PairInspection, String> {
    Ok(inspect_pair(Path::new(&left_path), Path::new(&right_path)))
}

#[tauri::command]
pub fn scan_directory(left_dir: String, right_dir: String) -> Result<DirectorySummary, String> {
    Ok(scan_directory_summary(
        Path::new(&left_dir),
        Path::new(&right_dir),
    ))
}

#[tauri::command]
pub fn inspect_single(path: String, side: DesktopSide) -> Result<SideInspection, String> {
    Ok(inspect_single_side(Path::new(&path), side.into()))
}

#[cfg(test)]
mod tests {
    use super::{DesktopSide, compare_single, inspect_single, scan_directory};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn compare_single_returns_service_payload_for_two_pngs() {
        let fixture = TestFixture::new("compare_single");
        let left = fixture.write_png("left.png", r#"{"Title":"Left"}"#);
        let right = fixture.write_png("right.png", r#"{"Title":"Right"}"#);

        let payload = compare_single(
            left.display().to_string(),
            right.display().to_string(),
        )
        .expect("compare command should succeed");

        assert_eq!(payload.left.file_path, left.display().to_string());
        assert_eq!(payload.right.file_path, right.display().to_string());
        assert_eq!(payload.diff_summary.modified, 1);
        assert_eq!(payload.left.raw_json.as_deref(), Some(r#"{"Title":"Left"}"#));
        assert_eq!(payload.right.raw_json.as_deref(), Some(r#"{"Title":"Right"}"#));
    }

    #[test]
    fn inspect_single_returns_side_specific_payload() {
        let fixture = TestFixture::new("inspect_single");
        let left = fixture.write_png("left-only.png", r#"{"Title":"Solo"}"#);

        let payload = inspect_single(left.display().to_string(), DesktopSide::Left)
            .expect("inspect command should succeed");

        assert_eq!(payload.side, "left");
        assert_eq!(payload.file_path, left.display().to_string());
        assert_eq!(payload.raw_json.as_deref(), Some(r#"{"Title":"Solo"}"#));
        assert!(payload.error.is_none());
    }

    #[test]
    fn scan_directory_returns_summary_payload() {
        let fixture = BatchFixture::new("scan_directory");
        fixture.write_left_png("same.png", "shared", r#"{"Title":"Same"}"#);
        fixture.write_right_png("same.png", "shared", r#"{"Title":"Same"}"#);
        fixture.write_left_png("diff.png", "shared", r#"{"Title":"Left"}"#);
        fixture.write_right_png("diff.png", "shared", r#"{"Title":"Right"}"#);
        fixture.write_left_png("left-only.png", "shared", r#"{"Title":"Only"}"#);

        let payload = scan_directory(
            fixture.left_dir().display().to_string(),
            fixture.right_dir().display().to_string(),
        )
        .expect("scan command should succeed");

        assert_eq!(payload.counts.identical, 1);
        assert_eq!(payload.counts.different, 1);
        assert_eq!(payload.counts.left_only, 1);
    }

    struct TestFixture {
        root: PathBuf,
    }

    impl TestFixture {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "png_metadata_compare_desktop_api_{label}_{}_{}",
                std::process::id(),
                unique
            ));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn write_png(&self, name: &str, json: &str) -> PathBuf {
            let path = self.root.join(name);
            fs::write(&path, png_with_metadata(json)).unwrap();
            path
        }
    }

    impl Drop for TestFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    struct BatchFixture {
        root: PathBuf,
        left: PathBuf,
        right: PathBuf,
    }

    impl BatchFixture {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "png_metadata_compare_desktop_api_batch_{label}_{}_{}",
                std::process::id(),
                unique
            ));
            let left = root.join("left");
            let right = root.join("right");
            fs::create_dir_all(&left).unwrap();
            fs::create_dir_all(&right).unwrap();
            Self { root, left, right }
        }

        fn left_dir(&self) -> &Path {
            &self.left
        }

        fn right_dir(&self) -> &Path {
            &self.right
        }

        fn write_left_png(&self, name: &str, dir: &str, json: &str) {
            write_batch_png(&self.left, name, dir, json);
        }

        fn write_right_png(&self, name: &str, dir: &str, json: &str) {
            write_batch_png(&self.right, name, dir, json);
        }
    }

    impl Drop for BatchFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn write_batch_png(root: &Path, name: &str, dir: &str, json: &str) {
        let folder = root.join(dir);
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join(name), png_with_metadata(json)).unwrap();
    }

    fn png_with_metadata(json: &str) -> Vec<u8> {
        let mut bytes = Vec::from(b"\x89PNG\r\n\x1a\n".as_slice());
        bytes.extend(png_chunk(*b"iTXt", stop_plate_itxt_data(json)));
        bytes.extend(png_chunk(*b"IEND", Vec::new()));
        bytes
    }

    fn stop_plate_itxt_data(json: &str) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"StopPlateMetadata");
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);
        data.extend_from_slice(json.as_bytes());
        data
    }

    fn png_chunk(kind: [u8; 4], data: Vec<u8>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&kind);
        bytes.extend_from_slice(&data);
        bytes.extend_from_slice(&png_chunk_crc(kind, &data).to_be_bytes());
        bytes
    }

    fn png_chunk_crc(kind: [u8; 4], data: &[u8]) -> u32 {
        let mut crc = 0xffff_ffffu32;
        for byte in kind.into_iter().chain(data.iter().copied()) {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                let mask = if crc & 1 == 1 { 0xedb8_8320 } else { 0 };
                crc = (crc >> 1) ^ mask;
            }
        }
        !crc
    }
}
