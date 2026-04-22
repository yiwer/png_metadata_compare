mod app;
mod diff;
mod error;
mod metadata;
mod png_reader;

mod ui {
    pub mod detail;
    pub mod summary;
    pub mod tree;
}

use app::PngMetadataCompareApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "PNG Metadata Compare",
        options,
        Box::new(|_cc| Ok(Box::new(PngMetadataCompareApp::default()))),
    )
}

#[cfg(test)]
#[test]
fn compare_is_disabled_until_both_paths_are_present() {
    let mut app = app::PngMetadataCompareApp::default();
    assert!(!app.can_compare());

    app.left_path = Some("left.png".into());
    assert!(!app.can_compare());

    app.right_path = Some("right.png".into());
    assert!(app.can_compare());
}

#[cfg(test)]
#[test]
fn compare_pipeline_builds_diff_and_counts() {
    app::run_compare_pipeline_builds_diff_and_counts_test();
}

#[cfg(test)]
#[test]
fn formats_summary_lines_for_sidebar() {
    ui::summary::formats_summary_lines_for_sidebar_test();
}

#[cfg(test)]
#[test]
fn hides_unchanged_nodes_when_only_differences_is_enabled() {
    ui::tree::hides_unchanged_nodes_when_only_differences_is_enabled_test();
}
