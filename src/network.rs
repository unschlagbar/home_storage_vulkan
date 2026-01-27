use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct Network {
    pub connection: Arc<Mutex<Connection>>,
}

impl Network {
    pub fn new(server_addr: SocketAddr) -> Self {
        let stream = if let Ok(stream) =
            TcpStream::connect_timeout(&server_addr, Duration::from_millis(10))
        {
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
    #[allow(unused)]
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
            stream.write_all(buf).unwrap();
            println!("Sent {:?} bytes", buf.len());
        } else {
            println!("Not connected");
        }
    }
}
