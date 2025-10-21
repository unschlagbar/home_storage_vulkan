//#![windows_subsystem = "windows"]

include!(concat!(env!("OUT_DIR"), "/gen_icons.rs"));

mod app;
mod explorer;
mod vulkan_render;

use winit::event_loop::EventLoop;

use crate::app::App;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::run();

    event_loop.run_app(&mut app).unwrap();
}
