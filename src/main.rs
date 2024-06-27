use std::{
    collections::HashMap,
    fmt::Write as FmtWrite,
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    result,
};

const IP: &str = "127.0.0.1";
const PORT: u16 = 6969;
const OK_STATUS: &str = "HTTP/1.1 200 OK";
const NOT_FOUND_STATUS: &str = "HTTP/1.1 404 NOT FOUND";
const HOME: &[u8; 16] = b"GET / HTTP/1.1\r\n";
const ADD_TODO: &[u8; 22] = b"POST /todos HTTP/1.1\r\n";
const DELETE_TODO: &[u8; 13] = b"DELETE /todos";

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

fn parse_add_request(request: &str) -> Option<&str> {
    request
        .lines()
        .filter(|line| line.starts_with("todo"))
        .find_map(|line| line.split_once("=").map(|(_, todo)| todo))
}

fn parse_delete_request(request: &str) -> Option<u64> {
    request
        .lines()
        .filter(|line| line.starts_with("DELETE"))
        .find_map(|line| {
            line.split_whitespace()
                .filter(|part| part.starts_with("/todos"))
                .flat_map(|part| part.split("/"))
                .find_map(|p| p.parse::<u64>().ok())
        })
}

fn todo_hashmap_to_string(todos: &mut HashMap<u64, String>) -> String {
    let mut serialized_data = String::new();
    for (key, value) in todos {
        writeln!(&mut serialized_data,
                "<li>{} <button hx-delete=\"/todos/{}\" hx-swap=\"innerHTML\" hx-target=\"#todos\">delete</button></li>\n",
                value, key
            ).unwrap();
    }

    serialized_data
}

fn handle_client(
    mut stream: TcpStream,
    todos: &mut HashMap<u64, String>,
    todos_count: &mut u64,
) -> Result<()> {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).map_err(|err| {
        eprintln!("ERROR: Cannot read the connection stream from client: {err}");
    })?;

    let request = String::from_utf8_lossy(&buffer[..]);

    if buffer.starts_with(HOME) {
        let index_page = fs::read_to_string("index.html").unwrap();
        respond(stream, OK_STATUS.to_string(), index_page)?;
    } else if buffer.starts_with(ADD_TODO) {
        let todo = parse_add_request(&request).unwrap();

        todos.insert(*todos_count, todo.to_string());
        *todos_count += 1;

        let serialized_data = todo_hashmap_to_string(todos);

        respond(stream, OK_STATUS.to_string(), serialized_data)?;
    } else if buffer.starts_with(DELETE_TODO) {
        let todo_id = parse_delete_request(&request).unwrap();

        todos.remove(&todo_id);

        let serialized_data = todo_hashmap_to_string(todos);

        respond(stream, OK_STATUS.to_string(), serialized_data)?;
    } else {
        println!("{}", String::from_utf8_lossy(&buffer[..]));
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
    let mut todos: HashMap<u64, String> = HashMap::new();
    let mut todos_count: u64 = 1;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let _ = handle_client(stream, &mut todos, &mut todos_count);
            }
            Err(err) => {
                eprintln!("ERROR: Cannot accept connection: {err}");
            }
        }
    }

    Ok(())
}
