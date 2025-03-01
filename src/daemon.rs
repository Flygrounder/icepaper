use std::time::Duration;

use async_std::task;
use iced::{
    Element, Subscription, Task,
    futures::{SinkExt, Stream},
    stream,
    widget::{horizontal_space, image},
};
use iced_layershell::{
    reexport::{Anchor, Layer},
    settings::LayerShellSettings,
    to_layer_message,
};
use icepaper::{Config, read_config};

struct App {
    background: Option<String>,
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    UpdateBackground(String),
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateBackground(background) => {
                self.background = Some(background);
                Task::none()
            }
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Element<Message> {
        self.background
            .as_ref()
            .map(|path| image(path).into())
            .unwrap_or(horizontal_space().into())
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(file_watcher)
    }
}

fn file_watcher() -> impl Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        loop {
            task::sleep(Duration::from_secs(1)).await;
            let maybe_config = read_config();
            if let Some(Config {
                background: Some(background),
                ..
            }) = maybe_config
            {
                output
                    .send(Message::UpdateBackground(background))
                    .await
                    .unwrap();
            }
        }
    })
}

fn main() -> Result<(), iced_layershell::Error> {
    iced_layershell::build_pattern::application("icepaper", App::update, App::view)
        .subscription(App::subscription)
        .layer_settings(LayerShellSettings {
            layer: Layer::Bottom,
            anchor: Anchor::all(),
            ..Default::default()
        })
        .run_with(|| (App { background: None }, Task::none()))
}
