use fiapo::core::test::test_fn;

#[test]
fn test_structure() {
    let result = test_fn(34, 35);
    assert_eq!(result, 69);
}
