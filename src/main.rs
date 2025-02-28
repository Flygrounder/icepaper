use iced::{
    Border, Color, ContentFit, Element, Task,
    border::radius,
    widget::{Container, Row, button, column, container::Style, horizontal_space, image, row},
};

#[derive(Default)]
struct App {
    background_path: Option<String>,
    history: Vec<String>,
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
                if !self.history.contains(&image_path) {
                    self.history.push(image_path.clone());
                }
                self.background_path = Some(image_path);
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
            .background_path
            .as_ref()
            .map(|path| image_cover(path).into())
            .unwrap_or(horizontal_space().into());
        Container::new(
            column![
                row![button("Open").on_press(Message::OpenFilePicker),],
                background,
                Row::from_vec(
                    self.history
                        .iter()
                        .map(|path| column![
                            image_cover(path),
                            button("Use").on_press(Message::UpdateBackgroundPath(path.clone()))
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
    iced::run("Icepaper", App::update, App::view)
}
