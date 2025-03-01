use std::{
    sync::mpsc::{Sender, channel},
    thread,
};

use iced::{
    Border, Color, ContentFit, Element, Task,
    border::radius,
    widget::{Container, Row, button, column, container::Style, horizontal_space, image, row},
};
use icepaper::read_config;
use icepaper::{Config, write_config};

struct App {
    config: Config,
    tx_config: Sender<Config>,
}

#[derive(Debug, Clone)]
enum Message {
    UpdateBackgroundPath(String),
    OpenFilePicker,
    CloseFilePicker,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateBackgroundPath(image_path) => {
                if !self.config.gallery.contains(&image_path) {
                    self.config.gallery.push(image_path.clone());
                }
                self.config.background = Some(image_path);
                self.tx_config.send(self.config.clone()).unwrap();
                Task::none()
            }
            Message::OpenFilePicker => Task::perform(pick_file(), |path| {
                path.map(Message::UpdateBackgroundPath)
                    .unwrap_or(Message::CloseFilePicker)
            }),
            Message::CloseFilePicker => Task::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        let background: Element<Message> = self
            .config
            .background
            .as_ref()
            .map(|path| image_cover(path))
            .unwrap_or(horizontal_space().into());
        Container::new(
            column![
                row![button("Open").on_press(Message::OpenFilePicker),],
                background,
                Row::from_vec(
                    self.config
                        .gallery
                        .iter()
                        .map(|path| column![
                            image_cover(path),
                            button("Use").on_press(Message::UpdateBackgroundPath(path.into()))
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

fn image_cover(path: &str) -> Element<Message> {
    Container::new(
        image(path)
            .width(200)
            .height(100)
            .content_fit(ContentFit::Cover),
    )
    .padding(4)
    .style(|_| Style {
        border: Border {
            width: 4.0,
            radius: radius(0),
            color: Color::BLACK,
        },
        ..Default::default()
    })
    .into()
}

async fn pick_file() -> Option<String> {
    rfd::AsyncFileDialog::new()
        .add_filter("Supported files", &["png"])
        .pick_file()
        .await
        .as_ref()
        .map(|handle| handle.path().to_string_lossy().into())
}

fn main() -> iced::Result {
    let (tx_config, rx_config) = channel();
    let config = read_config().unwrap_or_default();
    thread::spawn(move || {
        while let Ok(config) = rx_config.recv() {
            write_config(&config).unwrap();
        }
    });
    iced::application("Icepaper", App::update, App::view)
        .run_with(|| (App { config, tx_config }, Task::none()))
}
