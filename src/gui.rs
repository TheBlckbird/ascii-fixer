use std::path::PathBuf;

use iced::{
    Alignment::Center,
    Color, Element,
    Length::{Fill, Shrink},
    widget::{
        Space, bottom, button, center, column, container, mouse_area, opaque, rich_text, row,
        scrollable, span, stack, text,
    },
    window::{self, icon},
};
use rfd::FileDialog;

use crate::{
    fix_file::{FixFileError, fix_file},
    strip_home::path_to_human_readable,
};

#[derive(Debug, Clone)]
enum Message {
    PickFiles,
    PickFolders,
    FixFiles,
    ShowFixFilesDialog,
    HideFixFilesDialog,
    HideErrorModal,
    LinkClicked(String),
    RemoveFile(PathBuf),
    RemoveAll,
}

#[derive(Default)]
struct AsciiFixer {
    files: Vec<PathBuf>,
    show_dialog: bool,
    error_modals: Vec<String>,
    is_finished: bool,
}

impl AsciiFixer {
    fn update(&mut self, message: Message) {
        match message {
            Message::PickFiles => {
                #[cfg(target_os = "macos")]
                let files = FileDialog::new().pick_files_or_folders();

                #[cfg(not(target_os = "macos"))]
                let files = FileDialog::new().pick_files();

                self.add_files(files);
            }

            Message::PickFolders => {
                let folders = FileDialog::new().pick_folders();
                self.add_files(folders);
            }

            Message::FixFiles => {
                for file in &self.files {
                    if let Err(error) = fix_file(file) {
                        match error {
                            FixFileError::Io(error) => {
                                self.error_modals.push(format!(
                                    "Es ist ein Fehler bei dem Schreiben oder Lesen der Datei '{}' aufgetreten:\n{error}",
                                    file.display(),
                                ));
                            }
                            FixFileError::InvalidFilename => self
                                .error_modals
                                .push(format!("Pfad '{}' existiert nicht", file.display())),
                        }
                    }
                }

                self.files.clear();

                self.show_dialog = false;
                self.is_finished = true;
            }

            Message::ShowFixFilesDialog => self.show_dialog = true,
            Message::HideFixFilesDialog => self.show_dialog = false,
            Message::HideErrorModal => self.error_modals.clear(),

            Message::LinkClicked(link) => {
                let _ = open::that(link);
            }

            Message::RemoveFile(file_remove) => {
                if let Some(position) = self.files.iter().position(|file| *file == file_remove) {
                    self.files.remove(position);
                }
            }

            Message::RemoveAll => self.files.clear(),
        }
    }

    fn add_files(&mut self, files: Option<Vec<PathBuf>>) {
        if let Some(mut files) = files {
            self.files.append(&mut files);
        }

        self.is_finished = false;
    }

    fn view(&self) -> Element<'_, Message> {
        let files_list = match self.files.len() {
            0 => container(text(if self.is_finished {
                "Fertig!".to_string()
            } else {
                "Keine Dateien ausgewählt".to_string()
            })),
            _ => {
                let mut column = column![].spacing(5).padding(12);

                for file in self.files.iter() {
                    column = column.push(
                        row![
                            button("X")
                                .on_press(Message::RemoveFile(file.to_path_buf()))
                                .style(button::danger),
                            text(path_to_human_readable(file)),
                        ]
                        .spacing(10)
                        .align_y(Center),
                    );
                }

                container(scrollable(column).height(Fill))
            }
        };

        let select_files = if cfg!(target_os = "macos") {
            container(button("Dateien auswählen").on_press(Message::PickFiles))
        } else {
            container(
                row![
                    button("Dateien auswählen").on_press(Message::PickFiles),
                    button("Ordner auswählen").on_press(Message::PickFolders)
                ]
                .spacing(10),
            )
        };

        let base_interface = container(
            column![
                column![
                    column![
                        select_files,
                        button("Auswahl leeren")
                            .style(button::danger)
                            .on_press(Message::RemoveAll),
                    ]
                    .spacing(10)
                    .align_x(Center),
                    files_list,
                    button("Dateien fixen").on_press(Message::ShowFixFilesDialog),
                ]
                .padding(13)
                .spacing(40)
                .height(Fill)
                .align_x(Center),
                bottom(
                    row![
                        rich_text![
                            span(format!("v{}", env!("CARGO_PKG_VERSION"))).link(
                                "https://github.com/TheBlckbird/ascii-fixer/releases/latest"
                                    .to_string()
                            )
                        ]
                        .on_link_click(Message::LinkClicked),
                        Space::new().width(Fill),
                        rich_text![
                            span("© Louis Weigel").link("https://louisweigel.com".to_string())
                        ]
                        .on_link_click(Message::LinkClicked)
                    ]
                    .width(Fill)
                )
                .height(Shrink)
            ]
            .padding(7)
            .width(Fill)
            .height(Fill)
            .align_x(Center),
        );

        if self.show_dialog {
            if self.files.is_empty() {
                let information_modal = container(
                    column![
                        text("Du hast keine Dateien ausgewählt\n").align_x(Center),
                        row![button("Ok").on_press(Message::HideFixFilesDialog)].spacing(30)
                    ]
                    .align_x(Center),
                );

                modal(
                    base_interface,
                    information_modal,
                    Message::HideFixFilesDialog,
                )
            } else {
                let confirmation_dialog = container(column![
                    text("Willst du das wirklich durchführen?\nDiese Aktion wird alle ausgewählten Dateien überschreiben!\n").align_x(Center),
                    row![
                        button("Abbrechen").on_press(Message::HideFixFilesDialog),
                        button("Fortfahren").on_press(Message::FixFiles)
                    ].spacing(30)
                ].align_x(Center));

                modal(
                    base_interface,
                    confirmation_dialog,
                    Message::HideFixFilesDialog,
                )
            }
        } else if !self.error_modals.is_empty() {
            let information_modal = container(
                column![
                    text(format!("{}\n", self.error_modals.join("\n"))).align_x(Center),
                    row![button("Ok").on_press(Message::HideErrorModal)].spacing(30)
                ]
                .align_x(Center),
            );

            modal(base_interface, information_modal, Message::HideErrorModal)
        } else {
            base_interface.into()
        }
    }
}

pub fn show_gui() -> iced::Result {
    let window_settings = window::Settings {
        icon: Some(
            icon::from_file_data(include_bytes!("../assets/icon1024.png"), None)
                .expect("Icon should be valid"),
        ),
        ..window::Settings::default()
    };

    iced::application(AsciiFixer::default, AsciiFixer::update, AsciiFixer::view)
        .title("ASCII Fixer")
        .window(window_settings)
        .window_size((800, 500))
        .run()
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
                    text_color: Some(Color::WHITE),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}
