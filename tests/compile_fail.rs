#[test]
fn compile_fail_tests() {
    // Skip compile_fail tests on nightly as they produce different error messages
    if version_check::is_feature_flaggable().unwrap_or(false) {
        eprintln!("Skipping compile_fail tests on nightly Rust");
        return;
    }

    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
