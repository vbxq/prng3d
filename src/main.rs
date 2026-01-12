use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use glam::Vec2;

mod math;
mod renderer;
mod rng;
mod ui;

use math::examples::MathFunctionKind;
use math::{MathEngine, MathResult};
use renderer::{Camera, GpuState, generate_grid_vertices};
use rng::RngEngine;
use ui::state::{AppMode, MathViewMode, ViewMode};
use ui::{UiActions, UiState, apply_theme, draw_help_overlay, draw_side_panel};

struct InputState {
    forward: f32,
    right: f32,
    up: f32,
    mouse_captured: bool,
    mouse_delta: Vec2,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            forward: 0.0,
            right: 0.0,
            up: 0.0,
            mouse_captured: false,
            mouse_delta: Vec2::ZERO,
        }
    }
}

enum CurrentMathMesh {
    None,
    Surface,
    Curve,
    ParametricSurface,
}

fn surface_to_heatmap(vertices: &[f32], _z_min: f32, _z_max: f32) -> Vec<f32> {
    vertices
        .chunks(3)
        .flat_map(|chunk| {
            if chunk.len() >= 3 {
                let x = chunk[0] / 220.0;
                let y = chunk[2] / 220.0;
                let height = chunk[1];
                let normalized_value = ((height / 100.0) + 0.5).clamp(0.0, 1.0);
                vec![x, y, normalized_value]
            } else {
                vec![]
            }
        })
        .collect()
}

fn curve_to_2d(vertices: &[f32]) -> Vec<f32> {
    if vertices.is_empty() {
        return vec![];
    }

    let mut x_min = f32::MAX;
    let mut x_max = f32::MIN;
    let mut y_min = f32::MAX;
    let mut y_max = f32::MIN;

    for chunk in vertices.chunks(3) {
        if chunk.len() >= 3 {
            x_min = x_min.min(chunk[0]);
            x_max = x_max.max(chunk[0]);
            y_min = y_min.min(chunk[1]);
            y_max = y_max.max(chunk[1]);
        }
    }

    let x_range = (x_max - x_min).max(0.001);
    let y_range = (y_max - y_min).max(0.001);
    let scale = x_range.max(y_range);
    let x_center = (x_min + x_max) / 2.0;
    let y_center = (y_min + y_max) / 2.0;

    vertices
        .chunks(3)
        .flat_map(|chunk| {
            if chunk.len() >= 3 {
                let x = ((chunk[0] - x_center) / scale) * 1.8;
                let y = ((chunk[1] - y_center) / scale) * 1.8;
                vec![x, y]
            } else {
                vec![]
            }
        })
        .collect()
}

struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,
    egui_ctx: egui::Context,

    camera: Camera,
    rng_engine: RngEngine,
    math_engine: MathEngine,
    ui_state: UiState,
    input: InputState,

    last_frame: Instant,
    frame_count: u32,
    fps_timer: Instant,

    accumulated_points_3d: Vec<f32>,
    accumulated_points_2d: Vec<f32>,

    last_vsync_state: bool,
    last_frame_time: Instant,

    current_math_mesh: CurrentMathMesh,
    math_last_error: Option<String>,
    grid_uploaded: bool,

    cached_surface_vertices: Vec<f32>,
    cached_surface_z_min: f32,
    cached_surface_z_max: f32,
    cached_curve_vertices: Vec<f32>,
    math_2d_uploaded: bool,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            gpu: None,
            egui_state: None,
            egui_renderer: None,
            egui_ctx: egui::Context::default(),

            camera: Camera::default(),
            rng_engine: RngEngine::new(),
            math_engine: MathEngine::new(),
            ui_state: UiState::default(),
            input: InputState::default(),

            last_frame: Instant::now(),
            frame_count: 0,
            fps_timer: Instant::now(),

            accumulated_points_3d: Vec::with_capacity(4_000_000 * 3),
            accumulated_points_2d: Vec::with_capacity(1_000_000 * 3),

            last_vsync_state: false,
            last_frame_time: Instant::now(),

            current_math_mesh: CurrentMathMesh::None,
            math_last_error: None,
            grid_uploaded: false,

            cached_surface_vertices: Vec::new(),
            cached_surface_z_min: 0.0,
            cached_surface_z_max: 1.0,
            cached_curve_vertices: Vec::new(),
            math_2d_uploaded: false,
        }
    }

    fn init_gpu(&mut self, window: Arc<Window>) {
        let gpu = pollster::block_on(GpuState::new(window.clone()));

        let egui_state = egui_winit::State::new(
            self.egui_ctx.clone(),
            self.egui_ctx.viewport_id(),
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(2048),
        );

        let egui_renderer =
            egui_wgpu::Renderer::new(&gpu.device, gpu.config.format, None, 1, false);

        apply_theme(&self.egui_ctx);

        self.window = Some(window);
        self.gpu = Some(gpu);
        self.egui_state = Some(egui_state);
        self.egui_renderer = Some(egui_renderer);

        if self.ui_state.code_needs_compile {
            self.rng_engine.update_code(&self.ui_state.code);
            self.ui_state.code_needs_compile = false;
        }

        if self.ui_state.math_needs_compile {
            self.compile_math();
        }
    }

    fn compile_math(&mut self) {
        match self.ui_state.math_function_type {
            MathFunctionKind::Surface => {
                self.math_engine.compile_surface(
                    &self.ui_state.math_code,
                    (
                        self.ui_state.math_x_range.0 as f64,
                        self.ui_state.math_x_range.1 as f64,
                    ),
                    (
                        self.ui_state.math_y_range.0 as f64,
                        self.ui_state.math_y_range.1 as f64,
                    ),
                    self.ui_state.math_resolution as usize,
                );
            }
            MathFunctionKind::ParametricCurve => {
                self.math_engine.compile_parametric_curve(
                    &self.ui_state.math_code,
                    (
                        self.ui_state.math_t_range.0 as f64,
                        self.ui_state.math_t_range.1 as f64,
                    ),
                    self.ui_state.math_samples as usize,
                );
            }
            MathFunctionKind::ParametricSurface => {
                self.math_engine.compile_parametric_surface(
                    &self.ui_state.math_code,
                    (
                        self.ui_state.math_u_range.0 as f64,
                        self.ui_state.math_u_range.1 as f64,
                    ),
                    (
                        self.ui_state.math_v_range.0 as f64,
                        self.ui_state.math_v_range.1 as f64,
                    ),
                    self.ui_state.math_u_samples as usize,
                    self.ui_state.math_v_samples as usize,
                );
            }
        }
        self.ui_state.math_needs_compile = false;
    }

    fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        self.frame_count += 1;
        if self.fps_timer.elapsed().as_secs_f32() >= 1.0 {
            let fps = self.frame_count as f32 / self.fps_timer.elapsed().as_secs_f32();
            *self.rng_engine.stats().fps.lock() = fps;
            self.frame_count = 0;
            self.fps_timer = Instant::now();
        }

        self.camera.set_mode(self.ui_state.camera_mode);
        self.camera
            .process_keyboard(self.input.forward, self.input.right, self.input.up, dt);

        if self.input.mouse_captured {
            self.camera.process_mouse_movement(self.input.mouse_delta);
        }
        self.input.mouse_delta = Vec2::ZERO;

        match self.ui_state.app_mode {
            AppMode::Rng => self.update_rng(),
            AppMode::Math => self.update_math(),
        }
    }

    fn update_rng(&mut self) {
        self.rng_engine.bounds().set(
            self.ui_state.bounds_min[0] as i64,
            self.ui_state.bounds_max[0] as i64,
            self.ui_state.bounds_min[1] as i64,
            self.ui_state.bounds_max[1] as i64,
            self.ui_state.bounds_min[2] as i64,
            self.ui_state.bounds_max[2] as i64,
        );

        while let Some(batch) = self.rng_engine.try_recv_batch() {
            let max_floats = self.ui_state.max_points.min(4_000_000) * 3;

            match self.ui_state.view_mode {
                ViewMode::Mode3D => {
                    if self.accumulated_points_3d.len() + batch.len() > max_floats {
                        let overflow =
                            (self.accumulated_points_3d.len() + batch.len()) - max_floats;
                        if overflow < self.accumulated_points_3d.len() {
                            self.accumulated_points_3d.drain(0..overflow);
                        } else {
                            self.accumulated_points_3d.clear();
                        }
                    }
                    self.accumulated_points_3d.extend(&batch);
                }
                ViewMode::Mode2D => {
                    let grid = self.ui_state.grid_size as usize;
                    let max_2d = grid * grid * 3;

                    if self.accumulated_points_2d.len() + batch.len() > max_2d {
                        let overflow = (self.accumulated_points_2d.len() + batch.len()) - max_2d;
                        if overflow < self.accumulated_points_2d.len() {
                            self.accumulated_points_2d.drain(0..overflow);
                        } else {
                            self.accumulated_points_2d.clear();
                        }
                    }
                    self.accumulated_points_2d.extend(&batch);
                }
            }
        }

        if let Some(gpu) = &mut self.gpu {
            match self.ui_state.view_mode {
                ViewMode::Mode3D => {
                    gpu.point_buffers
                        .upload_3d(&gpu.queue, &self.accumulated_points_3d);
                    self.rng_engine
                        .stats()
                        .points_rendered
                        .store(self.accumulated_points_3d.len() / 3, Ordering::Relaxed);
                }
                ViewMode::Mode2D => {
                    let points_2d: Vec<f32> = self
                        .accumulated_points_2d
                        .chunks(3)
                        .flat_map(|chunk| {
                            if chunk.len() >= 3 {
                                let x = (chunk[0] / 500.0) * 0.9;
                                let y = (chunk[1] / 500.0) * 0.9;
                                let v = (chunk[2] + 500.0) / 1000.0;
                                vec![x, y, v]
                            } else {
                                vec![]
                            }
                        })
                        .collect();

                    gpu.point_buffers.upload_2d(&gpu.queue, &points_2d);
                    self.rng_engine
                        .stats()
                        .points_rendered
                        .store(points_2d.len() / 3, Ordering::Relaxed);
                }
            }
        }
    }

    fn update_math(&mut self) {
        while let Some(result) = self.math_engine.try_recv_result() {
            match result {
                MathResult::Surface(mesh) => {
                    if let Some(gpu) = &mut self.gpu {
                        gpu.math_buffers.upload_surface(&gpu.queue, &mesh);

                        self.cached_surface_vertices = mesh.mesh.vertices.clone();
                        self.cached_surface_z_min = mesh.z_min;
                        self.cached_surface_z_max = mesh.z_max;

                        self.current_math_mesh = CurrentMathMesh::Surface;
                        self.math_last_error = None;
                        self.math_2d_uploaded = false;
                    }
                }
                MathResult::ParametricCurve(mesh) => {
                    if let Some(gpu) = &mut self.gpu {
                        gpu.math_buffers.upload_curve(&gpu.queue, &mesh);

                        self.cached_curve_vertices = mesh.vertices.clone();

                        self.current_math_mesh = CurrentMathMesh::Curve;
                        self.math_last_error = None;
                        self.math_2d_uploaded = false;
                    }
                }
                MathResult::ParametricSurface(mesh) => {
                    if let Some(gpu) = &mut self.gpu {
                        gpu.math_buffers
                            .upload_parametric_surface(&gpu.queue, &mesh);

                        self.current_math_mesh = CurrentMathMesh::ParametricSurface;
                        self.math_last_error = None;
                        self.math_2d_uploaded = false;
                    }
                }
                MathResult::Error(e) => {
                    self.math_last_error = Some(e);
                }
            }
        }

        if self.ui_state.math_view_mode == MathViewMode::Mode2D && !self.math_2d_uploaded {
            if let Some(gpu) = &mut self.gpu {
                match self.current_math_mesh {
                    CurrentMathMesh::Surface => {
                        let heatmap_data = surface_to_heatmap(
                            &self.cached_surface_vertices,
                            self.cached_surface_z_min,
                            self.cached_surface_z_max,
                        );
                        gpu.math_buffers.upload_heatmap(&gpu.queue, &heatmap_data);
                        self.math_2d_uploaded = true;
                    }
                    CurrentMathMesh::Curve => {
                        let curve_2d_data = curve_to_2d(&self.cached_curve_vertices);
                        gpu.math_buffers.upload_curve_2d(&gpu.queue, &curve_2d_data);
                        self.math_2d_uploaded = true;
                    }
                    CurrentMathMesh::ParametricSurface => {}
                    CurrentMathMesh::None => {}
                }
            }
        }

        if self.ui_state.show_grid && !self.grid_uploaded {
            if let Some(gpu) = &mut self.gpu {
                let grid_verts = generate_grid_vertices(250.0, 20);
                gpu.math_buffers.upload_grid(&gpu.queue, &grid_verts);
                self.grid_uploaded = true;
            }
        }
    }

    fn render(&mut self) {
        if self.ui_state.fps_cap_enabled {
            let frame_duration = Duration::from_secs_f64(1.0 / self.ui_state.fps_cap as f64);
            let elapsed = self.last_frame_time.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
        self.last_frame_time = Instant::now();

        let (Some(window), Some(egui_state)) = (&self.window, &mut self.egui_state) else {
            return;
        };

        let raw_input = egui_state.take_egui_input(window);

        let stats = Arc::clone(self.rng_engine.stats());

        let last_error = match self.ui_state.app_mode {
            AppMode::Rng => self.rng_engine.last_error(),
            AppMode::Math => self
                .math_last_error
                .clone()
                .or_else(|| self.math_engine.last_error()),
        };

        let camera_pos = self.camera.position.to_array();
        let camera_speed = self.camera.move_speed;
        let is_paused = self.rng_engine.is_paused();
        let app_mode = self.ui_state.app_mode;

        let mut ui_actions = UiActions::default();

        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            ui_actions = draw_side_panel(ctx, &mut self.ui_state, &stats, &last_error, is_paused);

            let show_overlay = match app_mode {
                AppMode::Rng => self.ui_state.view_mode == ViewMode::Mode3D,
                AppMode::Math => self.ui_state.math_view_mode == MathViewMode::Mode3D,
            };

            if show_overlay {
                draw_help_overlay(ctx, camera_pos, camera_speed);
            }
        });

        self.handle_ui_actions(ui_actions);

        let Some(gpu) = &mut self.gpu else { return };
        let Some(window) = &self.window else { return };
        let Some(egui_state) = &mut self.egui_state else {
            return;
        };
        let Some(egui_renderer) = &mut self.egui_renderer else {
            return;
        };

        egui_state.handle_platform_output(window, full_output.platform_output);

        if self.ui_state.vsync_enabled != self.last_vsync_state {
            gpu.set_vsync(self.ui_state.vsync_enabled);
            self.last_vsync_state = self.ui_state.vsync_enabled;
        }

        let output = match gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                gpu.resize(gpu.size);
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                panic!("Out of GPU memory");
            }
            Err(wgpu::SurfaceError::Timeout) => {
                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        gpu.update_camera(&self.camera);

        let paint_jobs = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [gpu.config.width, gpu.config.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        for (id, delta) in full_output.textures_delta.set {
            egui_renderer.update_texture(&gpu.device, &gpu.queue, id, &delta);
        }

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });

        egui_renderer.update_buffers(
            &gpu.device,
            &gpu.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        match self.ui_state.app_mode {
            AppMode::Rng => match self.ui_state.view_mode {
                ViewMode::Mode3D => gpu.render_3d(&view, &mut encoder),
                ViewMode::Mode2D => gpu.render_2d(&view, &mut encoder),
            },
            AppMode::Math => match self.ui_state.math_view_mode {
                MathViewMode::Mode3D => {
                    if self.ui_state.show_grid {
                        gpu.render_grid(&view, &mut encoder, true);
                    }

                    match self.current_math_mesh {
                        CurrentMathMesh::Surface => {
                            if self.ui_state.show_grid {
                                gpu.render_surface_no_clear(&view, &mut encoder);
                            } else {
                                gpu.render_surface(&view, &mut encoder);
                            }
                        }
                        CurrentMathMesh::Curve => {
                            if self.ui_state.show_grid {
                                gpu.render_curve_no_clear(&view, &mut encoder);
                            } else {
                                gpu.render_curve(&view, &mut encoder);
                            }
                        }
                        CurrentMathMesh::ParametricSurface => {
                            if self.ui_state.show_grid {
                                gpu.render_surface_no_clear(&view, &mut encoder);
                            } else {
                                gpu.render_surface(&view, &mut encoder);
                            }
                        }
                        CurrentMathMesh::None => {
                            if !self.ui_state.show_grid {
                                gpu.render_grid(&view, &mut encoder, true);
                            }
                        }
                    }
                }
                MathViewMode::Mode2D => match self.current_math_mesh {
                    CurrentMathMesh::Surface => {
                        gpu.render_math_2d(&view, &mut encoder);
                    }
                    CurrentMathMesh::Curve => {
                        gpu.render_curve_2d(&view, &mut encoder);
                    }
                    CurrentMathMesh::ParametricSurface => {
                        gpu.render_grid(&view, &mut encoder, true);
                    }
                    CurrentMathMesh::None => {
                        gpu.render_grid(&view, &mut encoder, true);
                    }
                },
            },
        }

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let mut render_pass = render_pass.forget_lifetime();
            egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        for id in full_output.textures_delta.free {
            egui_renderer.free_texture(&id);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        window.request_redraw();
    }

    fn handle_ui_actions(&mut self, actions: UiActions) {
        if actions.compile_code {
            self.rng_engine.update_code(&self.ui_state.code);
        }

        if actions.reset_rng {
            self.rng_engine.reset();
        }

        if let Some(seed) = actions.set_seed {
            self.rng_engine.set_seed(seed);
        }

        if actions.toggle_pause {
            if self.rng_engine.is_paused() {
                self.rng_engine.resume();
            } else {
                self.rng_engine.pause();
            }
        }

        if actions.clear_points {
            self.accumulated_points_3d.clear();
            self.accumulated_points_2d.clear();
        }

        if actions.compile_math {
            self.compile_math();
        }
    }

    fn handle_key(&mut self, key: KeyCode, pressed: bool) {
        let value = if pressed { 1.0 } else { 0.0 };

        match key {
            KeyCode::KeyW | KeyCode::KeyZ => self.input.forward = value,
            KeyCode::KeyS => self.input.forward = -value,
            KeyCode::KeyA | KeyCode::KeyQ => self.input.right = -value,
            KeyCode::KeyD => self.input.right = value,
            KeyCode::Space => self.input.up = value,
            KeyCode::ShiftLeft | KeyCode::ControlLeft => self.input.up = -value,
            KeyCode::Escape if pressed => {
                self.input.mouse_captured = false;
                if let Some(window) = &self.window {
                    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                    window.set_cursor_visible(true);
                }
            }
            KeyCode::KeyP if pressed => {
                if self.rng_engine.is_paused() {
                    self.rng_engine.resume();
                } else {
                    self.rng_engine.pause();
                }
            }
            _ => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attrs = Window::default_attributes()
            .with_title("PRNG 3D Visualizer")
            .with_inner_size(PhysicalSize::new(1600, 900));

        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
        self.init_gpu(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(egui_state) = &mut self.egui_state {
            if let Some(window) = &self.window {
                let response = egui_state.on_window_event(window, &event);
                if response.consumed {
                    return;
                }
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                self.rng_engine.stop();
                self.math_engine.stop();
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size);
                    self.camera
                        .set_aspect(size.width as f32, size.height as f32);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    self.handle_key(key, event.state == ElementState::Pressed);
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state,
                ..
            } => {
                self.input.mouse_captured = state == ElementState::Pressed;

                if let Some(window) = &self.window {
                    if self.input.mouse_captured {
                        let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined);
                        window.set_cursor_visible(false);
                    } else {
                        let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                        window.set_cursor_visible(true);
                    }
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 50.0,
                };
                self.camera.process_scroll(scroll);
            }

            WindowEvent::RedrawRequested => {
                self.update();
                self.render();
            }

            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.input.mouse_captured {
                self.input.mouse_delta.x += delta.0 as f32;
                self.input.mouse_delta.y += delta.1 as f32;
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
