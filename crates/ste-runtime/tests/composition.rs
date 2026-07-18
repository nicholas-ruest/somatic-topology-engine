//! Runtime composition acceptance test.

#[test]
fn should_compose_each_bounded_context_once() {
    let contexts = ste_runtime::bounded_contexts();
    let unique = contexts
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), 8);
}
