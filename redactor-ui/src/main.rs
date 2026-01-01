use eframe::egui;

mod views {
    pub mod editor;
    pub mod home;
}

mod renders {
    pub mod image_renderer;
    pub mod pdf_renderer;
}

use redactor_core::utils::logging;
use views::editor::EditorPage;
use views::home::HomePage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppView {
    Home,
    Editor,
}

fn main() -> Result<(), eframe::Error> {
    logging::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    eframe::run_native(
        "Redactatron 9000",
        options,
        Box::new(|cc| {
            let ctx = cc.egui_ctx.clone();

            let mut style = (*ctx.style()).clone();
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(28.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Small,
                egui::FontId::new(12.0, egui::FontFamily::Proportional),
            );
            ctx.set_style(style);

            let mut visuals = egui::Visuals::dark();
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(32, 34, 37);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 60, 65);
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(24, 24, 28);
            ctx.set_visuals(visuals);

            Ok(Box::new(RedactorApp::default()))
        }),
    )
}

struct RedactorApp {
    current_view: AppView,
    home_page: HomePage,
    editor_page: Option<EditorPage>,
    show_about: bool,
}

impl Default for RedactorApp {
    fn default() -> Self {
        Self {
            current_view: AppView::Home,
            home_page: HomePage::default(),
            editor_page: None,
            show_about: false,
        }
    }
}

impl eframe::App for RedactorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // Top panel with title and about button
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("Redactatron 9000").size(40.0));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("About").clicked() {
                    self.show_about = !self.show_about;
                }

                // Add back button when in editor view
                if matches!(self.current_view, AppView::Editor)
                    && ui.button("← Back to Home").clicked()
                {
                    self.current_view = AppView::Home;
                    self.editor_page = None;
                }
            });
        });

        // Render current view
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                AppView::Home => {
                    if let Some(files) = self.home_page.update(ui, ctx) {
                        // User submitted files, switch to editor
                        self.editor_page = Some(EditorPage::new(files));
                        self.current_view = AppView::Editor;
                    }
                }
                AppView::Editor => {
                    if let Some(editor) = &mut self.editor_page {
                        editor.update(ui, ctx);
                    }
                }
            }
        });

        // About window
        if self.show_about {
            egui::Window::new("About Redactatron")
                .movable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label("Redactatron v0.1.0");
                    ui.separator();
                    ui.label("A redaction tool built with Rust and egui.");
                    ui.separator();
                    ui.label("Features:");
                    ui.label("  • Drag and drop file upload");
                    ui.label("  • Document viewing and redaction");
                    ui.label("  • Keyword search and highlight");
                    ui.label("  • Cross-platform support");
                    ui.separator();
                    ui.label("Author: William Green");
                    ui.label("License: MIT");
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.show_about = false;
                    }
                });
        }
    }
}
