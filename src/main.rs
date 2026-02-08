#![cfg(not(target_os = "android"))]
include!(concat!(env!("OUT_DIR"), "/gen_assets.rs"));
include!(concat!(env!("OUT_DIR"), "/include_dirs.rs"));

mod app;
mod asset_manager;
mod explorer;
mod file_size;
mod gen_fef;
mod network;
mod properties_view;
mod render_assets;
mod tooltip_view;
mod utils;
mod vulkan_render;

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};

use winit::event_loop::EventLoop;

use crate::{app::App, network::Network};

#[cfg_attr(target_os = "linux", path = "file_handling/linux.rs")]
#[cfg_attr(target_os = "windows", path = "file_handling/windows.rs")]
pub mod file_handling;

pub fn main() {
    let event_loop = EventLoop::new().unwrap();
    let network = Arc::new(Network::new(
        SocketAddrV4::new(Ipv4Addr::LOCALHOST, 2350).into(),
    ));
    let mut app = App::create(network.clone());

    network.connection.lock().unwrap().send(&[1, 2, 4]);

    event_loop.run_app(&mut app).unwrap();
}
