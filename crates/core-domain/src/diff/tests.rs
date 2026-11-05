use super::{DiffContent, DiffOptions, LfsPointer, Whitespace, empty_text, is_image_extension};

const POINTER: &str = "version https://git-lfs.github.com/spec/v1\n\
                       oid sha256:4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393\n\
                       size 12345\n";

#[test]
fn an_lfs_pointer_gives_its_object_and_size() {
    assert_eq!(
        LfsPointer::parse(POINTER),
        Some(LfsPointer {
            oid: "sha256:4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393"
                .to_owned(),
            size: 12_345,
        })
    );
}

#[test]
fn text_that_only_looks_like_a_pointer_is_not_one() {
    let cases = [
        "hello\n",
        "",
        "version https://git-lfs.github.com/spec/v1\nsize 3\n",
        "version https://git-lfs.github.com/spec/v1\noid sha256:x\nsize many\n",
        "version https://git-lfs.github.com/spec/v1\nnot a key value\n",
    ];
    for text in cases {
        assert_eq!(LfsPointer::parse(text), None, "{text:?}");
    }
}

#[test]
fn pointers_with_crlf_line_endings_are_read() {
    let crlf = POINTER.replace('\n', "\r\n");
    assert_eq!(LfsPointer::parse(&crlf).map(|p| p.size), Some(12_345));
}

#[test]
fn images_are_recognized_by_extension() {
    assert!(is_image_extension("png"));
    assert!(is_image_extension("jpeg"));
    assert!(
        !is_image_extension("svg"),
        "svg is text and gets a text diff"
    );
    assert!(!is_image_extension("rs"));
}

#[test]
fn defaults_match_plain_git_diff() {
    let options = DiffOptions::default();
    assert_eq!(options.whitespace, Whitespace::Exact);
    assert!(!options.word_diff);
    assert_eq!(options.context_lines, 3);
    assert_eq!(empty_text(), DiffContent::Text { hunks: Vec::new() });
}
