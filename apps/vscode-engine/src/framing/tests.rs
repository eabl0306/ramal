#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use pretty_assertions::assert_eq;

use super::{FrameError, FrameReader, MAX_FRAME_BYTES, write_frame};

async fn frames(input: &[u8]) -> Vec<Result<Option<Vec<u8>>, String>> {
    let mut reader = FrameReader::new(input);
    let mut out = Vec::new();
    loop {
        let next = reader.read_frame().await.map_err(|error| error.to_string());
        let stop = !matches!(next, Ok(Some(_)));
        out.push(next);
        if stop {
            return out;
        }
    }
}

async fn first_error(input: &[u8]) -> FrameError {
    FrameReader::new(input).read_frame().await.unwrap_err()
}

#[tokio::test]
async fn back_to_back_frames_are_read_in_order_until_a_clean_end() {
    let input = b"Content-Length: 2\r\n\r\n{}Content-Length: 4\r\n\r\nnull";

    assert_eq!(
        frames(input).await,
        vec![
            Ok(Some(b"{}".to_vec())),
            Ok(Some(b"null".to_vec())),
            Ok(None)
        ]
    );
}

#[tokio::test]
async fn other_headers_are_ignored_and_the_name_is_case_insensitive() {
    let input = b"content-type: application/vscode-jsonrpc; charset=utf-8\r\nCONTENT-LENGTH:  3 \r\n\r\n[1]";

    assert_eq!(frames(input).await[0], Ok(Some(b"[1]".to_vec())));
}

#[tokio::test]
async fn the_length_counts_bytes_not_characters() {
    let body = "\"cafe \u{e9}\"".as_bytes();
    let input = [
        format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes(),
        body,
    ]
    .concat();

    assert_eq!(frames(&input).await[0], Ok(Some(body.to_vec())));
}

#[tokio::test]
async fn a_bare_newline_after_the_header_is_accepted() {
    assert_eq!(
        frames(b"Content-Length: 1\n\n7").await[0],
        Ok(Some(b"7".to_vec()))
    );
}

#[tokio::test]
async fn a_frame_over_32_mib_is_rejected_before_reading_it() {
    let announced = MAX_FRAME_BYTES + 1;
    let input = format!("Content-Length: {announced}\r\n\r\n");

    let error = first_error(input.as_bytes()).await;

    assert!(matches!(error, FrameError::TooLarge(size) if size == announced));
}

#[tokio::test]
async fn a_frame_of_exactly_32_mib_is_allowed() {
    let body = vec![b' '; MAX_FRAME_BYTES];
    let input = [
        format!("Content-Length: {MAX_FRAME_BYTES}\r\n\r\n").as_bytes(),
        &body,
    ]
    .concat();

    let frame = FrameReader::new(input.as_slice())
        .read_frame()
        .await
        .unwrap();

    assert_eq!(frame.map(|body| body.len()), Some(MAX_FRAME_BYTES));
}

#[tokio::test]
async fn headers_without_length_or_with_a_bad_one_are_errors() {
    assert!(matches!(
        first_error(b"Content-Type: x\r\n\r\n{}").await,
        FrameError::MissingLength
    ));
    assert!(matches!(
        first_error(b"\r\n{}").await,
        FrameError::MissingLength
    ));
    assert!(matches!(
        first_error(b"Content-Length: -4\r\n\r\n").await,
        FrameError::InvalidLength(value) if value == "-4"
    ));
}

#[tokio::test]
async fn input_that_ends_inside_a_frame_is_an_error() {
    assert!(matches!(
        first_error(b"Content-Length: 10\r\n").await,
        FrameError::UnexpectedEof
    ));
    assert!(matches!(
        first_error(b"Content-Len").await,
        FrameError::UnexpectedEof
    ));
    assert!(matches!(
        first_error(b"Content-Length: 10\r\n\r\n{}").await,
        FrameError::UnexpectedEof
    ));
}

#[tokio::test]
async fn endless_headers_cannot_fill_the_memory() {
    let long_line = vec![b'x'; 9 * 1024];
    assert!(matches!(
        first_error(&long_line).await,
        FrameError::HeaderTooLong
    ));

    let many = "X-Noise: 1\r\n".repeat(40);
    assert!(matches!(
        first_error(many.as_bytes()).await,
        FrameError::TooManyHeaders
    ));
}

#[tokio::test]
async fn written_frames_read_back_identically() {
    let mut buffer = Vec::new();
    write_frame(&mut buffer, b"{\"jsonrpc\":\"2.0\"}")
        .await
        .unwrap();
    write_frame(&mut buffer, b"[]").await.unwrap();

    assert!(buffer.starts_with(b"Content-Length: 17\r\n\r\n{"));
    assert_eq!(
        frames(&buffer).await,
        vec![
            Ok(Some(b"{\"jsonrpc\":\"2.0\"}".to_vec())),
            Ok(Some(b"[]".to_vec())),
            Ok(None)
        ]
    );
}
