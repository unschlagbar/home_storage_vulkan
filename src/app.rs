#![allow(clippy::collapsible_match)]
#![allow(clippy::single_match)]
use crate::{explorer::Explorer, vulkan_render::VulkanRender};
use iron_oxide::{
    primitives::Vec2,
    ui::{DirtyFlags, UiEvent, UiState},
};
use std::{
    cell::RefCell,
    rc::Rc,
    thread::sleep,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{KeyCode, PhysicalKey},
    window::{Theme, Window, WindowId},
};

#[cfg(target_os = "windows")]
use winit::platform::windows::{CornerPreference, WindowAttributesExtWindows};

const APP_NAME: &str = "Home Server";
const WIDTH: u32 = 1080;
const HEIGHT: u32 = 720;

pub const VSYNC: bool = true;
const DEFAULT_FPS: f32 = 144.0;

pub struct App {
    pub window: Option<Window>,
    pub renderer: Option<Rc<RefCell<VulkanRender>>>,
    pub ui: Rc<RefCell<UiState>>,

    pub explorer: Explorer,
    pub time: Instant,

    pub cursor_pos: Vec2,
    pub touch_id: u64,
    pub dirty: bool,
    pub target_frame_time: f32,
}

impl App {
    #[allow(unused)]
    pub fn run() -> Self {
        let renderer = None;

        let ui = Rc::new(RefCell::new(UiState::create(true)));
        let mut explorer = Explorer::new(ui.clone());

        explorer.display_path();

        Self {
            window: None,
            renderer,
            cursor_pos: Vec2::default(),
            time: Instant::now(),
            ui,
            explorer,
            touch_id: 0,
            dirty: false,
            target_frame_time: 1.0 / DEFAULT_FPS,
        }
    }

    fn get_framerate(&mut self, window: &Window) {
        if let Some(monitor) = window.current_monitor() {
            if let Some(refresh_rate) = monitor.refresh_rate_millihertz() {
                self.target_frame_time = 1000.0 / refresh_rate as f32;
                println!("target pfs: {}", refresh_rate / 1000);
            } else {
                println!("Refresh rate not available");
            }
        }
    }

    fn create_window(&self, event_loop: &ActiveEventLoop) -> Window {
        let window_attributes = Window::default_attributes()
            .with_title(APP_NAME)
            .with_inner_size(PhysicalSize {
                width: WIDTH,
                height: HEIGHT,
            })
            .with_visible(false)
            .with_theme(Some(Theme::Dark));

        #[cfg(target_os = "windows")]
        let window_attributes =
            window_attributes.with_corner_preference(CornerPreference::RoundSmall);
        event_loop.create_window(window_attributes).unwrap()
    }
}

impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let (renderer, window) = if let Some(window) = &self.window
            && let Some(renderer) = &self.renderer
        {
            (renderer, window)
        } else {
            return;
        };

        match event {
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.cursor_pos = position.into();

                let result = {
                    let mut ui = self.ui.borrow_mut();
                    ui.update_cursor(self.cursor_pos, UiEvent::Move)
                };

                if result.is_new() {
                    self.dirty = true;
                    window.request_redraw();
                } else if self.dirty && result.is_none() {
                    window.request_redraw();
                    self.dirty = false;
                }
            }
            WindowEvent::CursorLeft { device_id: _ } => {
                let result = {
                    let mut ui = self.ui.borrow_mut();
                    ui.update_cursor(Vec2::new(1000.0, 1000.0), UiEvent::Move)
                };

                if result.is_new() {
                    self.dirty = true;
                    window.request_redraw();
                } else if self.dirty && result.is_none() {
                    window.request_redraw();
                    self.dirty = false;
                }
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                let result = self
                    .ui
                    .borrow_mut()
                    .update_cursor(self.cursor_pos, UiEvent::Scroll(delta));

                if result.is_new() {
                    self.dirty = true;
                    window.request_redraw();
                } else if self.dirty && result.is_none() {
                    window.request_redraw();
                    self.dirty = false;
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                match button {
                    MouseButton::Left => {
                        let result = self
                            .ui
                            .borrow_mut()
                            .update_cursor(self.cursor_pos, state.into());

                        if result.is_new() {
                            window.request_redraw();
                        }
                        if self.explorer.mouse_click() {
                            window.request_redraw();
                        }
                    }
                    MouseButton::Right => {
                        //self.explorer.mouse_click();
                        if state == ElementState::Pressed && self.explorer.right_click() {
                            window.request_redraw();
                        }
                    }
                    _ => (),
                }
            }
            WindowEvent::Touch(touch) => {
                let cursor_pos = touch.location.into();
                match touch.phase {
                    TouchPhase::Started => {
                        if self.touch_id == 0 {
                            self.touch_id = touch.id;
                        }
                    }
                    TouchPhase::Moved => (),
                    TouchPhase::Ended | TouchPhase::Cancelled => self.touch_id = 0,
                }
                self.ui
                    .borrow_mut()
                    .update_cursor(cursor_pos, touch.phase.into());
            }
            WindowEvent::RedrawRequested => {
                renderer.borrow_mut().draw_frame();
                return;
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    match key_code {
                        KeyCode::F1 => {
                            if event.state.is_pressed() {
                                let mut ui = self.ui.borrow_mut();
                                ui.visible = !ui.visible;
                                window.request_redraw();
                            }
                        }
                        KeyCode::KeyT => {
                            window.set_maximized(true);
                        }
                        KeyCode::KeyU => {
                            window.set_minimized(true);
                        }
                        _ => (),
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                let mut renderer = renderer.borrow_mut();
                if new_size == renderer.window_size {
                    return;
                }
                renderer.recreate_swapchain(new_size);
                window.request_redraw();
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }

        let ui_event = {
            let mut ui = self.ui.borrow_mut();
            ui.event.take()
        };

        if let Some(event) = ui_event {
            self.explorer.proceed_event(event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window
            && self.ui.borrow().needs_ticking()
        {
            event_loop.set_control_flow(ControlFlow::Poll);

            if self.time.elapsed().as_secs_f32() > self.target_frame_time {
                self.time = Instant::now();
                self.ui.borrow_mut().process_ticks();
                if !matches!(self.ui.borrow().dirty, DirtyFlags::None) {
                    window.request_redraw();
                }
                sleep(Duration::from_secs_f32(self.target_frame_time * 0.9));
            }
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        println!("suspended");

        if let Some(renderer) = &self.renderer {
            renderer.borrow_mut().destroy();
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed");

        let window = self.create_window(event_loop);
        self.get_framerate(&window);

        let renderer = if let Some(renderer) = &self.renderer {
            renderer.replace(VulkanRender::create(&window, self.ui.clone()));
            renderer
        } else {
            self.renderer = Some(Rc::new(RefCell::new(VulkanRender::create(
                &window,
                self.ui.clone(),
            ))));
            self.renderer.as_ref().unwrap()
        }
        .borrow_mut();

        let base_shaders = (
            include_bytes!("../spv/basic.vert.spv").as_ref(),
            include_bytes!("../spv/basic.frag.spv").as_ref(),
        );

        let font_shaders = (
            include_bytes!("../spv/atlas_texture.vert.spv").as_ref(),
            include_bytes!("../spv/bitmap.frag.spv").as_ref(),
        );

        let atlas_shaders = (
            include_bytes!("../spv/atlas_texture.vert.spv").as_ref(),
            include_bytes!("../spv/atlas_texture.frag.spv").as_ref(),
        );

        {
            let mut ui = self.ui.borrow_mut();
            ui.init_graphics(
                &renderer.base,
                renderer.single_time_cmd_pool,
                renderer.window_size,
                renderer.render_pass,
                &renderer.uniform_buffers[0],
                renderer.font_atlas.view,
                renderer.texture_sampler,
                base_shaders,
                font_shaders,
                atlas_shaders,
            );
        }

        window.set_visible(true);

        self.window = Some(window);
        println!("window time: {:?}", self.time.elapsed());
        self.time = Instant::now();
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        println!("exiting");

        if let Some(renderer) = &self.renderer {
            renderer.borrow_mut().destroy();
        }
    }
}
