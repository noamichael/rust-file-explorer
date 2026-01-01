use crate::app::{Action, ContextMenuAction, FileExplorerApp, PaneContent};

use iced::widget::text::{Rich, Span};
use iced::widget::{center, mouse_area, opaque, pane_grid, rule, scrollable, stack, text_input};
use iced::{Alignment, Element, Theme, border};
use iced::{
    Background, Color, Font, Length, Task,
    font::Weight,
    padding,
    widget::{button, column, container, row, space, span, text},
};
use iced_aw::ContextMenu;

use syntect::easy::HighlightLines;

const HEADING_FONT_SIZE: f32 = 32.0;
const FILE_NAME_FONT_SIZE: f32 = 24.0;

impl FileExplorerApp {
    pub fn update(&mut self, action: Action) -> Task<Action> {
        self.post_update(action)
    }

    pub fn view(&self) -> iced::Element<'_, Action> {
        let grid = pane_grid::PaneGrid::new(&self.panes, |_pane, pc, _focus| {
            let side_bar = container(self.side_bar());
            let content = container(self.file_contents());

            pane_grid::Content::new(match pc {
                PaneContent::Sidebar => side_bar,
                PaneContent::Content => content,
            })
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .on_resize(10, Action::PanesResized);

        let app_content = row![grid].spacing(20.0).into();

        if self.file_info_modal_open {
            let modal_content = self.file_info_modal_content();
            modal(app_content, modal_content, Action::CloseFileInfoModal)
        } else {
            app_content
        }
    }

    fn side_bar(&self) -> iced::Element<'_, Action> {
        let back_button: iced::Element<Action> = button(row![
            text("⬆️ ../")
                .shaping(text::Shaping::Advanced)
                .size(FILE_NAME_FONT_SIZE)
        ])
        .on_press(Action::GoBack())
        .style(file_node_style(false))
        .width(Length::Fill)
        .into();

        let mut file_nodes: Vec<iced::Element<Action>> = Vec::new();

        for (index, f) in self.files.iter().enumerate() {
            if !f.matches_filters {
                continue;
            }

            let file_name_row = text(f.display_name())
                .shaping(text::Shaping::Advanced)
                .size(FILE_NAME_FONT_SIZE);

            let is_selected = match &self.opened_file {
                Some(opened_file) => opened_file.absolute_path == f.absolute_path,
                None => false,
            };

            file_nodes.push(add_context_menu_to(
                index,
                button(file_name_row)
                    .style(file_node_style(is_selected))
                    .on_press(Action::OpenFile(index))
                    .width(Length::Fill)
                    .into(),
            ));
        }

        let left_border = container(text(""))
            .width(2.0) // The "border" width
            .height(Length::Fill)
            .style(|theme: &iced::Theme| container::Style {
                background: Some(theme.extended_palette().background.neutral.color.into()),
                ..Default::default()
            });

        container(
            row![
                column![
                    // Directory name and search bar
                    column![
                        text(self.opened_dir.display_name())
                            .size(HEADING_FONT_SIZE)
                            .font(Font {
                                weight: Weight::Bold,
                                ..Font::default()
                            }),
                        text_input("Search file names", &self.filters.file_name_search)
                            .on_input(Action::DebouncedSearch)
                            .width(Length::Fill),
                    ]
                    .padding(5.0),
                    // File nodes
                    scrollable(column![
                        back_button,
                        iced::widget::Column::from_vec(file_nodes).width(Length::Fill)
                    ]),
                ],
                column![left_border]
            ]
            .width(Length::Fill),
        )
        .into()
    }

    fn file_contents(&self) -> iced::Element<'_, Action> {
        let result = match &self.opened_file {
            Some(opened_file) => match &self.opened_file_contents {
                Ok(contents) => {
                    let ps = &self.highlighting.syntax_set;
                    let ts = &self.highlighting.theme_set;

                    let syntax = ps
                        .find_syntax_by_extension(
                            &self.opened_file_type.clone().unwrap_or(String::from("txt")),
                        )
                        .or(ps.find_syntax_by_extension("txt"))
                        .unwrap();
                    let theme = match &self.system_color_mode {
                        dark_light::Mode::Dark => &ts.themes["base16-ocean.dark"],
                        dark_light::Mode::Light => &ts.themes["Solarized (light)"],
                        dark_light::Mode::Unspecified => &ts.themes["Solarized (light)"],
                    };
                    let mut h = HighlightLines::new(syntax, theme);

                    let lines = contents.lines().collect::<Vec<&str>>();
                    let line_number_digits = lines.len().to_string().len();

                    let highlighted = iced::widget::Column::with_children(
                        lines
                            .iter()
                            .enumerate()
                            .map(|(index, line)| {
                                let spans = h
                                    .highlight_line(line, &ps)
                                    .unwrap()
                                    .iter()
                                    .map(|(style, text)| {
                                        span(*text)
                                            .color(Color::from_rgb8(
                                                style.foreground.r,
                                                style.foreground.g,
                                                style.foreground.b,
                                            ))
                                            .font(Font::MONOSPACE)
                                    })
                                    .collect::<Vec<Span<String, Font>>>();

                                let rich = Rich::with_spans(spans);
                                row![
                                    text(format!(
                                        "{:width$}",
                                        index + 1,
                                        width = line_number_digits
                                    ))
                                    .font(Font::MONOSPACE),
                                    space::vertical().width(Length::Fixed(15.0)),
                                    rich
                                ]
                            })
                            .map(iced::Element::from)
                            .collect::<Vec<_>>(),
                    );

                    let top_border = container(text(""))
                        .height(2.0) // The "border" width
                        .width(Length::Fill)
                        .style(|theme: &iced::Theme| container::Style {
                            background: Some(
                                theme.extended_palette().background.neutral.color.into(),
                            ),
                            ..Default::default()
                        });

                    column![
                        row![
                            // Opened file name
                            container(text(&opened_file.file_name).size(HEADING_FONT_SIZE).font(
                                Font {
                                    weight: Weight::Bold,
                                    ..Font::default()
                                }
                            ))
                            .padding(padding::left(5.0)),
                            // Empty spave to push the close button to the right
                            space::horizontal().width(Length::Fill),
                            // File Actions
                            container(
                                button("Close")
                                    .on_press(Action::CloseFile)
                                    .style(button::secondary)
                            )
                            .padding(padding::right(5.0))
                        ]
                        .align_y(Alignment::Center),
                        top_border,
                        scrollable(highlighted)
                            .width(Length::Fill)
                            .height(Length::Fill)
                    ]
                    .spacing(10.0)
                }
                Err(e) => {
                    column![
                        text(format!("Error: {}", e))
                            .size(FILE_NAME_FONT_SIZE)
                            .font(Font::MONOSPACE)
                            .color(Color::from_rgb(1.0, 0.0, 0.0))
                    ]
                }
            },
            None => {
                column![
                    text("Please select a file from the menu")
                        .size(24.0)
                        .width(Length::Fill)
                        .center()
                ]
            }
        };

