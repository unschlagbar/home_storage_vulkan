use crate::thread_event::{LogicEvent, RenderEvent};
use crate::logic_thread::Logic;
use crate::render_assets::RenderAssets;
use crate::{explorer::Explorer, vulkan_render::VulkanRender};
use iron_oxide::ui::Ui;
use winit::event_loop::EventLoopProxy;

use std::sync::mpsc::{self, Sender};
use std::thread::spawn;
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Theme, Window},
};

#[cfg(target_os = "linux")]
use winit::platform::wayland::WindowAttributesExtWayland;

#[cfg(target_os = "windows")]
use winit::platform::windows::{CornerPreference, WindowAttributesExtWindows};

const APP_NAME: &str = "Home Server";
const WIDTH: u32 = 1080;
const HEIGHT: u32 = 720;

pub const VSYNC: bool = false;
pub const LIMIT_FPS: bool = true;
pub const ONLY_DRAW_ON_UPDATE: bool = true;
const DEFAULT_FPS: u64 = 60;

pub const DEBUG_PERF: bool = !true;

pub struct App {
    pub window: Option<Window>,
    pub renderer: Option<Rc<RefCell<VulkanRender>>>,
    pub render_assets: RenderAssets,
    pub ui: Rc<RefCell<Ui>>,

    pub explorer: Explorer,
    pub logic: Sender<LogicEvent>,

    pub target_frame_time: Duration,
    pub time: Instant,
}

impl App {
    pub fn create(proxy: EventLoopProxy<RenderEvent>) -> Self {
        let renderer = None;
        let (tx, rx) = mpsc::channel::<LogicEvent>();

        spawn(|| {
            Logic::new(rx, proxy).run();
        });

        let ui = Rc::new(RefCell::new(Ui::create(true)));
        let mut explorer = Explorer::new(ui.clone(), tx.clone());

        explorer.data.display_path();

        Self {
            window: None,
            renderer,
            render_assets: RenderAssets::default(),
            ui,
            explorer,
            logic: tx,
            target_frame_time: Duration::from_millis(1000 / DEFAULT_FPS),
            time: Instant::now(),
        }
    }

    pub fn get_framerate(&mut self, window: &Window) {
        if let Some(monitor) = window.current_monitor()
            && let Some(refresh_rate) = monitor.refresh_rate_millihertz()
        {
            self.target_frame_time = Duration::from_millis(1_000_000 / refresh_rate as u64);
            println!("target frametime: {:?}", self.target_frame_time);
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

    pub fn create_window(&self, event_loop: &ActiveEventLoop) -> Window {
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
