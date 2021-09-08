use std::net::TcpListener;
use std::net::TcpStream;
use std::io::{self, BufRead, Write};

static HOST: &str = "127.0.0.1";
static PORT: &str = "7878";

// Borrowed LinesCodec from https://thepacketgeek.com/rust/tcpstream/lines-codec/
pub struct LinesCodec {
    // Our buffered reader & writers
    reader: io::BufReader<TcpStream>,
    writer: io::LineWriter<TcpStream>,
}

/// Encapsulate a TcpStream with buffered reader/writer functionality
impl LinesCodec {
    pub fn new(stream: TcpStream) -> io::Result<Self> {
        // Both BufReader and LineWriter need to own a stream
        // We can clone the stream to simulate splitting Tx & Rx with `try_clone()`
        let writer = io::LineWriter::new(stream.try_clone()?);
        let reader = io::BufReader::new(stream);
        Ok(Self { reader, writer })
    }
}

impl LinesCodec {
    /// Write the given message (appending a newline) to the TcpStream
    pub fn send_message(&mut self, message: &str) -> io::Result<()> {
        self.writer.write(&message.as_bytes())?;
        // This will also signal a `writer.flush()` for us; thanks LineWriter!
        self.writer.write(&['\n' as u8])?;
        Ok(())
    }

    /// Read a received message from the TcpStream
    pub fn read_message(&mut self) -> io::Result<String> {
        let mut line = String::new();
        // Use `BufRead::read_line()` to read a line from the TcpStream
        self.reader.read_line(&mut line)?;
        line.pop(); // Remove the trailing "\n"
        Ok(line)
    }
}


pub(crate) fn try_connect() -> std::io::Result<LinesCodec> {
    if let Ok(stream) = TcpStream::connect(format!("{}:{}", HOST, PORT)) {
        println!("Connected to host at {}:{}", HOST, PORT);
        LinesCodec::new(stream)
    } else {
        listen()
    }
}

pub(crate) fn listen() -> std::io::Result<LinesCodec> {
    let address = format!("{}:{}", HOST, PORT);
    let listener = TcpListener::bind(&address).expect(&format!("Could not bind to address: {}", &address));
    println!("listening at {}", address);

    match listener.accept() {
        Ok((stream, addr)) => {
            println!("accepted connection from {}", addr);
            LinesCodec::new(stream)
        }
        Err(e) => Err(e)
    }
}
