use std::fs;
use std::path::{Path, PathBuf};

use crate::error::CompareError;

#[derive(Clone, Debug, PartialEq, Eq)]
// Staged API for upcoming batch-compare wiring in later tasks.
#[allow(dead_code)]
pub struct BatchFileRecord {
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub file_name: String,
    pub parent_dir_name: Option<String>,
}

// Staged API for upcoming batch-compare wiring in later tasks.
#[allow(dead_code)]
pub fn scan_png_files(root: &Path) -> Result<Vec<BatchFileRecord>, CompareError> {
    let root_absolute = if root.is_absolute() {
        root.to_path_buf()
    } else {
        root.canonicalize().map_err(|err| CompareError::FileRead {
            path: root.to_path_buf(),
            reason: err.to_string(),
        })?
    };

    let mut records = Vec::new();
    walk_png_files(&root_absolute, &root_absolute, &mut records)?;
    records.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(records)
}

fn walk_png_files(
    root: &Path,
    directory: &Path,
    records: &mut Vec<BatchFileRecord>,
) -> Result<(), CompareError> {
    let entries = fs::read_dir(directory).map_err(|err| CompareError::FileRead {
        path: directory.to_path_buf(),
        reason: err.to_string(),
    })?;

    for entry_result in entries {
        let entry = entry_result.map_err(|err| CompareError::FileRead {
            path: directory.to_path_buf(),
            reason: err.to_string(),
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|err| CompareError::FileRead {
            path: path.clone(),
            reason: err.to_string(),
        })?;

        if file_type.is_dir() {
            walk_png_files(root, &path, records)?;
            continue;
        }

        if !file_type.is_file() || !is_png_path(&path) {
            continue;
        }

        let relative_path = path
            .strip_prefix(root)
            .map_err(|err| CompareError::FileRead {
                path: path.clone(),
                reason: err.to_string(),
            })?
            .to_path_buf();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| CompareError::FileRead {
                path: path.clone(),
                reason: "file name is not valid UTF-8".to_string(),
            })?
            .to_string();
        let parent_dir_name = relative_path
            .parent()
            .and_then(|parent| parent.file_name())
            .map(|name| name.to_string_lossy().to_string());

        records.push(BatchFileRecord {
            absolute_path: path,
            relative_path,
            file_name,
            parent_dir_name,
        });
    }

    Ok(())
}

fn is_png_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("png"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::scan_png_files;

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "png_metadata_compare_batch_scan_{label}_{}_{}",
                std::process::id(),
                unique
            ));
            fs::create_dir_all(&path).expect("test directory should be created");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn write_file(root: &Path, relative: &str) {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("test parent directory should be created");
        }
        fs::write(path, b"test").expect("test file should be written");
    }

    #[test]
    fn recursively_finds_png_files_and_ignores_non_png_files() {
        let fixture = TestDir::new("recursive_pngs");
        write_file(fixture.path(), "root.png");
        write_file(fixture.path(), "nested/deep/second.PnG");
        write_file(fixture.path(), "nested/ignore.txt");
        write_file(fixture.path(), "not_png.jpg");

        let records = scan_png_files(fixture.path()).expect("scan should succeed");
        let relative_paths: Vec<PathBuf> = records
            .iter()
            .map(|record| record.relative_path.clone())
            .collect();
        assert_eq!(
            relative_paths,
            vec![
                PathBuf::from("nested").join("deep").join("second.PnG"),
                PathBuf::from("root.png")
            ]
        );
    }

    #[test]
    fn captures_immediate_parent_directory_name_for_nested_png_files() {
        let fixture = TestDir::new("parent_dir");
        write_file(fixture.path(), "a/b/c/image.png");

        let records = scan_png_files(fixture.path()).expect("scan should succeed");
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].absolute_path,
            fixture.path().join("a").join("b").join("c").join("image.png")
        );
        assert_eq!(records[0].file_name, "image.png");
        assert_eq!(records[0].parent_dir_name.as_deref(), Some("c"));
    }

    #[test]
    fn root_level_png_has_no_parent_directory_name() {
        let fixture = TestDir::new("root_parent_none");
        write_file(fixture.path(), "root.png");

        let records = scan_png_files(fixture.path()).expect("scan should succeed");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].relative_path, PathBuf::from("root.png"));
        assert_eq!(records[0].parent_dir_name, None);
    }

    #[test]
    fn returns_file_read_error_when_scan_target_is_not_a_directory() {
        let fixture = TestDir::new("scan_failure");
        let file_path = fixture.path().join("not_a_directory.png");
        fs::write(&file_path, b"test").expect("test file should be written");

        let error = scan_png_files(&file_path).expect_err("scan should fail");
        match error {
            crate::error::CompareError::FileRead { path, .. } => assert_eq!(path, file_path),
            other => panic!("expected FileRead error, got {other:?}"),
        }
    }

    #[test]
    fn empty_directory_returns_no_records() {
        let fixture = TestDir::new("empty_directory");

        let records = scan_png_files(fixture.path()).expect("scan should succeed");
        assert!(records.is_empty());
    }
}
