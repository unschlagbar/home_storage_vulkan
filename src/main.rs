#![allow(clippy::uninit_assumed_init)]
//#![windows_subsystem = "windows"]

use game::app::App;
use winit::event_loop::EventLoop;

mod game;
mod graphics;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut application = App::run();

    event_loop.run_app(&mut application).unwrap();
    application.renderer.borrow_mut().destroy();
}
