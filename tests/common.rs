use std::process::Command;

/// Path to the built WebAssembly artifact (wasip1 or wasip2)
pub fn wasm_binary_path() -> String {
    let target = std::env::var("WASM_TARGET").unwrap_or_else(|_| "wasm32-wasip2".to_string());
    format!("target/{}/release/wstressr.wasm", target)
}

/// Run Wasmtime on the given CLI args, trying fallback options for wasip2 components
pub fn run_wasmtime(args: &[&str]) -> String {
    let bin_path = wasm_binary_path();
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    let try_cmd = |cmd_args: Vec<String>| {
        let output = Command::new("wasmtime")
            .args(&cmd_args)
            .output()
            .expect("Failed to execute wasmtime");
        (
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )
    };

    // Default module mode
    {
        let mut cmd_args = vec![bin_path.clone()];
        cmd_args.extend(args_vec.clone());
        let (ok, out, _err) = try_cmd(cmd_args);
        if ok {
            return out;
        }
    }

    // Component mode
    {
        let mut cmd_args = vec![
            "run".to_string(),
            "--component".to_string(),
            bin_path.clone(),
            "--".to_string(),
        ];
        cmd_args.extend(args_vec.clone());
        let (ok, out, _err) = try_cmd(cmd_args);
        if ok {
            return out;
        }
    }

    // Explicit invoke
    {
        let mut cmd_args = vec![
            "run".to_string(),
            bin_path.clone(),
            "--invoke".to_string(),
            "run".to_string(),
        ];
        cmd_args.extend(args_vec.clone());
        let (ok, out, _err) = try_cmd(cmd_args);
        if ok {
            return out;
        }
    }

    panic!("Wasmtime execution failed for {}", bin_path);
}

/// Extract JSON block (`[...]` or `{...}`) from stdout
pub fn extract_json(stdout: &str) -> &str {
    let start = stdout.find(|c| c == '[' || c == '{').expect("No JSON start in output");
    let end = stdout.rfind(|c| c == ']' || c == '}').expect("No JSON end in output");
    &stdout[start..=end]
}
