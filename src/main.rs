#![cfg(not(target_os = "android"))]
include!(concat!(env!("OUT_DIR"), "/gen_assets.rs"));
include!(concat!(env!("OUT_DIR"), "/include_dirs.rs"));

mod app;
mod app_window;
mod asset_manager;
mod explorer;
mod file_size;
mod gen_fef;
mod thread_event;
mod logic_thread;
mod network;
mod properties_view;
mod render_assets;
mod tooltip_view;
mod utils;
mod vulkan_render;

use winit::event_loop::EventLoop;

use crate::{app::App, thread_event::RenderEvent};

#[cfg_attr(target_os = "linux", path = "file_handling/linux.rs")]
#[cfg_attr(target_os = "windows", path = "file_handling/windows.rs")]
pub mod file_handling;

pub fn main() {
    let event_loop: EventLoop<RenderEvent> = EventLoop::with_user_event().build().unwrap();
    let mut app = App::create(event_loop.create_proxy());

    event_loop.run_app(&mut app).unwrap();
}
