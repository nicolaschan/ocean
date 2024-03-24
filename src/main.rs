use std::thread;

use fontdue::{Font, FontSettings};
use nannou::App;
use ocean::config::config;
use ocean::render::vertex::{vertices_as_bytes, Vertex, VERTICES};
use ocean::render::wgpu::render_on_device;
use ocean::shell::shell::get_shell;
use ocean::{
    app::state::TerminalState,
    shell::{event::ShellEvent, shell},
};
use tokio::sync::mpsc;
use tracing::{debug, error};

use ::wgpu::Buffer;
use nannou::prelude::*;

pub struct Model {
    shell_event_receiver: std::sync::mpsc::Receiver<ShellEvent>,
    shell_event_sender: mpsc::UnboundedSender<String>,
    state: TerminalState,
    font: Font,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: Buffer,
}

fn model(app: &App) -> Model {
    let config = config::read_config().expect("Failed to read config file");
    error!("Config: {:?}", config);

    let window_id = app
        .new_window()
        .title(format!("{} — Ocean", get_shell(&config)))
        .transparent(true)
        .size(1200, 600)
        .event(event)
        .view(view)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let device = window.device();
    let format = Frame::TEXTURE_FORMAT;
    let sample_count = window.msaa_samples();

    let (event_sink, shell_event_receiver) = std::sync::mpsc::channel();
    let (shell_tx, shell_rx) = mpsc::unbounded_channel();

    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime");

        runtime.block_on(async move {
            let mut child = shell::spawn_shell(&config, event_sink.clone(), shell_rx).await;
            let _ = child
                .wait()
                .await
                .expect("Failed to wait for shell process");
            debug!("Shell process exited");
            event_sink
                .send(ShellEvent::ProcessExited)
                .expect("Failed to send shell event");
        });
    });

    let font = include_bytes!("/usr/share/fonts/TTF/CascadiaCode.ttf");
    let font = Font::from_bytes(&font[..], FontSettings::default()).unwrap();

    let vs_desc = ::wgpu::include_wgsl!("render/shaders/vs.wgsl");
    let fs_desc = ::wgpu::include_wgsl!("render/shaders/fs.wgsl");
    let vs_mod = device.create_shader_module(vs_desc);
    let fs_mod = device.create_shader_module(fs_desc);

    // Create vertex buffer
    let vertices_bytes = vertices_as_bytes(&VERTICES[..]);
    let usage = wgpu::BufferUsages::VERTEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage,
    });

    // Render pipeline
    let bind_group_layout = wgpu::BindGroupLayoutBuilder::new().build(window.device());
    let bind_group = wgpu::BindGroupBuilder::new().build(device, &bind_group_layout);
    let pipeline_layout = wgpu::create_pipeline_layout(device, None, &[&bind_group_layout], &[]);
    let render_pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
        .fragment_shader(&fs_mod)
        .color_format(format)
        .add_vertex_buffer::<Vertex>(&::wgpu::vertex_attr_array![0 => Float32x2])
        .sample_count(sample_count)
        .build(device);

    Model {
        shell_event_receiver,
        shell_event_sender: shell_tx,
        state: TerminalState::new(),
        font,
        bind_group,
        render_pipeline,
        vertex_buffer,
    }
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    // Event handling here
    match event {
        ReceivedCharacter(c) => {
            model.shell_event_sender.send(c.to_string()).unwrap();
        }
        KeyPressed(key) => {
            if key == Key::Return {
                model.shell_event_sender.send("\n".to_string()).unwrap();
            }
        }
        _ => {}
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    while let Ok(event) = model.shell_event_receiver.try_recv() {
        if event == ShellEvent::ProcessExited {
            app.quit();
        }
        model.state.consume(event);
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);

    let window = app.main_window();

    draw.background().color(rgba(0.0, 0.0, 0.0, 0.8));
    draw.text(&model.state.get_as_string())
        .color(rgb(1.0, 1.0, 1.0))
        .left_justify()
        .align_text_top()
        .wh(window.rect().wh())
        .x_y(0.0, 0.0);

    // draw.to_frame(app, &frame).unwrap();

    let encoder = frame.command_encoder();
    let texture_view = frame.texture_view();
    render_on_device(
        &model.font,
        &model.state,
        encoder,
        texture_view,
        &model.bind_group,
        &model.render_pipeline,
        &model.vertex_buffer,
    );
}

fn main() {
    tracing_subscriber::fmt::init();

    nannou::app(model).update(update).run();
}
