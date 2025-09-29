mod common;
use serde_json::Value;

#[test]
fn test_memcpy_bench() {
    let stdout = common::run_wasmtime(&["--memcpy", "--memsize", "1024"]);
    println!("wasmtime output: {}", stdout);

    let json_str = common::extract_json(&stdout);
    let result: Value = serde_json::from_str(json_str).unwrap();

    assert_eq!(result[0]["name"], "memcpy");
    assert!(result[0]["duration_ns_total"].as_u64().unwrap() > 0);
}
