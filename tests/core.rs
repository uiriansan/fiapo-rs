use fiapo::core::test::_test_fn;

#[test]
fn test_structure() {
    let result = _test_fn(34, 35);
    assert_eq!(result, 69);
}
