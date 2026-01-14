#![allow(clippy::collapsible_match)]
#![allow(clippy::single_match)]
use crate::{explorer::Explorer, network::Network, vulkan_render::VulkanRender};
use iron_oxide::ui::Ui;

use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Theme, Window, WindowId},
};

#[cfg(target_os = "windows")]
use winit::platform::windows::{CornerPreference, WindowAttributesExtWindows};

#[cfg(target_os = "android")]
use ndk::asset::AssetManager;

const APP_NAME: &str = "Home Server";
const WIDTH: u32 = 1080;
const HEIGHT: u32 = 720;

pub const VSYNC: bool = false;
const LIMIT_FPS: bool = true;
const ONLY_DRAW_ON_UPDATE: bool = true;
const DEFAULT_FPS: u64 = 60;

pub const DEBUG_PERF: bool = true;

pub struct App {
    pub window: Option<Window>,
    pub renderer: Option<Rc<RefCell<VulkanRender>>>,
    pub ui: Rc<RefCell<Ui>>,

    pub net: Arc<Network>,

    #[cfg(target_os = "android")]
    pub assets: AssetManager,

    pub explorer: Explorer,

    pub target_frame_time: Duration,
    pub time: Instant,
}

impl App {
    #[cfg(not(target_os = "android"))]
    pub fn create(net: Arc<Network>) -> Self {
        let renderer = None;

        let ui = Rc::new(RefCell::new(Ui::create(true)));
        let mut explorer = Explorer::new(ui.clone());

        explorer.display_path();

        Self {
            window: None,
            renderer,
            ui,
            net,
            explorer,
            target_frame_time: Duration::from_millis(1000 / DEFAULT_FPS),
            time: Instant::now(),
        }
    }

    #[cfg(target_os = "android")]
    pub fn create(assets: AssetManager) -> Self {
        let renderer = None;

        let ui = Rc::new(RefCell::new(Ui::create(true)));
        let mut explorer = Explorer::new(ui.clone());

        explorer.display_path();

        Self {
            window: None,
            renderer,
            ui,

            assets,

            cursor_pos: Vec2::default(),
            time: Instant::now(),
            explorer,
            target_frame_time: 1.0 / DEFAULT_FPS,
        }
    }

    fn get_framerate(&mut self, window: &Window) {
        if let Some(monitor) = window.current_monitor() {
            if let Some(refresh_rate) = monitor.refresh_rate_millihertz() {
                self.target_frame_time = Duration::from_millis(1000000 / refresh_rate as u64);
                println!("target frame_time: {:?}", self.target_frame_time);
            } else {
                println!("Refresh rate not available {:?}", self.target_frame_time);
            }
        } else {
            let mut refresh_rate = 60000;
            window.available_monitors().for_each(|x| {
                if let Some(x) = x.refresh_rate_millihertz() {
                    refresh_rate = refresh_rate.max(x);
                } else {
                    println!("Refresh rate not available {:?}", self.target_frame_time)
                }
            });
            self.target_frame_time = Duration::from_millis(1000000 / refresh_rate as u64);
            println!("target frame_time: {:?}", self.target_frame_time);
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

        let input_consumed;

        let (ui_event, ui_event2) = {
            let mut ui = self.ui.borrow_mut();
            input_consumed = ui.window_event(&event, window).is_new();
            (ui.get_event(), ui.get_event())
        };

        if let Some(event) = ui_event {
            self.explorer.proceed_event(event);
            if let Some(event) = ui_event2 {
                self.explorer.proceed_event(event)
            }
        }

        match event {
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                let mut ui = self.ui.borrow_mut();
                match (button, state) {
                    (MouseButton::Left, ElementState::Released) => {
                        if self.explorer.mouse_click(&mut ui) {
                            window.request_redraw();
                        }
                    }
                    (MouseButton::Right, ElementState::Pressed) => {
                        if state == ElementState::Pressed && self.explorer.right_click(&mut ui) {
                            window.request_redraw();
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }

        if input_consumed {
            return;
        }

        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.state == ElementState::Pressed
                    && let PhysicalKey::Code(key_code) = event.physical_key
                {
                    match key_code {
                        KeyCode::F1 => {
                            if event.state.is_pressed() {
                                let mut ui = self.ui.borrow_mut();
                                ui.visible = !ui.visible;
                                window.request_redraw();
                            }
                        }
                        KeyCode::F11 => {
                            if window.fullscreen().is_none() {
                                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                            } else {
                                window.set_fullscreen(None);
                            }
                        }
                        KeyCode::KeyT => {
                            window.set_maximized(true);
                        }
                        KeyCode::KeyZ => {
                            window.set_maximized(false);
                        }
                        KeyCode::KeyU => {
                            window.set_minimized(true);
                        }
                        _ => (),
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if DEBUG_PERF {
                    let start = Instant::now();
                    renderer.borrow_mut().draw_frame();
                    println!("Draw: {:?}", start.elapsed());
                } else {
                    renderer.borrow_mut().draw_frame();
                }
            }
            WindowEvent::Resized(new_size) => {
                let mut renderer = renderer.borrow_mut();
                if new_size == renderer.window_size {
                    return;
                }
                renderer.recreate_swapchain(new_size);
                // The window_event fn will take care of RedrawRequest
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if ONLY_DRAW_ON_UPDATE {
            if let Some(window) = &self.window {
                let mut ui = self.ui.borrow_mut();
                if ui.needs_ticking() {
                    ui.process_ticks();
                    if ui.is_dirty() {
                        window.request_redraw();
                    }
                }
                event_loop.set_control_flow(ControlFlow::wait_duration(self.target_frame_time));
            }
        } else {
            if LIMIT_FPS {
                event_loop.set_control_flow(ControlFlow::wait_duration(self.target_frame_time));
            }

            if let Some(window) = &self.window {
                let mut ui = self.ui.borrow_mut();
                if ui.needs_ticking() {
                    ui.process_ticks();
                }
                window.request_redraw();
            }
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

        let window = if let Some(window) = self.window.take() {
            window
        } else {
            let window = self.create_window(event_loop);
            self.get_framerate(&window);
            window
        };

        if let Some(renderer) = &self.renderer {
            renderer.replace(VulkanRender::create(&window, self.ui.clone()));
        } else {
            self.renderer = Some(Rc::new(RefCell::new(VulkanRender::create(
                &window,
                self.ui.clone(),
            ))));
        };

        self.ui.borrow_mut().resize(window.inner_size().into());

        window.set_visible(true);

        self.window = Some(window);
        if DEBUG_PERF {
            println!("window time: {:?}", self.time.elapsed());
        }
        self.time = Instant::now();
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        println!("exiting");

        if let Some(renderer) = &self.renderer {
            renderer.borrow_mut().destroy();
        }
    }
}
