// wasm-regex-tree - WebAssembly visualizer for Rust regular expressions.
// Copyright (C) 2026 Soumendra Ganguly

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("js") => "application/javascript",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn handle(mut stream: TcpStream, root: PathBuf) {
    let request_line = BufReader::new(&stream)
        .lines()
        .next()
        .and_then(|l| l.ok())
        .unwrap_or_default();

    let url_path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .split('?')
        .next()
        .unwrap_or("/");

    let file_path = match url_path {
        "/" => root.join("index.html"),
        p => root.join(p.trim_start_matches('/')),
    };

    match fs::read(&file_path) {
        Ok(body) => {
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                content_type(&file_path),
                body.len()
            );
            let _ = stream.write_all(&body);
        }
        Err(_) => {
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        }
    }
}

fn main() {
    let port = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: serve <port>");
        std::process::exit(1);
    });
    let addr = format!("127.0.0.1:{port}");
    let root = PathBuf::from("app");
    let listener = TcpListener::bind(&addr).expect("bind failed");
    eprintln!("http://{addr}");
    for stream in listener.incoming().flatten() {
        let root = root.clone();
        thread::spawn(move || handle(stream, root));
    }
}
