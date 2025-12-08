#![cfg(not(target_os = "android"))]
//#![windows_subsystem = "windows"]

include!(concat!(env!("OUT_DIR"), "/gen_icons.rs"));

use winit::event_loop::EventLoop;

pub mod app;
mod explorer;
mod vulkan_render;

use crate::app::App;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}
