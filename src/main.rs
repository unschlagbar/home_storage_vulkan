#![cfg(not(target_os = "android"))]
//#![windows_subsystem = "windows"]

include!(concat!(env!("OUT_DIR"), "/gen_assets.rs"));

use std::{net::{Ipv4Addr, SocketAddrV4}, sync::Arc};

use winit::event_loop::EventLoop;

pub mod app;
mod explorer;
mod vulkan_render;
mod network;

use crate::{app::App, network::Network};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let network = Arc::new(Network::new(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 2350).into()));
    let mut app = App::create(network.clone());

    network.connection.lock().unwrap().send(&[1, 2, 4]);

    event_loop.run_app(&mut app).unwrap();

}
