//#![windows_subsystem = "windows"]

use game::app::App;
use winit::event_loop::EventLoop;

mod game;
mod graphics;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::run();

    event_loop.run_app(&mut app).unwrap();
}
