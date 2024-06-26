use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    result,
};

const IP: &str = "127.0.0.1";
const PORT: u16 = 6969;
const OK_STATUS: &str = "HTTP/1.1 200 OK";
const NOT_FOUND_STATUS: &str = "HTTP/1.1 404 NOT FOUND";

type Result<T> = result::Result<T, ()>;

fn respond(mut stream: TcpStream, status_line: String, content: String) -> Result<()> {
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content.len(),
        content
    );

    stream.write(response.as_bytes()).map_err(|err| {
        eprintln!("ERROR: Cannot send response: {err}");
    })?;

    stream.flush().map_err(|err| {
        eprintln!("ERROR: Failed to flush the stream: {err}");
    })?;

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).map_err(|err| {
        eprintln!("ERROR: Cannot read the connection stream from client: {err}");
    })?;

    let home = b"GET / HTTP/1.1\r\n";
    let test = b"GET /test HTTP/1.1\r\n";

    if buffer.starts_with(home) {
        let index_page = fs::read_to_string("index.html").unwrap();
        respond(stream, OK_STATUS.to_string(), index_page)?;
    } else if buffer.starts_with(test) {
        let message = String::from("HELLO WORLD");
        respond(stream, OK_STATUS.to_string(), message)?;
    } else {
        let not_found = fs::read_to_string("404.html").unwrap();
        respond(stream, NOT_FOUND_STATUS.to_string(), not_found)?;
    }

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
