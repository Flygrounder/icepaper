use iced::widget::{Column, button, column, image, row, text, text_input};

#[derive(Default)]
struct State {
    form: String,
    path: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(String),
    Save,
}

impl State {
    fn update(&mut self, show: Message) {
        match show {
            Message::Edit(text) => self.form = text,
            Message::Save => self.path = Some(self.form.clone()),
        }
    }

    fn view(&self) -> Column<Message> {
        let input = text_input("background path", &self.form).on_input(|text| Message::Edit(text));
        let submit = button("submit").on_press(Message::Save);
        let form = row![input, submit];
        let background = self.path.as_ref().map(|path| image(path));
        match background {
            Some(background) => column![form, background],
            None => column![form],
        }
    }
}

fn main() -> iced::Result {
    iced::run("Background Core", State::update, State::view)
}
