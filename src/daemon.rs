use std::thread;

use async_std::channel::Sender;
use bytemuck::bytes_of;
use iced::{
    Element, Subscription, Task,
    futures::{SinkExt, Stream},
    mouse, stream,
    widget::{
        Shader, horizontal_space,
        shader::{
            self, Primitive, Program,
            wgpu::{
                self, BufferDescriptor, BufferUsages, FragmentState, IndexFormat, MultisampleState,
                Operations, PipelineLayout, PrimitiveState, RenderPassColorAttachment,
                RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
                TextureDescriptor, VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
                util::{BufferInitDescriptor, DeviceExt, TextureDataOrder},
                vertex_attr_array,
            },
        },
    },
};
use iced_layershell::{
    reexport::{Anchor, Layer},
    settings::LayerShellSettings,
    to_layer_message,
};
use icepaper::{Config, get_config_path, read_config};
use image::ImageReader;
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
            .map(|path| Shader::new(Scene {}).into())
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

struct Scene {}

impl Program<Message> for Scene {
    type State = ();

    type Primitive = ScenePrimitive;

    fn draw(
        &self,
        state: &Self::State,
        cursor: mouse::Cursor,
        bounds: iced::Rectangle,
    ) -> Self::Primitive {
        ScenePrimitive {}
    }
}

#[derive(Debug)]
struct ScenePrimitive {}

const INDEXES: [[u32; 3]; 2] = [[0, 1, 3], [1, 2, 3]];
const VERTEXES: [[f32; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [1.0, 1.0, 0.0],
    [1.0, 0.0, 0.0],
];

impl Primitive for ScenePrimitive {
    fn prepare(
        &self,
        device: &shader::wgpu::Device,
        queue: &shader::wgpu::Queue,
        format: shader::wgpu::TextureFormat,
        storage: &mut shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &shader::Viewport,
    ) {
        if !storage.has::<RenderData>() {
            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("vertex buffer"),
                usage: BufferUsages::VERTEX,
                contents: bytes_of(&VERTEXES),
            });
            let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("index buffer"),
                usage: BufferUsages::INDEX,
                contents: bytes_of(&INDEXES),
            });
            let module = device.create_shader_module(include_wgsl!("shader.wgsl"));
            let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("pipeline"),
                layout: None,
                vertex: VertexState {
                    module: &module,
                    entry_point: "vs_main",
                    buffers: &[VertexBufferLayout {
                        step_mode: VertexStepMode::Vertex,
                        array_stride: 12,
                        attributes: &vertex_attr_array![0 => Float32x3],
                    }],
                },
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module: &module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });
            let data = RenderData {
                vertex_buffer,
                index_buffer,
                pipeline,
            };
            storage.store(data);
        }
    }

    fn render(
        &self,
        encoder: &mut shader::wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &shader::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        let data = storage.get::<RenderData>().unwrap();
        let mut render_pass = encoder.begin_render_pass(&shader::wgpu::RenderPassDescriptor {
            label: Some("pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                ops: Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                resolve_target: None,
                view: &target,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&data.pipeline);
        render_pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(data.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..2, 0, 0..1);
    }
}

struct RenderData {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    pipeline: RenderPipeline,
}

fn main() -> Result<(), iced_layershell::Error> {
    iced_layershell::build_pattern::application("icepaper", App::update, App::view)
        .subscription(App::subscription)
        .layer_settings(LayerShellSettings {
            layer: Layer::Background,
            anchor: Anchor::all(),
            ..Default::default()
        })
        .run_with(|| (App { background: None }, Task::none()))
}
