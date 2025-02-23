use iced::{
    Element, Task,
    widget::{Column, button, image},
};

#[derive(Default)]
struct State {
    image_path: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    UpdateImagePath(Option<String>),
    OpenFilePicker,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::UpdateImagePath(image_path) => {
            if image_path.is_some() {
                state.image_path = image_path;
            }
            Task::none()
        }
        Message::OpenFilePicker => Task::perform(pick_file(), Message::UpdateImagePath),
    }
}

fn view(state: &State) -> Element<Message> {
    let submit = button("Open").on_press(Message::OpenFilePicker).into();
    let background = state.image_path.as_ref().map(image).map(Into::into);
    Column::with_children([submit].into_iter().chain(background)).into()
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
    iced::run("Icepaper", update, view)
}
