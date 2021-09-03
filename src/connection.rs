use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

static HOST: &str = "127.0.0.1";
static PORT: &str = "7878";

pub(crate) fn try_connect() -> std::io::Result<TcpStream> {
    if let Ok(stream) = TcpStream::connect(format!("{}:{}", HOST, PORT)) {
        println!("Connected to host at {}:{}", HOST, PORT);
        Ok(stream)
    } else {
        listen()
    }
}

pub(crate) fn listen() -> std::io::Result<TcpStream> {
    let address = format!("{}:{}", HOST, PORT);
    let listener = TcpListener::bind(&address).expect(&format!("Could not bind to address: {}", &address));
    println!("listening at {}", address);

    match listener.accept() {
        Ok((stream, addr)) => {
            println!("accepted connection from {}", addr);
            Ok(stream)
        }
        Err(e) => Err(e)
    }
}

pub(crate) fn handle_stream(mut stream: TcpStream) {
    loop {
        let message = "Move::Up";
        stream.write(message.as_bytes()).unwrap();
        stream.flush().unwrap();

        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        println!("got message: {}", String::from_utf8_lossy(&buffer[..]));

    }
}
