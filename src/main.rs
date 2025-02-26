use iced::{
    Element, Task,
    widget::{button, horizontal_space, image, row},
    window::{self, Mode, Settings, change_mode},
};

struct App {
    editor_window_id: window::Id,
    background_window_id: window::Id,
    background_path: Option<String>,
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
            Message::Initialize => change_mode(self.background_window_id, Mode::Fullscreen),
            Message::UpdateBackgroundPath(image_path) => {
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
            row![
                button("Open").on_press(Message::OpenFilePicker),
                button("Clear").on_press(Message::ResetBackgroundPath)
            ]
            .into()
        } else if id == self.background_window_id {
            self.background_path
                .as_ref()
                .map(image)
                .map(Into::into)
                .unwrap_or_else(|| horizontal_space().into())
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
        let (background_window_id, background_task) = window::open(Settings::default());
        let app = App {
            editor_window_id,
            background_window_id,
            background_path: None,
        };
        (
            app,
            editor_task
                .chain(background_task)
                .map(|_| Message::Initialize),
        )
    })
}
