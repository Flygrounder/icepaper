use iced::{
    Element, Task,
    widget::{Row, button, column, horizontal_space, image, row},
    window::{self, Settings},
};

struct App {
    editor_window_id: window::Id,
    background_path: Option<String>,
    history: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    Initialize,
    UpdateBackgroundPath(String),
    ResetBackgroundPath,
    OpenFilePicker,
    CloseFilePicker,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Initialize => Task::none(),
            Message::UpdateBackgroundPath(image_path) => {
                if !self.history.contains(&image_path) {
                    self.history.push(image_path.clone());
                }
                self.background_path = Some(image_path);
                Task::none()
            }
            Message::ResetBackgroundPath => {
                self.background_path = None;
                Task::none()
            }
            Message::OpenFilePicker => Task::perform(pick_file(), |path| {
                path.map(Message::UpdateBackgroundPath)
                    .unwrap_or(Message::CloseFilePicker)
            }),
            Message::CloseFilePicker => Task::none(),
        }
    }

    fn view(&self, id: window::Id) -> Element<Message> {
        if id == self.editor_window_id {
            column![
                row![
                    button("Open").on_press(Message::OpenFilePicker),
                    button("Clear").on_press(Message::ResetBackgroundPath)
                ],
                Row::from_vec(
                    self.history
                        .iter()
                        .map(|path| column![
                            image(path).height(100),
                            button("Use").on_press(Message::UpdateBackgroundPath(path.clone()))
                        ]
                        .into())
                        .collect()
                )
            ]
            .into()
        } else {
            horizontal_space().into()
        }
    }
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
    iced::daemon("Icepaper", App::update, App::view).run_with(|| {
        let (editor_window_id, editor_task) = window::open(Settings::default());
        let app = App {
            editor_window_id,
            background_path: None,
            history: Vec::new(),
        };
        (app, editor_task.map(|_| Message::Initialize))
    })
}
