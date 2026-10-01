#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::sync::Arc;
use std::time::Duration;

use pretty_assertions::assert_eq;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream, duplex};
use tokio::sync::Barrier;

use super::{MessageHandler, ServeError, serve};
use crate::framing::{FrameReader, write_frame};

/// Responde con el mismo mensaje, salvo `"quiet"`, que no lleva respuesta.
struct Echo;

impl MessageHandler for Echo {
    fn handle(&self, message: Vec<u8>) -> impl Future<Output = Option<Vec<u8>>> + Send {
        std::future::ready((message != b"quiet").then_some(message))
    }
}

/// Solo responde cuando hay dos peticiones atendiendose a la vez: si el servidor
/// las atendiera de una en una, la primera esperaria para siempre.
struct NeedsTwoAtOnce(Barrier);

impl MessageHandler for NeedsTwoAtOnce {
    async fn handle(&self, message: Vec<u8>) -> Option<Vec<u8>> {
        self.0.wait().await;
        Some(message)
    }
}

async fn read_all_frames(mut output: DuplexStream) -> Vec<Vec<u8>> {
    let mut bytes = Vec::new();
    output.read_to_end(&mut bytes).await.unwrap();
    let mut reader = FrameReader::new(bytes.as_slice());
    let mut frames = Vec::new();
    while let Some(frame) = reader.read_frame().await.unwrap() {
        frames.push(frame);
    }
    frames
}

#[tokio::test]
async fn every_request_gets_its_response_and_notifications_get_none() {
    let (mut client, server_in) = duplex(64 * 1024);
    let (server_out, output) = duplex(64 * 1024);
    for body in [b"one".as_slice(), b"quiet", b"two"] {
        write_frame(&mut client, body).await.unwrap();
    }
    drop(client);

    serve(Arc::new(Echo), server_in, server_out).await.unwrap();

    let mut frames = read_all_frames(output).await;
    frames.sort();
    assert_eq!(frames, vec![b"one".to_vec(), b"two".to_vec()]);
}

#[tokio::test]
async fn requests_are_served_concurrently() {
    let (mut client, server_in) = duplex(64 * 1024);
    let (server_out, output) = duplex(64 * 1024);
    write_frame(&mut client, b"first").await.unwrap();
    write_frame(&mut client, b"second").await.unwrap();
    drop(client);
    let handler = Arc::new(NeedsTwoAtOnce(Barrier::new(2)));

    tokio::time::timeout(
        Duration::from_secs(10),
        serve(handler, server_in, server_out),
    )
    .await
    .expect("requests were served one at a time")
    .unwrap();

    assert_eq!(read_all_frames(output).await.len(), 2);
}

#[tokio::test]
async fn a_frame_that_is_too_large_closes_the_connection_after_answering_the_rest() {
    let (mut client, server_in) = duplex(64 * 1024);
    let (server_out, output) = duplex(64 * 1024);
    write_frame(&mut client, b"before").await.unwrap();
    client
        .write_all(b"Content-Length: 99999999999\r\n\r\n")
        .await
        .unwrap();

    let error = serve(Arc::new(Echo), server_in, server_out)
        .await
        .unwrap_err();

    assert!(matches!(error, ServeError::Frame(_)), "{error}");
    assert!(error.to_string().contains("exceeds"));
    assert_eq!(read_all_frames(output).await, vec![b"before".to_vec()]);
}

#[tokio::test]
async fn a_closed_output_is_reported() {
    let (mut client, server_in) = duplex(64 * 1024);
    let (server_out, output) = duplex(16);
    drop(output);
    write_frame(&mut client, b"nobody is listening")
        .await
        .unwrap();
    drop(client);

    let error = serve(Arc::new(Echo), server_in, server_out)
        .await
        .unwrap_err();

    assert!(matches!(error, ServeError::Write(_)), "{error}");
}
