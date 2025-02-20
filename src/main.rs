use iced::widget::{Column, column, text};

#[derive(Default)]
struct State {}

impl State {
    fn update(&mut self, _: ()) {}

    fn view(&self) -> Column<()> {
        column![text("Hello, world!")]
    }
}

fn main() -> iced::Result {
    iced::run("Background Core", State::update, State::view)
}
