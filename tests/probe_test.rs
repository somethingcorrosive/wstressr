mod common;
use serde_json::Value;

#[test]
fn test_probe() {
    let stdout = common::run_wasmtime(&["--probe"]);
    println!("wasmtime output: {}", stdout);

    let json_str = common::extract_json(&stdout);
    let result: Value = serde_json::from_str(json_str).unwrap();

    assert!(result["runtime"].is_string());
    assert!(result["fs"].is_boolean());
    assert!(result["env"].is_boolean());
}
