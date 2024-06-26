use std::{
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
    result,
};

const IP: &str = "127.0.0.1";
const PORT: u16 = 6969;

type Result<T> = result::Result<T, ()>;

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).map_err(|err| {
        eprintln!("ERROR: Cannot read the connection stream from client: {err}");
    })?;

    println!("INFO: Read connection into buffer");
    println!("{}", String::from_utf8_lossy(&buffer[..]));

    Ok(())
}

fn main() -> Result<()> {
    let addr = format!("{IP}:{PORT}").parse::<SocketAddr>().unwrap();

    let listener = TcpListener::bind(addr).map_err(|err| {
        eprintln!("ERROR: Cannot bind to {addr}: {err}");
    })?;

    println!("INFO: Listening on {addr}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let _ = handle_client(stream);
            }
            Err(err) => {
                eprintln!("ERROR: Cannot accept connection: {err}");
            }
        }
    }

    Ok(())
}
