//! Bounded local observability integration tests.

use std::collections::BTreeMap;
use ste_observability::*;
#[test]
fn metrics_and_rotation_are_bounded() {
    let mut m = MetricRegistry::default();
    m.register("q", ["task".into()], 1);
    m.record("q", BTreeMap::from([("task".into(), "a".into())]), 1.0)
        .unwrap();
    assert_eq!(
        m.record("q", BTreeMap::from([("task".into(), "b".into())]), 2.0),
        Err(MetricError::CardinalityExceeded)
    );
    let mut s = RecordStore::new(1);
    for code in ["a", "b"] {
        s.push(Record {
            class: RecordClass::Diagnostic,
            code: code.into(),
            time_ns: 0,
            fields: BTreeMap::new(),
        });
    }
    assert_eq!(s.dropped(RecordClass::Diagnostic), 1);
}
#[test]
fn bundle_is_schema_redacted_and_preview_bound() {
    let mut schema = RedactionSchema::default();
    schema.allow(RecordClass::Diagnostic, "health", ["state".into()]);
    let mut b = SupportBundleBuilder::new(&schema);
    b.add(Record {
        class: RecordClass::Diagnostic,
        code: "health".into(),
        time_ns: 1,
        fields: BTreeMap::from([
            ("state".into(), "ok".into()),
            ("participant".into(), "secret".into()),
        ]),
    });
    let preview = b.preview().unwrap();
    let files = b.export(&preview).unwrap();
    let body = String::from_utf8(files.into_values().next().unwrap()).unwrap();
    assert!(!body.contains("secret"));
    assert!(body.contains("state"));
}
