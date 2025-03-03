use iced::{
    Border, Color, ContentFit, Element, Task,
    border::radius,
    widget::{
        Container, Row, button, column, container::Style, image, row,
    },
};
use icepaper::read_config;
use icepaper::{Config, write_config};

struct App {
    config: Config,
}

#[derive(Debug, Clone)]
enum Message {
    ChangeCurrentBackground(String),
    AddBackground(String),
    RemoveBackground(String),
    OpenFilePicker,
    CloseFilePicker,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangeCurrentBackground(background) => {
                self.config.background = Some(background);
                write_config(&self.config).unwrap();
                Task::none()
            }
            Message::AddBackground(background) => {
                if !self.config.gallery.contains(&background) {
                    self.config.gallery.push(background.clone());
                }
                write_config(&self.config).unwrap();
                Task::none()
            }
            Message::RemoveBackground(background) => {
                self.config.gallery = self
                    .config
                    .gallery
                    .clone()
                    .into_iter()
                    .filter(|path| *path != background)
                    .collect();
                if self
                    .config
                    .background
                    .as_ref()
                    .is_some_and(|path| *path == background)
                {
                    self.config.background = self.config.gallery.first().cloned();
                }
                write_config(&self.config).unwrap();
                Task::none()
            }
            Message::OpenFilePicker => Task::perform(pick_file(), |path| {
                path.map(Message::AddBackground)
                    .unwrap_or(Message::CloseFilePicker)
            }),
            Message::CloseFilePicker => Task::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        Container::new(
            column![
                row![button("Add").on_press(Message::OpenFilePicker),],
                Row::from_vec(
                    self.config
                        .gallery
                        .iter()
                        .map(|path| column![
                            image_cover(
                                path,
                                self.config
                                    .background
                                    .as_ref()
                                    .is_some_and(|current| current == path)
                            ),
                            row![
                                button("Use")
                                    .on_press(Message::ChangeCurrentBackground(path.into())),
                                button("Delete").on_press(Message::RemoveBackground(path.into())),
                            ]
                            .spacing(5)
                        ]
                        .spacing(10)
                        .into())
                        .collect()
                )
                .spacing(20)
            ]
            .spacing(20),
        )
        .padding(30)
        .into()
    }
}

fn image_cover(path: &str, is_highlighted: bool) -> Element<Message> {
    let color = if is_highlighted {
        Color::from_rgb(0.0, 0.0, 1.0)
    } else {
        Color::BLACK
    };
    Container::new(
        image(path)
            .width(200)
            .height(100)
            .content_fit(ContentFit::Cover),
    )
    .padding(4)
    .style(move |_| Style {
        border: Border {
            width: 4.0,
            radius: radius(0),
            color,
        },
        ..Default::default()
    })
    .into()
}

async fn pick_file() -> Option<String> {
    rfd::AsyncFileDialog::new()
        .add_filter("Supported files", &["png", "jpg"])
        .pick_file()
        .await
        .as_ref()
        .map(|handle| handle.path().to_string_lossy().into())
}

fn main() -> iced::Result {
    let config = read_config().unwrap_or_default();
    iced::application("Icepaper", App::update, App::view)
        .run_with(|| (App { config }, Task::none()))
}
