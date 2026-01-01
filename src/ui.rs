use crate::app::{Action, FileExplorerApp};

use iced::widget::scrollable;
use iced::widget::text::{Rich, Span};
use iced::{
    Background, Color, Font, Length,
    font::Weight,
    widget::{button, column, container, row, space, span, text},
};

use syntect::easy::HighlightLines;
use syntect::util::LinesWithEndings;
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

const HEADING_FONT_SIZE: f32 = 32.0;
const FILE_NAME_FONT_SIZE: f32 = 24.0;

impl FileExplorerApp {
    pub fn update(&mut self, action: Action) {
        let _ = self.post_update(action);
    }
    pub fn view(&self) -> iced::Element<'_, Action> {
        let side_bar = container(self.side_bar());
        let content = container(self.file_contents());

        row![
            side_bar.width(Length::FillPortion(1)),
            content.width(Length::FillPortion(4)),
        ]
        .spacing(20.0)
        .into()
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
            let file_name_row = text(f.display_name())
                .shaping(text::Shaping::Advanced)
                .size(FILE_NAME_FONT_SIZE);

            let is_selected = match &self.opened_file {
                Some(opened_file) => opened_file.absolute_path == f.absolute_path,
                None => false,
            };

            file_nodes.push(
                button(file_name_row)
                    .style(file_node_style(is_selected))
                    .on_press(Action::OpenFile(index))
                    .width(Length::Fill)
                    .into(),
            );
        }

        container(
            column![
                text(self.opened_dir.display_name())
                    .size(HEADING_FONT_SIZE)
                    .font(Font {
                        weight: Weight::Bold,
                        ..Font::default()
                    }),
                scrollable(column![
                    back_button,
                    iced::widget::Column::from_vec(file_nodes).width(Length::Fill)
                ])
            ]
            .width(Length::Fill),
        )
        .into()
    }

    fn file_contents(&self) -> iced::Element<'_, Action> {
        let result = match &self.opened_file {
            Some(opened_file) => match &self.opened_file_contents {
                Ok(contents) => {
                    let ps = SyntaxSet::load_defaults_newlines();
                    let ts = ThemeSet::load_defaults();
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

                    let highlighted = iced::widget::Column::with_children(
                        LinesWithEndings::from(contents)
                            .enumerate()
                            .map(|(index, line)| {
                                let spans = h
                                    .highlight_line(line, &ps)
                                    .unwrap()
                                    .iter()
                                    .map(|(style, text)| {
                                        span(*text)
                                            .color(Color::from_rgb(
                                                style.foreground.r as f32 / 255.0,
                                                style.foreground.g as f32 / 255.0,
                                                style.foreground.b as f32 / 255.0,
                                            ))
                                            .font(Font::MONOSPACE)
                                    })
                                    .collect::<Vec<Span<String, Font>>>();

                                let rich = Rich::with_spans(spans);
                                row![
                                    text(format!("{:4}", index + 1)).font(Font::MONOSPACE),
                                    space::vertical().width(Length::Fixed(15.0)),
                                    rich
                                ]
                            })
                            .map(iced::Element::from)
                            .collect::<Vec<_>>(),
                    );

                    column![
                        row![
                            text(&opened_file.file_name)
                                .size(HEADING_FONT_SIZE)
                                .font(Font {
                                    weight: Weight::Bold,
                                    ..Font::default()
                                }),
                            space::horizontal().width(Length::Fill),
                            container(
                                button("Close")
                                    .on_press(Action::CloseFile)
                                    .style(button::secondary)
                            )
                            .padding(10.0)
                        ],
                        scrollable(highlighted)
                            .width(Length::Fill)
                            .height(Length::Fill)
                    ]
                    .spacing(20.0)
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
