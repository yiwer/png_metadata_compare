use png_metadata_compare::batch_report::UnmatchedSide;
use png_metadata_compare::inspection::{
    DirectorySummary, PairInspection, ScanProgress, SideInspection, inspect_pair,
    inspect_single_side, scan_directory_summary_cancellable,
};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::ipc::Channel;

/// Scan generation counter: every new scan or explicit cancel bumps it.
/// Workers check their captured generation against this; a mismatch means cancel.
static SCAN_GENERATION: AtomicU64 = AtomicU64::new(0);

// Commands are async so the work runs off the main thread — synchronous Tauri
// commands block the webview event loop, which froze the app on large inputs.

#[tauri::command]
pub async fn compare_single(
    left_path: String,
    right_path: String,
) -> Result<PairInspection, String> {
    run_blocking(move || inspect_pair(Path::new(&left_path), Path::new(&right_path))).await
}

#[tauri::command]
pub async fn scan_directory(
    left_dir: String,
    right_dir: String,
    on_progress: Channel<ScanProgress>,
) -> Result<DirectorySummary, String> {
    let my_gen = SCAN_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    run_blocking(move || {
        scan_directory_summary_cancellable(
            Path::new(&left_dir),
            Path::new(&right_dir),
            |p| {
                if should_emit_progress(p) {
                    let _ = on_progress.send(p);
                }
            },
            || SCAN_GENERATION.load(Ordering::SeqCst) != my_gen,
        )
    })
    .await?
    .map_err(|_| "cancelled".to_string())
}

#[tauri::command]
pub fn cancel_scan() {
    SCAN_GENERATION.fetch_add(1, Ordering::SeqCst);
}

#[tauri::command]
pub async fn inspect_single(path: String, side: String) -> Result<SideInspection, String> {
    let parsed_side = match side.as_str() {
        "left" => UnmatchedSide::Left,
        "right" => UnmatchedSide::Right,
        _ => return Err(format!("unsupported side: {side}")),
    };

    run_blocking(move || inspect_single_side(Path::new(&path), parsed_side)).await
}

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| format!("background task failed: {err}"))
}

/// Caps progress IPC traffic at roughly 200 messages per scan; stage
/// transitions and the final event always go through.
fn should_emit_progress(progress: ScanProgress) -> bool {
    if progress.done == 0 || progress.done == progress.total {
        return true;
    }
    let step = (progress.total / 200).max(1);
    progress.done % step == 0
}

#[cfg(test)]
mod tests {
    use super::{compare_single, inspect_single, scan_directory};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn compare_single_returns_service_payload_for_two_pngs() {
        let fixture = TestFixture::new("compare_single");
        let left = fixture.write_png("left.png", r#"{"Title":"Left"}"#);
        let right = fixture.write_png("right.png", r#"{"Title":"Right"}"#);

        let payload = tauri::async_runtime::block_on(compare_single(
            left.display().to_string(),
            right.display().to_string(),
        ))
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

        let payload = tauri::async_runtime::block_on(inspect_single(
            left.display().to_string(),
            "left".to_string(),
        ))
        .expect("inspect command should succeed");

        assert_eq!(payload.side, "left");
        assert_eq!(payload.file_path, left.display().to_string());
        assert_eq!(payload.raw_json.as_deref(), Some(r#"{"Title":"Solo"}"#));
        assert!(payload.error.is_none());
    }

    #[test]
    fn inspect_single_rejects_unsupported_side_values() {
        let fixture = TestFixture::new("inspect_single_invalid_side");
        let left = fixture.write_png("left-only.png", r#"{"Title":"Solo"}"#);

        let error = tauri::async_runtime::block_on(inspect_single(
            left.display().to_string(),
            "center".to_string(),
        ))
        .expect_err("inspect command should reject unsupported sides");

        assert_eq!(error, "unsupported side: center");
    }

    #[test]
    fn cancel_scan_bumps_generation() {
        use std::sync::atomic::Ordering;
        let before = super::SCAN_GENERATION.load(Ordering::SeqCst);
        super::cancel_scan();
        assert_eq!(super::SCAN_GENERATION.load(Ordering::SeqCst), before + 1);
    }

    #[test]
    fn scan_directory_returns_summary_payload() {
        let fixture = BatchFixture::new("scan_directory");
        fixture.write_left_png("same.png", "shared", r#"{"Title":"Same"}"#);
        fixture.write_right_png("same.png", "shared", r#"{"Title":"Same"}"#);
        fixture.write_left_png("diff.png", "shared", r#"{"Title":"Left"}"#);
        fixture.write_right_png("diff.png", "shared", r#"{"Title":"Right"}"#);
        fixture.write_left_png("left-only.png", "shared", r#"{"Title":"Only"}"#);

        let payload = tauri::async_runtime::block_on(scan_directory(
            fixture.left_dir().display().to_string(),
            fixture.right_dir().display().to_string(),
            tauri::ipc::Channel::new(|_| Ok(())),
        ))
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
