#![allow(clippy::collapsible_match)]
#![allow(clippy::single_match)]
use super::states::build_main;
use crate::{game::states::explorer::Explorer, graphics::VulkanRender};
use iron_oxide::ui::{DirtyFlags, UiEvent, UiState};
use log::info;
use std::{
    cell::RefCell,
    mem::{MaybeUninit, forget},
    rc::Rc,
    time::Instant,
};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{KeyCode, PhysicalKey},
    window::{Theme, Window, WindowId},
};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
pub const FPS_LIMIT: bool = false;

pub struct App {
    pub init: bool,
    pub window: MaybeUninit<Window>,
    pub renderer: Rc<RefCell<VulkanRender>>,
    pub ui: Rc<RefCell<UiState>>,
    pub explorer: Explorer,
    pub cursor_pos: PhysicalPosition<f64>,
    pub time: Instant,
    pub last_cursor_location: PhysicalPosition<f64>,
    pub touch_id: u64,
    pub mouse_pressed: bool,
    pub dirty: bool,
    pub target_frame_time: f32,
}

impl App {
    #[allow(unused)]
    pub fn run() -> Self {
        #[allow(invalid_value)]
        #[allow(clippy::uninit_assumed_init)]
        let renderer = Rc::new(RefCell::new(unsafe { MaybeUninit::uninit().assume_init() }));
        let mut state = build_main();
        let ui = Rc::new(RefCell::new(state));
        let mut explorer = Explorer::new(ui.clone());
        explorer.display_path();

        Self {
            window: MaybeUninit::uninit(),
            renderer,
            init: false,
            cursor_pos: PhysicalPosition::default(),
            time: Instant::now(),
            ui,
            explorer,
            last_cursor_location: PhysicalPosition::default(),
            touch_id: 0,
            mouse_pressed: false,
            dirty: false,
            target_frame_time: 1.0 / 144.0,
        }
    }

    pub fn window(&self) -> &Window {
        unsafe { self.window.assume_init_ref() }
    }
}

impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if !self.init {
            return;
        }

        match event {
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                let in_ui;

                {
                    let mut ui = self.ui.borrow_mut();
                    in_ui = ui.update_cursor(position.into(), UiEvent::Move);
                }

                if in_ui.is_new() {
                    self.dirty = true;
                    self.window().request_redraw();
                } else if self.dirty {
                    self.window().request_redraw();
                    self.dirty = false;
                }

                self.cursor_pos = position;
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                let in_ui = self
                    .ui
                    .borrow_mut()
                    .update_cursor(self.cursor_pos.into(), UiEvent::Scroll(delta));

                if in_ui.is_new() {
                    self.dirty = true;
                    self.window().request_redraw();
                } else if in_ui.is_old() {
                } else if self.dirty {
                    self.window().request_redraw();
                    self.dirty = false;
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => match button {
                MouseButton::Left => {
                    self.mouse_pressed = state == ElementState::Pressed;
                    self.ui.borrow_mut().update_cursor(
                        self.cursor_pos.into(),
                        match state {
                            ElementState::Pressed => UiEvent::Press,
                            ElementState::Released => UiEvent::Release,
                        },
                    );

                    self.window().request_redraw();
                }
                _ => (),
            },
            WindowEvent::Touch(touch) => {
                let cursor_pos = touch.location.into();
                match touch.phase {
                    TouchPhase::Started => {
                        if touch.id != 0 || self.touch_id != touch.id {
                            return;
                        }
                        self.touch_id = touch.id;
                        self.ui
                            .borrow_mut()
                            .update_cursor(cursor_pos, UiEvent::Press);
                        self.last_cursor_location = touch.location;
                    }
                    TouchPhase::Moved => {
                        self.last_cursor_location = touch.location;
                        self.ui
                            .borrow_mut()
                            .update_cursor(cursor_pos, UiEvent::Move);
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        self.touch_id = 0;
                        self.ui
                            .borrow_mut()
                            .update_cursor(cursor_pos, UiEvent::Release);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.renderer.borrow_mut().draw_frame();
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
                                {
                                    let mut value = self.ui.borrow_mut();
                                    value.visible = !value.visible;
                                    value.dirty = DirtyFlags::Size;
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                if !self.init {
                    return;
                }
                let size = self.window().inner_size();
                let mut renderer = self.renderer.borrow_mut();
                if new_size != size || new_size == renderer.window_size {
                    return;
                }
                renderer.recreate_swapchain(size);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => (),
        }

        let (needs_ticking, ui_event) = {
            let mut ui = self.ui.borrow_mut();
            (ui.needs_ticking(), ui.event.take())
        };

        if let Some(event) = ui_event {
            self.explorer.proceed_event(event);
        }

        if needs_ticking {
            event_loop.set_control_flow(ControlFlow::Poll);

            if self.time.elapsed().as_secs_f32() > self.target_frame_time {
                self.time = Instant::now();
                self.ui.borrow_mut().process_ticks();
                self.window().request_redraw();
            }
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}

    fn suspended(&mut self, _: &ActiveEventLoop) {
        println!("suspended");
        if !self.init {
            return;
        }
        self.init = false;
        let mut renderer = self.renderer.borrow_mut();
        renderer.destroy();
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed");
        if self.init {
            return;
        } else {
            self.init = true;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Vulkan Homeserver")
            .with_inner_size(PhysicalSize {
                width: WIDTH,
                height: HEIGHT,
            })
            .with_visible(false)
            .with_theme(Some(Theme::Dark));

        let window = event_loop.create_window(window_attributes).unwrap();
        if let Some(monitor) = window.current_monitor() {
            if let Some(refresh_rate) = monitor.refresh_rate_millihertz() {
                self.target_frame_time = 1000.0 / refresh_rate as f32;
                println!("target pfs: {}", refresh_rate / 1000);
            } else {
                println!("Refresh rate not available");
            }
        }
        forget(
            self.renderer
                .replace(VulkanRender::create(&window, self.ui.clone())),
        );

        let mut renderer = self.renderer.borrow_mut();

        let base_shaders = (
            include_bytes!("../../spv/basic.vert.spv").as_ref(),
            include_bytes!("../../spv/basic.frag.spv").as_ref(),
        );

        let font_shaders = (
            include_bytes!("../../spv/bitmap.vert.spv").as_ref(),
            include_bytes!("../../spv/bitmap.frag.spv").as_ref(),
        );

        let atlas_shaders = (
            include_bytes!("../../spv/atlas_texture.vert.spv").as_ref(),
            include_bytes!("../../spv/atlas_texture.frag.spv").as_ref(),
        );

        {
            let mut ui = self.ui.borrow_mut();
            ui.init_graphics(
                &renderer.base,
                renderer.single_time_command_pool,
                renderer.window_size,
                renderer.render_pass,
                renderer.ui_descriptor_set_layout,
                base_shaders,
                font_shaders,
                atlas_shaders,
            );
        }

        renderer.draw_frame();
        window.set_visible(true);

        self.window.write(window);
        println!("window time: {:?}", self.time.elapsed());
        self.time = Instant::now();
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        info!("exiting");
        if !self.init {
            return;
        }
        self.init = false;
    }
}
