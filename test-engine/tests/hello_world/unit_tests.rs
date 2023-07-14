#[cfg(tests)]
mod hello_word_tests {
    use sdt_test_engine::test_engine::TestEngine;

    #[test]
    fn test_publish() {
        let mut test_engine = TestEngine::new();
        test_engine.new_package("hello world", "test-engine/tests/hello_world/package/");
        // Check the package was indeed published
        test_engine.get_package("hello_world");
    }


}