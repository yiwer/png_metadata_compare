#[derive(Default)]
pub struct PngMetadataCompareApp {
    pub left_path: Option<String>,
    pub right_path: Option<String>,
}

impl PngMetadataCompareApp {
    pub fn can_compare(&self) -> bool {
        self.left_path.is_some() && self.right_path.is_some()
    }
}

impl eframe::App for PngMetadataCompareApp {
    fn update(&mut self, _ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {}
}

#[cfg(test)]
mod tests {
    use super::PngMetadataCompareApp;

    #[test]
    fn compare_is_disabled_until_both_paths_are_present() {
        let mut app = PngMetadataCompareApp::default();
        assert!(!app.can_compare());

        app.left_path = Some("left.png".into());
        assert!(!app.can_compare());

        app.right_path = Some("right.png".into());
        assert!(app.can_compare());
    }
}
