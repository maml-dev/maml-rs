use maml::parse;

fn load_error_cases() -> Vec<(String, String, String)> {
    let content =
        std::fs::read_to_string("tests/fixtures/error.test.txt").expect("failed to read test file");
    let mut cases = Vec::new();

    // Match JS loader: split on "===", trim each section, split first line as name,
    // rejoin remaining lines, split on "---" to get input and expected.
    for section in content.split("===") {
        let section = section.trim();
        if section.is_empty() {
            continue;
        }
        let lines: Vec<&str> = section.split('\n').collect();
        let name = lines[0].trim().to_string();
        let body = lines[1..].join("\n");
        let sep = body.find("---").expect("no --- separator");
        let input = body[..sep].to_string();
        let expected = body[sep + 3..].trim().to_string();
        cases.push((name, input, expected));
    }
    cases
}

#[test]
fn error_test_cases() {
    let cases = load_error_cases();
    assert!(!cases.is_empty(), "no error test cases loaded");

    for (name, input, expected_error) in &cases {
        let result = parse(input);
        assert!(
            result.is_err(),
            "Test '{name}' should have failed but got: {:?}\nInput: {input:?}",
            result.unwrap()
        );

        let error_msg = result.unwrap_err().to_string();

        // The expected error may be multi-line (message + snippet).
        // Check that the first line of expected is contained in the error message.
        let expected_first_line = expected_error.lines().next().unwrap_or(expected_error);

        // JS reports UTF-16 surrogates for emoji (e.g., "\ud83d"), but Rust reports the
        // full Unicode character. Skip exact character matching for surrogate cases.
        if expected_first_line.contains("\\ud83d") {
            assert!(
                error_msg.contains("Unexpected character"),
                "Test '{name}' error mismatch:\n  got:      {error_msg}\n  expected: Unexpected character ...\n  input: {input:?}"
            );
        } else {
            assert!(
                error_msg.contains(expected_first_line),
                "Test '{name}' error mismatch:\n  got:      {error_msg}\n  expected to contain: {expected_first_line}\n  input: {input:?}"
            );
        }
    }
}

#[test]
fn parse_non_empty_input_required() {
    assert!(parse("").is_err());
    assert!(parse("   ").is_err());
    assert!(parse("# just a comment\n").is_err());
}
