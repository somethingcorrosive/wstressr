mod common;
use serde_json::Value;

#[test]
fn test_cpu_integer_bench() {
    let stdout = common::run_wasmtime(&["--int", "--iterations", "1000"]);
    println!("wasmtime output: {}", stdout);

    let json_str = common::extract_json(&stdout);
    let result: Value = serde_json::from_str(json_str).unwrap();

    assert_eq!(result[0]["name"], "cpu_integer_add");
    assert!(result[0]["duration_ns_total"].as_u64().unwrap() > 0);
}
