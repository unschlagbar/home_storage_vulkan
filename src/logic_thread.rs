use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::{Arc, mpsc::Receiver},
};

use winit::event_loop::EventLoopProxy;

use crate::{
    network::Network,
    properties_view::PropertiesView,
    thread_event::{LogicEvent, RenderEvent},
};

pub struct Logic {
    #[allow(unused)]
    pub net: Arc<Network>,

    pub logic: Receiver<LogicEvent>,
    pub proxy: EventLoopProxy<RenderEvent>,
}

impl Logic {
    pub fn new(logic: Receiver<LogicEvent>, proxy: EventLoopProxy<RenderEvent>) -> Self {
        let net = Arc::new(Network::new(
            SocketAddrV4::new(Ipv4Addr::LOCALHOST, 2350).into(),
        ));
        Self { net, logic, proxy }
    }

    pub fn run(&mut self) {
        self.net.connection.lock().unwrap().send(&[1, 2, 4]);
        loop {
            match self.logic.recv() {
                Ok(LogicEvent::FolderSize(path)) => {
                    let size = PropertiesView::calculate_folder_size(path);
                    let _ = self.proxy.send_event(RenderEvent::FolderSize(size));
                }
                Err(_) => return,
            }
        }
    }
}
