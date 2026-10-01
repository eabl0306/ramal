//! Cliente HTTP/1.1 minimo sobre `TcpStream`.
//!
//! Basta para las sondas de estos tests y evita traer un cliente HTTP entero
//! como dependencia de test.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

/// Respuesta leida del socket.
#[derive(Debug)]
pub struct HttpResponse {
    /// Codigo de estado.
    pub status: u16,
    /// Cabeceras en crudo, en minusculas.
    pub headers: String,
    /// Cuerpo en crudo.
    pub body: String,
}

/// Hace un `GET` y lee la respuesta entera.
pub fn get(port: u16, path: &str) -> std::io::Result<HttpResponse> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )?;
    let mut raw = String::new();
    stream.read_to_string(&mut raw)?;
    Ok(parse(&raw))
}

/// Repite el `GET` hasta que responda con `status` o pase el plazo.
pub fn get_until(port: u16, path: &str, status: u16, within: Duration) -> HttpResponse {
    let deadline = Instant::now() + within;
    loop {
        let response = get(port, path).expect("the server should answer");
        if response.status == status || Instant::now() >= deadline {
            return response;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn parse(raw: &str) -> HttpResponse {
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw, ""));
    let (status_line, headers) = head.split_once("\r\n").unwrap_or((head, ""));
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    HttpResponse {
        status,
        headers: headers.to_ascii_lowercase(),
        body: body.to_owned(),
    }
}
