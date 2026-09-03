//! L2 窗口 smoke：winit 0.30 + wgpu 30 清色循环。
//! `--frames N` 自动退出（SDD render-wgpu.md L2 叙述条款；CI xvfb-run 以 N=3 跑）。

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

const CLEAR: wgpu::Color = wgpu::Color {
    r: 0.05,
    g: 0.07,
    b: 0.10,
    a: 1.0,
};

struct Gfx {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl Gfx {
    fn new(window: Arc<Window>) -> Option<Self> {
        let instance = visia_render_wgpu::create_instance();
        let size = window.inner_size();
        // Arc<Window>：rwh 0.6 对 Arc 有 blanket impl，'static 借用面由 Arc 生命周期担保
        let surface = instance.create_surface(window).expect("create_surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .ok()?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("visia-clear"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .ok()?;
        let caps = surface.get_capabilities(&adapter);
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("no compatible surface config");
        config.format = *caps.formats.first()?;
        surface.configure(&device, &config);
        Some(Self {
            surface,
            device,
            queue,
            config,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn draw(&mut self) {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                let view = tex
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                {
                    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(CLEAR),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        ..Default::default()
                    });
                }
                self.queue.submit(std::iter::once(encoder.finish()));
                self.queue.present(tex);
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
            }
            // Timeout/Occluded/Validation：跳帧即正确语义
            other => {
                eprintln!("skipped frame: {other:?}");
            }
        }
    }
}

struct App {
    window: Option<Arc<Window>>,
    gfx: Option<Gfx>,
    frames_left: Option<u32>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gfx.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(winit::dpi::PhysicalSize::new(640, 480))
                        .with_title("VisiaEngine L2 smoke"),
                )
                .expect("create_window"),
        );
        self.gfx = Gfx::new(Arc::clone(&window)).or_else(|| {
            eprintln!("no compatible GPU adapter for surface — lavapipe/real GPU required");
            None
        });
        self.window = Some(window);
        if self.gfx.is_none() {
            event_loop.exit();
            return;
        }
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gfx) = &mut self.gfx {
                    gfx.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(gfx) = &mut self.gfx {
                    gfx.draw();
                }
                let chain = match self.frames_left {
                    None => true,
                    Some(1) => false,
                    Some(n) => {
                        self.frames_left = Some(n - 1);
                        true
                    }
                };
                if chain {
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                } else {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let mut frames: Option<u32> = None;
    while let Some(a) = args.next() {
        if a == "--frames" {
            frames = args
                .next()
                .and_then(|v| v.parse().ok())
                .filter(|n: &u32| *n > 0);
        }
    }
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        window: None,
        gfx: None,
        frames_left: frames,
    };
    // 首帧自触发
    event_loop.run_app(&mut app)?;
    Ok(())
}
