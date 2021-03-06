use html;

#[test]
fn strip_tags() {
    let test_bad_tag = html::strip_tags("<script>Test</script>");
    let test_bad_attribute = html::strip_tags("<a onclick=\"malicious_stuff\">Test</a>");
    assert_eq!("Test", &test_bad_tag);
    assert_eq!("<a >Test</a>", &test_bad_attribute);
}
