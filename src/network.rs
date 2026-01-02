use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    sync::{Arc, Mutex},
};

pub struct Network {
    pub connection: Arc<Mutex<Connection>>,
}

impl Network {
    pub fn new(server_addr: SocketAddr) -> Self {
        let stream = if let Ok(stream) = TcpStream::connect(server_addr) {
            println!("connected");
            Some(stream)
        } else {
            None
        };

        Self {
            connection: Arc::new(Mutex::new(Connection::new(stream))),
        }
    }
}

pub struct Connection {
    pub stream: Option<TcpStream>,
    pub connected: bool,
}

impl Connection {
    pub fn new(mut stream: Option<TcpStream>) -> Self {
        let connected = if let Some(stream) = &mut stream {
            stream.set_nodelay(true).unwrap();
            true
        } else {
            false
        };
        Self { stream, connected }
    }

    pub fn send(&mut self, buf: &[u8]) {
        if let Some(stream) = &mut self.stream {
            stream.write(buf).unwrap();
            println!("Sent {:?} bytes", buf.len());
        } else {
            println!("Not connected");
        }
    }
}
