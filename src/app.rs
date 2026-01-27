#![allow(clippy::collapsible_match)]
#![allow(clippy::single_match)]
use crate::network::Network;
use crate::render_assets::RenderAssets;
use crate::{explorer::Explorer, vulkan_render::VulkanRender};
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

#[cfg(target_os = "linux")]
use winit::platform::wayland::WindowAttributesExtWayland;

#[cfg(target_os = "windows")]
use winit::platform::windows::{CornerPreference, WindowAttributesExtWindows};

const APP_NAME: &str = "Home Server";
const WIDTH: u32 = 1080;
const HEIGHT: u32 = 720;

pub const VSYNC: bool = false;
const LIMIT_FPS: bool = true;
const ONLY_DRAW_ON_UPDATE: bool = true;
const DEFAULT_FPS: u64 = 60;

pub const DEBUG_PERF: bool = false;

pub struct App {
    pub window: Option<Window>,
    pub renderer: Option<Rc<RefCell<VulkanRender>>>,
    pub render_assets: RenderAssets,
    pub ui: Rc<RefCell<Ui>>,

    #[allow(unused)]
    pub net: Arc<Network>,

    pub explorer: Explorer,

    pub target_frame_time: Duration,
    pub time: Instant,
}

impl App {
    pub fn create(net: Arc<Network>) -> Self {
        let renderer = None;

        let ui = Rc::new(RefCell::new(Ui::create(true)));
        let mut explorer = Explorer::new(ui.clone());

        explorer.display_path();

        Self {
            window: None,
            renderer,
            render_assets: RenderAssets::default(),
            ui,
            net,
            explorer,
            target_frame_time: Duration::from_millis(1000 / DEFAULT_FPS),
            time: Instant::now(),
        }
    }

    fn get_framerate(&mut self, window: &Window) {
        if let Some(monitor) = window.current_monitor() {
            if let Some(refresh_rate) = monitor.refresh_rate_millihertz() {
                self.target_frame_time = Duration::from_millis(1_000_000 / refresh_rate as u64);
                println!("target frametime: {:?}", self.target_frame_time);
            } else {
                println!("Refresh rate not available {:?}", self.target_frame_time);
            }
        } else {
            let mut refresh_rate = 60_000;
            window.available_monitors().for_each(|x| {
                if let Some(x) = x.refresh_rate_millihertz() {
                    refresh_rate = refresh_rate.max(x);
                } else {
                    println!("Refresh rate not available {:?}", self.target_frame_time)
                }
            });
            self.target_frame_time = Duration::from_millis(1_000_000 / refresh_rate as u64);
            println!("target frametime: {:?}", self.target_frame_time);
        }
    }

    fn create_window(&self, event_loop: &ActiveEventLoop) -> Window {
        let window_attributes = Window::default_attributes()
            .with_title(APP_NAME)
            .with_inner_size(PhysicalSize {
                width: WIDTH,
                height: HEIGHT,
            })
            .with_theme(Some(Theme::Dark));

        #[cfg(target_os = "linux")]
        let window_attributes = window_attributes.with_name(APP_NAME, APP_NAME);

        #[cfg(target_os = "windows")]
        let window_attributes = window_attributes
            .with_corner_preference(CornerPreference::RoundSmall)
            .with_visible(false);

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

        let ui_events = {
            let mut ui = self.ui.borrow_mut();
            input_consumed = ui.window_event(&event, window).is_new();
            [ui.get_event(), ui.get_event()]
        };

        for event in ui_events.into_iter().flatten() {
            self.explorer.proceed_event(event)
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
                        KeyCode::F11 => {
                            if window.fullscreen().is_none() {
                                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                            } else {
                                window.set_fullscreen(None);
                            }
                        }
                        KeyCode::F3 => {
                            let mut ui = self.ui.borrow_mut();
                            let elem = ui.get_element(self.explorer.properties_view.id).unwrap();
                            println!("f3 Debug: {:?}", elem);
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
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Destroyed => event_loop.exit(),
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
            let mut renderer = renderer.borrow_mut();
            renderer.destroy_ressources(&mut self.render_assets);
        }

        self.renderer = None;
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = if let Some(window) = self.window.take() {
            window
        } else {
            let window = self.create_window(event_loop);
            self.get_framerate(&window);
            window
        };

        if let Some(renderer) = &self.renderer {
            let mut renderer = renderer.borrow_mut();
            self.render_assets.init(&mut renderer);
        } else if DEBUG_PERF {
            let start_time = Instant::now();

            let mut renderer = VulkanRender::create(&window, self.ui.clone());
            self.render_assets.init(&mut renderer);
            self.renderer = Some(Rc::new(RefCell::new(renderer)));

            println!("Vulkan time: {:?}", start_time.elapsed());
        } else {
            let mut renderer = VulkanRender::create(&window, self.ui.clone());
            self.render_assets.init(&mut renderer);
            self.renderer = Some(Rc::new(RefCell::new(renderer)));
        }

        let mut ui = self.ui.borrow_mut();
        ui.scale_factor = window.scale_factor() as f32;
        ui.resize(window.inner_size().into());

        #[cfg(target_os = "windows")]
        window.set_visible(true);

        self.window = Some(window);
        if DEBUG_PERF {
            println!("window + Vulkan time: {:?}", self.time.elapsed());
        }
        self.time = Instant::now();
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        println!("exiting");

        if let Some(renderer) = &self.renderer {
            let mut renderer = renderer.borrow_mut();
            renderer.destroy(&mut self.render_assets);
        }
    }
}
