# wstressr

More tests to come this is a WIP ( Work in Progress )

A **Simple WebAssembly (WASM) stress and benchmarking tool** built in Rust.  

## Features

- **CPU Integer Benchmarks** (`--int`)
- **CPU Floating-Point Benchmarks** (`--float`)
- **Memory Read/Write Benchmarks** (`--mem`)
- **Memory Copy (memcpy) Benchmarks** (`--memcpy`)
- **Multithreaded Benchmarks** (`--threads N`) with graceful fallback if threads unsupported
- **Environment Probe** (`--probe`) reports runtime capabilities (threads, fs, env, clock resolution)
- JSON-formatted results for easy parsing
- Wasmtime is required for this project 

## Build

### Add WASI Preview 1
rustup target add wasm32-wasip1

### Add WASI Preview 2
rustup target add wasm32-wasip2

### WASI Preview 1
```bash
cargo build --release --target wasm32-wasip1
```

### WASI Preview 2
```bash
cargo build --release --target wasm32-wasip2
```

## Usage
Run benchmarks using wasmtime

### Integer Benchmark
```bash
wasmtime run target/wasm32-wasip1/release/wstressr.wasm -- --int --iterations 1000000
```

### Float Benchmark
```bash
wasmtime run target/wasm32-wasip2/release/wstressr.wasm -- --float --iterations 500000
```

### Memory Benchmark
```bash
wasmtime run target/wasm32-wasip1/release/wstressr.wasm -- --mem --memsize 1048576
```

### Memcpy Benchmark
```bash
wasmtime run target/wasm32-wasip2/release/wstressr.wasm -- --memcpy --memsize 1048576
```

### Multithreading (IF Supported )
```bash
wasmtime run target/wasm32-wasip2/release/wstressr.wasm -- --int --threads 4
```

### Probe Runtime
```bash
wasmtime run target/wasm32-wasip1/release/wstressr.wasm -- --probe
```

#### Example Output
```bash
{
  "runtime": "WASI Preview 1",
  "threads": false,
  "fs": true,
  "env": true,
  "clock_resolution_ns": 1000
}
```

## Testing

### Test on WASAPI Preview 1
```bash
cargo build --release --target wasm32-wasip1
WASM_TARGET=wasm32-wasip1 cargo test
```

### Test on WASAPI Preview 2
```bash
cargo build --release --target wasm32-wasip2
WASM_TARGET=wasm32-wasip2 cargo test
```