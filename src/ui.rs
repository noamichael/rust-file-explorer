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
            ui.heading(self.opened_dir.display_name());

            ui.horizontal(|ui| {
                // Add text search box
                let file_search = ui.add(
                    egui::TextEdit::singleline(&mut self.filters.file_name_search)
                        .hint_text("Search Files"),
                );

                // On enter key press of the search bar, trigger search action.
                //
                // TODO: Improve this by triggering search after the user is done typing. This would
                // typically be done by "debouncing" the input event. What this means is that we don't want
                // to trigger the search action until the user "pauses" (or stops) typing. This requires
                // being able to schedule "cancelable" tasks, probably via a channel and background thread.
                if file_search.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    action = Action::SearchByFilename(self.filters.file_name_search.clone());
                }
            });

            ui.add(egui::Separator::default().horizontal());

            // Draw the file tree
            egui::ScrollArea::both().auto_shrink(true).show(ui, |ui| {
                // Render back link for directory
                if self.opened_dir.absolute_path != "/" {
                    let back_label = ui.add(egui::Label::new("../").sense(egui::Sense::click()));

                    ui.add(egui::Separator::default().horizontal());

                    if back_label.hovered() {
                        ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    }

                    if back_label.clicked() {
                        action = Action::GoBack();
                    }
                }

                // Build left side file tree
                for (index, node) in self.files.iter().enumerate() {
                    // Skip rendering nodes that don't match the filters
                    if !node.matches_filters {
                        continue;
                    }
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

                    // Add frame for file node
                    ui.push_id(&node.file_name, |ui| {
                        let file_node_frame = egui::Frame::default().show(ui, |ui| {
                            let _file_label = ui.add(
                                egui::Label::new(file_name_text)
                                    .wrap_mode(egui::TextWrapMode::Extend),
                            );

                            ui.add(egui::Separator::default().horizontal());
                        });

                        let frame_rect = file_node_frame.response.rect;

                        // Sense clicks on the background of the *parent* ui, using the frame's rectangle for bounds
                        let bg_response = ui.interact(
                            frame_rect,
                            ui.id().with(&node.file_name),
                            egui::Sense::click(),
                        );

                        if bg_response.clicked() {
                            println!("CLICKED {}", node.file_name);
                            action = Action::OpenFile(index);
                        }

                        if bg_response.hovered() {
                            ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        }
                    });
                }
            });
        });

        // Main window panel
        egui::CentralPanel::default().show(ctx, |ui| {
            //Content that DOES NOT SCROLL

            match &self.opened_file {
                Some(file) => {
                    ui.horizontal(|ui| {
                        ui.heading(&file.file_name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_button = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("Close").color(Color32::WHITE), // Set text color to red
                                )
                                .fill(Color32::DARK_RED),
                            );

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

                                let code_theme = if ctx.style().visuals.dark_mode {
                                    egui_extras::syntax_highlighting::CodeTheme::dark(12.0)
                                } else {
                                    egui_extras::syntax_highlighting::CodeTheme::light(12.0)
                                };

                                let layout_job = egui_extras::syntax_highlighting::highlight_with(
                                    ui.ctx(),
                                    ui.style(),
                                    &code_theme,
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
