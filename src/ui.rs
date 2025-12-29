use crate::app::{Action, FileExplorerApp};
use egui::Color32;
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

/// Implement the eframe App trait (interface) for the FileExplorerApp
impl eframe::App for FileExplorerApp {
    /// Draws the UI for the given frame. This is called for each frame.
    /// This function should not mutate any state so as to avoid borrow issues.
    ///
    /// # Arguments
    ///
    /// * `self` - The application instance
    /// * `ctx` - The drawing context
    /// * `_frame` - The frame being drawn (unused)
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // The action performed during this frame.
        let mut action = Action::None;

        // Set Styles
        ctx.style_mut(|style| {
            style
                .text_styles
                .get_mut(&egui::TextStyle::Heading)
                .unwrap()
                .size = 32.0;
            style
                .text_styles
                .get_mut(&egui::TextStyle::Body)
                .unwrap()
                .size = 24.0;
        });

        // Left navigation tree
        egui::SidePanel::left("file_explorer").show(ctx, |ui| {
            ui.heading(&self.opened_dir.display_name());
            ui.add(egui::Separator::default().horizontal());

            // Draw the file tree
            egui::ScrollArea::vertical()
                .auto_shrink(true)
                .show(ui, |ui| {
                    // Render back link for directory
                    if self.opened_dir.absolute_path != "/" {
                        let back_label =
                            ui.add(egui::Label::new("../").sense(egui::Sense::click()));

                        ui.add(egui::Separator::default().horizontal());

                        if back_label.hovered() {
                            ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        }

                        if back_label.clicked() {
                            action = Action::GoBack(self.opened_dir.clone());
                        }
                    }

                    // Build left side file tree
                    for node in &self.files {
                        let gui_file_name = node.display_name();

                        let mut file_name_text = egui::RichText::new(gui_file_name);

                        // Draw selected file
                        match &self.opened_file {
                            Some(opened_file) => {
                                if opened_file.absolute_path == node.absolute_path {
                                    file_name_text = file_name_text
                                        .underline()
                                        .background_color(Color32::LIGHT_BLUE)
                                        .color(Color32::BLACK);
                                }
                            }
                            None => {
                                // do nothing
                            }
                        }

                        let file_label =
                            ui.add(egui::Label::new(file_name_text).sense(egui::Sense::click()));

                        ui.add(egui::Separator::default().horizontal());

                        if file_label.hovered() {
                            ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        }

                        if file_label.clicked() {
                            println!("CLICKED {}", node.file_name);
                            action = Action::OpenFile(node.clone());
                        }
                    }
                });
        });

        // Main window panel
        egui::CentralPanel::default().show(ctx, |ui| {
            //Content that DOES NOT SCROLL

            match &self.opened_file {
                Some(file) => {
                    ui.horizontal(|ui| {
                        ui.heading(format!("{}", file.file_name));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_button =
                                ui.add(egui::Button::new("Close").fill(Color32::DARK_RED));
                            if close_button.clicked() {
                                action = Action::CloseFile;
                            }
                        });
                    });
                }
                None => {
                    ui.heading(String::from("No File Opened"));
                }
            };

            ui.add(egui::Separator::default().horizontal());

            let ps = SyntaxSet::load_defaults_newlines();
            let ts = ThemeSet::load_defaults();
            let syntax = egui_extras::syntax_highlighting::SyntectSettings { ps, ts };

            // Scrolling text content
            egui::ScrollArea::vertical()
                .auto_shrink(true)
                .show(ui, |ui| {
                    match &self.opened_file {
                        Some(_) => match &self.opened_file_contents {
                            Ok(contents) => {
                                let file_type = &self.opened_file_type.as_ref();
                                let layout_job = egui_extras::syntax_highlighting::highlight_with(
                                    ui.ctx(),
                                    ui.style(),
                                    &egui_extras::syntax_highlighting::CodeTheme::default(),
                                    contents,
                                    file_type.unwrap_or(&String::from("text")),
                                    &syntax,
                                );

                                ui.add(egui::Label::new(layout_job).selectable(true));
                            }
                            Err(e) => {
                                let error = egui::RichText::new(format!("Error: {}", e))
                                    .color(Color32::RED);
                                ui.add(egui::Label::new(error));
                            }
                        },
                        None => {
                            ui.add(egui::Label::new(String::from(
                                "Please select a file from the menu",
                            )));
                        }
                    };
                });
        });

        // Handle any actions that occurred during this frame
        let _ = self.post_update(action);
    }
}
