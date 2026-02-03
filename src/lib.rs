#![cfg(target_os = "android")]

include!(concat!(env!("OUT_DIR"), "/gen_assets.rs"));
include!(concat!(env!("OUT_DIR"), "/include_dirs.rs"));

mod app;
mod asset_manager;
mod explorer;
#[path = "file_handling/linux.rs"]
mod file_handling;
mod file_size;
mod network;
mod properties_view;
mod render_assets;
mod tooltip_view;
mod utils;
mod vulkan_render;

use crate::{app::App, network::Network};
use android::activity::AndroidApp;
use std::panic;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};
use winit::event_loop::{EventLoop, EventLoopBuilder};
use winit::platform::android::{self, EventLoopBuilderExtAndroid};

#[unsafe(no_mangle)]
pub fn android_main(app: AndroidApp) {
    //panic::set_hook(Box::new(|info| {
    //    log::error!("Panic occurred: {:?}", info);
    //}));

    //android_logger::init_once(
    //    android_logger::Config::default().with_max_level(log::LevelFilter::max()),
    //);

    let event_loop: EventLoop<()> = EventLoopBuilder::default()
        .with_android_app(app)
        .build()
        .unwrap();

    let network = Arc::new(Network::new(
        SocketAddrV4::new(Ipv4Addr::LOCALHOST, 2350).into(),
    ));
    let mut app = App::create(network.clone());

    network.connection.lock().unwrap().send(&[1, 2, 4]);

    event_loop.run_app(&mut app).unwrap();
}