        column!(result).into()
    }

    pub fn file_info_modal_content(&self) -> iced::Element<'_, Action> {
        match &self.file_info_modal_node {
            Some(file) => {
                // Placeholder content for the file info modal
                container(
                    column![
                        text(file.display_name()).size(HEADING_FONT_SIZE).font(Font {
                            weight: Weight::Bold,
                            ..Font::default()
                        }),
                        labeled("Type", if file.is_dir {
                            "Directory"
                        } else {
                            "File"
                        }),
                        labeled("Path", &file.absolute_path),
                        labeled("Size", &file.file_size),
                        labeled("Created At", &file.created_at),
                        labeled("Modified At", &file.modified_at),
                        labeled("Accessed At", &file.accessed_at),
                        rule::horizontal(2.0),
                        row![
                            // Fill space to push the button
                            space::horizontal().width(Length::Fill),
                            button("Close")
                            .on_press(Action::CloseFileInfoModal)
                            .style(button::primary)
                        ].align_y(Alignment::Center)
                    ]
                    .spacing(20.0)
                    .padding(20.0),
                )
                .style(|style: &Theme |{
                    container::Style {
                        background: Some(style.extended_palette().background.base.color.into()),
                        border: border::rounded(5.0),
                        ..Default::default()
                    }
                })
                .into()
            }
            None => column![text("No file info available")].into(),
        }
    }
}

fn file_node_style(selected: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |theme: &iced::Theme, status: button::Status| {
        // Get the base theme color
        let palette = theme.extended_palette();
        // If the file is selected, use the primary button style
        if selected {
            button::primary(theme, status)
        } else {
            // If not selected, use a custom style
            match status {
                // Normal state - do not add any backgroun and use default text
                button::Status::Active | button::Status::Pressed => button::Style {
                    background: Some(Background::Color(palette.background.base.color)),
                    text_color: Color::from_rgb(
                        palette.background.base.text.r,
                        palette.background.base.text.g,
                        palette.background.base.text.b,
                    ),
                    ..button::Style::default()
                },
                // Hovered and disabled states use the primary style
                button::Status::Hovered => button::primary(theme, status),
                button::Status::Disabled => button::primary(theme, status),
            }
        }
    }
}

fn add_context_menu_to(
    index: usize,
    element: iced::Element<'_, Action>,
) -> iced::Element<'_, Action> {
    ContextMenu::new(element, move || {
        // Create a container for the context menu
        container(column![
            button(text("Open"))
                .style(context_menu_button_style())
                .on_press(Action::OpenFile(index)),
            //rule::horizontal(2.0),
            button(text("Get Info"))
                .style(context_menu_button_style())
                .on_press(Action::OpenContextMenu(
                    ContextMenuAction::OpenFileInfoModal(index)
                ))
        ])
        .padding(10.0)
        // Style the context menu background
        .style(|theme: &Theme| {
            return container::Style {
                background: Some(theme.extended_palette().background.weak.color.into()),
                border: border::rounded(2.0),
                ..Default::default()
            };
        })
        .into()
    })
    .into()
}

fn context_menu_button_style() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    // This is a workaround for a bug in iced_aw where the context menu button style is not applied correctly
    // The status is always set to Disabled, so we have to manually handle the different states
    // See issue:
    // https://github.com/iced-rs/iced_aw/issues/378
    move |theme: &iced::Theme, _status: button::Status| {
        // Get the base theme color
        let palette = theme.extended_palette();
        button::Style {
            background: Some(Color::TRANSPARENT.into()),
            text_color: Color::from_rgb(
                palette.background.base.text.r,
                palette.background.base.text.g,
                palette.background.base.text.b,
            ),
            ..button::Style::default()
        }
    }
}

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }).padding(20.0))
            .on_press(on_blur)
        )
    ]
    .into()
}

fn labeled<'a>(label: &'a str, value: &'a str) -> iced::Element<'a, Action> {
    row![
        text(format!("{}: ", label))
            .font(Font {
                weight: Weight::Bold,
                ..Font::default()
            })
            .size(FILE_NAME_FONT_SIZE),
        text(value).size(FILE_NAME_FONT_SIZE)
    ]
    .spacing(10.0)
    .into()
}