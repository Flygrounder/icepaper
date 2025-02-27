use iced::{
    Element, Task,
    platform_specific::shell::commands::layer_surface::{Anchor, get_layer_surface},
    runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings,
    widget::{button, horizontal_space, image, row},
    window::{self, Mode, Settings, change_mode},
};

struct App {
    editor_window_id: window::Id,
    background_surface_id: window::Id,
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
            Message::Initialize => change_mode(self.background_surface_id, Mode::Fullscreen),
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
        } else if id == self.background_surface_id {
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
        let background_surface_id = window::Id::unique();
        let background_task = get_layer_surface(SctkLayerSurfaceSettings {
            id: background_surface_id,
            layer: iced::platform_specific::shell::commands::layer_surface::Layer::Bottom,
            anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
            ..Default::default()
        });
        let (editor_window_id, editor_task) = window::open(Settings::default());
        let app = App {
            editor_window_id,
            background_surface_id,
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
