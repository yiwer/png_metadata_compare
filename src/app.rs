use crate::ui::{detail, summary, tree};

#[derive(Default)]
pub struct PngMetadataCompareApp {
    pub left_path: Option<String>,
    pub right_path: Option<String>,
}

impl PngMetadataCompareApp {
    pub fn can_compare(&self) -> bool {
        self.left_path.is_some() && self.right_path.is_some()
    }

    fn render_scaffold(&mut self, ctx: &eframe::egui::Context) {
        eframe::egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.heading("PNG Metadata Compare");
            ui.label("Task 1 scaffold");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Left: {}",
                    self.left_path.as_deref().unwrap_or("Not selected")
                ));
                ui.label(format!(
                    "Right: {}",
                    self.right_path.as_deref().unwrap_or("Not selected")
                ));
                ui.add_enabled(self.can_compare(), eframe::egui::Button::new("Compare"));
            });
        });

        eframe::egui::SidePanel::left("summary_panel").show(ctx, |ui| {
            summary::draw_summary(ui);
        });

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                tree::draw_tree(&mut columns[0]);
                detail::draw_detail(&mut columns[1]);
            });
        });
    }
}

impl eframe::App for PngMetadataCompareApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.render_scaffold(ctx);
    }
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

    #[test]
    fn scaffold_render_produces_output() {
        let mut app = PngMetadataCompareApp::default();
        let ctx = eframe::egui::Context::default();
        let output = ctx.run(Default::default(), |ctx| {
            app.render_scaffold(ctx);
        });

        assert!(!output.shapes.is_empty());
    }
}
