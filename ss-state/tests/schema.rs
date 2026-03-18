use serde_json::json;
use ss_state::{StateFieldSchema, StateValueType};

#[test]
fn rejects_default_outside_enum_values() {
    let error = StateFieldSchema::new(StateValueType::String)
        .with_default(json!("dock"))
        .with_enum_values(vec![json!("tower"), json!("swamp")])
        .validate()
        .expect_err("default outside enum should fail");

    assert!(error.contains("default"));
    assert!(error.contains("enum_values"));
}

#[test]
fn rejects_enum_values_for_non_scalar_types() {
    let error = StateFieldSchema::new(StateValueType::Array)
        .with_enum_values(vec![json!(["dock"]), json!(["tower"])])
        .validate()
        .expect_err("array enums should fail");

    assert!(error.contains("enum_values"));
    assert!(error.contains("scalar"));
}

#[test]
fn accepts_scalar_enum_values_matching_type() {
    StateFieldSchema::new(StateValueType::String)
        .with_default(json!("dock"))
        .with_enum_values(vec![json!("dock"), json!("tower")])
        .validate()
        .expect("matching scalar enum should pass");
}

#[test]
fn ignores_empty_enum_values_as_unset_constraint() {
    StateFieldSchema::new(StateValueType::String)
        .with_default(json!(""))
        .with_enum_values(Vec::new())
        .validate()
        .expect("empty enum_values should be treated as unset");
}
