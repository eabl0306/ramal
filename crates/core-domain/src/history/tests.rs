use super::SearchQuery;

#[test]
fn a_hex_word_is_searched_as_sha_and_as_text() {
    let query = SearchQuery::parse("  CAFE12 ").unwrap_or_else(|| unreachable!());

    assert_eq!(query.text(), "CAFE12");
    assert_eq!(query.sha_prefix(), Some("cafe12"));
}

#[test]
fn plain_text_and_too_short_hex_are_only_text() {
    for text in ["fix login", "abc", "g123456"] {
        let query = SearchQuery::parse(text).unwrap_or_else(|| unreachable!());
        assert_eq!(query.sha_prefix(), None, "{text}");
    }
}

#[test]
fn an_empty_search_is_no_search() {
    assert_eq!(SearchQuery::parse("   "), None);
}
