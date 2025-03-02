use std::thread;

use async_std::channel::Sender;
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
use icepaper::{Config, get_config_path, read_config};
use notify::{RecursiveMode, Watcher};

struct App {
    background: Option<String>,
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    WatchConfig(Sender<Config>),
    UpdateBackground(String),
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WatchConfig(sender) => {
                thread::spawn(move || {
                    let (tx, rx) = std::sync::mpsc::channel();
                    let mut watcher = notify::recommended_watcher(tx).unwrap();
                    watcher
                        .watch(&get_config_path().unwrap(), RecursiveMode::NonRecursive)
                        .unwrap();
                    let maybe_config = read_config();
                    if let Some(config) = maybe_config {
                        sender.send_blocking(config).unwrap();
                    }
                    while let Ok(Ok(_)) = rx.recv() {
                        let maybe_config = read_config();
                        if let Some(config) = maybe_config {
                            sender.send_blocking(config).unwrap();
                        }
                    }
                });
                Task::none()
            }
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
            let (tx, rx) = async_std::channel::unbounded();
            output.send(Message::WatchConfig(tx)).await.unwrap();
            while let Ok(config) = rx.recv().await {
                if let Some(background) = config.background {
                    output
                        .send(Message::UpdateBackground(background))
                        .await
                        .unwrap();
                }
            }
        }
    })
}

fn main() -> Result<(), iced_layershell::Error> {
    iced_layershell::build_pattern::application("icepaper", App::update, App::view)
        .subscription(App::subscription)
        .layer_settings(LayerShellSettings {
            layer: Layer::Background,
            anchor: Anchor::all(),
            events_transparent: true,
            ..Default::default()
        })
        .run_with(|| (App { background: None }, Task::none()))
}
