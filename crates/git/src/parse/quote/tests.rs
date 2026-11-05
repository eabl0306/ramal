use super::unquote;

#[test]
fn plain_paths_are_kept_and_quoted_ones_are_decoded() {
    assert_eq!(unquote("src/a file.rs"), "src/a file.rs");
    assert_eq!(unquote("\"tab\\there\""), "tab\there");
    assert_eq!(unquote("\"quo\\\"te\""), "quo\"te");
    assert_eq!(unquote("\"back\\\\slash\""), "back\\slash");
    assert_eq!(unquote("\"line\\nbreak\\r\""), "line\nbreak\r");
    assert_eq!(unquote("\"caf\\303\\251\""), "café");
    assert_eq!(unquote("\"trailing\\\""), "trailing\\");
}
