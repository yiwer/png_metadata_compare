use png_metadata_compare::app::PngMetadataCompareApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "PNG Metadata Compare",
        options,
        Box::new(|_cc| Ok(Box::new(PngMetadataCompareApp::default()))),
    )
}
