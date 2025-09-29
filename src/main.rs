use std::time::Instant;
use std::hint::black_box;
use std::cell::UnsafeCell;
use std::env;
use std::thread;
use serde::Serialize;

const MAX_ITERATIONS: u64 = 4_294_967_295;
const MAX_MEM_SIZE: u64 = 4_294_967_295;

struct SyncUnsafeCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for SyncUnsafeCell<T> {}

static INT_SUM_SINK: SyncUnsafeCell<u64> = SyncUnsafeCell(UnsafeCell::new(0));

#[derive(Serialize)]
struct BenchResult {
    name: &'static str,
    duration_ns_total: u128,
    duration_ns_avg: u128,
    threads: usize,
}

#[derive(Serialize)]
struct ProbeResult {
    runtime: &'static str,
    threads: bool,
    fs: bool,
    env: bool,
    clock_resolution_ns: Option<u128>,
}

// Simple Benchmarks

fn cpu_integer_bench(iterations: u64, debug: bool) -> u64 {
    let mut sum = 0u64;
    for i in 0..iterations {
        sum = sum.wrapping_add(i);
    }
    unsafe {
        std::ptr::write_volatile(INT_SUM_SINK.0.get(), sum);
    }
    black_box(sum);

    if debug {
        println!("[debug] Final sum = {}", sum);
    }
    sum
}

fn cpu_float_bench(iterations: u64, debug: bool) -> f64 {
    let mut f = 1.0f64;
    for i in 1..iterations {
        f *= (i as f64).sqrt();
        f = f.ln();
    }
    black_box(f);

    if debug {
        println!("[debug] Final float = {}", f);
    }
    f
}

fn memory_rw_bench(mem_size: u64, debug: bool) {
    let size = mem_size as usize;
    let mut data = vec![0u8; size];
    for i in 0..size {
        data[i] = (i % 256) as u8;
    }
    black_box(&data);

    if debug {
        println!("[debug] Memory last byte = {}", data[size - 1]);
    }
}

fn memcpy_bench(mem_size: u64, debug: bool) {
    let size = mem_size as usize;
    let src = vec![42u8; size];
    let mut dst = vec![0u8; size];
    dst.copy_from_slice(&src);
    black_box(&dst);

    if debug {
        println!("[debug] Memcpy last byte = {}", dst[size - 1]);
    }
}

// Simple Threaded Runner

fn run_in_threads<F>(
    threads: usize,
    name: &'static str,
    f: F,
    threads_supported: bool,
) -> BenchResult
where
    F: Fn() + Send + Sync + 'static + Clone,
{
    if threads <= 1 || !threads_supported {
        if threads > 1 && !threads_supported {
            eprintln!("[error] Threads not supported in this runtime. Falling back to 1 thread.");
        }

        let start = Instant::now();
        f();
        let duration = start.elapsed();
        return BenchResult {
            name,
            duration_ns_total: duration.as_nanos(),
            duration_ns_avg: duration.as_nanos(),
            threads: 1,
        };
    }

    let f = std::sync::Arc::new(f);
    let mut handles = vec![];

    let start = Instant::now();

    for _ in 0..threads {
        let f_clone = f.clone();
        let handle = thread::spawn(move || {
            f_clone();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    BenchResult {
        name,
        duration_ns_total: duration.as_nanos(),
        duration_ns_avg: duration.as_nanos() / threads as u128,
        threads,
    }
}

// Probe Implementation based on Runtime

#[cfg(target_env = "p1")]
fn run_probe() -> ProbeResult {
    ProbeResult {
        runtime: "WASI Preview 1",
        threads: false,
        fs: true,
        env: true,
        clock_resolution_ns: Some(1_000),
    }
}

#[cfg(target_env = "p2")]
fn run_probe() -> ProbeResult {
    ProbeResult {
        runtime: "WASI Preview 2",
        threads: true,
        fs: true,
        env: true,
        clock_resolution_ns: Some(100),
    }
}

#[cfg(not(any(target_env = "p1", target_env = "p2")))]
fn run_probe() -> ProbeResult {
    ProbeResult {
        runtime: "Host Build",
        threads: true,
        fs: true,
        env: true,
        clock_resolution_ns: Some(50),
    }
}

// CLI Handling

fn parse_arg_value(args: &[String], key: &str, default: u64, max: u64) -> u64 {
    match args
        .iter()
        .position(|x| x == key)
        .and_then(|idx| args.get(idx + 1))
        .and_then(|val| val.parse::<u64>().ok())
    {
        Some(v) if v <= max => v,
        Some(_) => {
            eprintln!("[error] Value for {} exceeds max allowed: {}", key, max);
            std::process::exit(1);
        }
        None => default,
    }
}

fn parse_thread_count(args: &[String]) -> usize {
    args.iter()
        .position(|x| x == "--threads")
        .and_then(|idx| args.get(idx + 1))
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(1)
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let debug = args.contains(&"--debug".to_string());

    if args.contains(&"--probe".to_string()) {
        let probe = run_probe();
        let json = serde_json::to_string_pretty(&probe).unwrap();
        println!("{}", json);
        return;
    }

    let iterations = parse_arg_value(&args, "--iterations", 10_000_000, MAX_ITERATIONS);
    let mem_size = parse_arg_value(&args, "--memsize", 1024 * 1024, MAX_MEM_SIZE);
    let threads = parse_thread_count(&args);

    let probe = run_probe();
    let threads_supported = probe.threads;

    let run_int = args.contains(&"--int".to_string());
    let run_float = args.contains(&"--float".to_string());
    let run_mem = args.contains(&"--mem".to_string());
    let run_memcpy = args.contains(&"--memcpy".to_string());

    let run_any = run_int || run_float || run_mem || run_memcpy;

    if !run_any {
        println!("[info] No benchmark flags provided. Running all benchmarks by default.");
    }

    println!(
        "Using iterations: {}, mem_size: {} bytes, threads: {}",
        iterations, mem_size, threads
    );

    let mut results = vec![];

    if run_int || !run_any {
        results.push(run_in_threads(
            threads,
            "cpu_integer_add",
            move || {
                cpu_integer_bench(iterations, debug);
            },
            threads_supported,
        ));
    }
    if run_float || !run_any {
        results.push(run_in_threads(
            threads,
            "cpu_float_ops",
            move || {
                cpu_float_bench(iterations, debug);
            },
            threads_supported,
        ));
    }
    if run_mem || !run_any {
        results.push(run_in_threads(
            threads,
            "memory_rw",
            move || {
                memory_rw_bench(mem_size, debug);
            },
            threads_supported,
        ));
    }
    if run_memcpy || !run_any {
        results.push(run_in_threads(
            threads,
            "memcpy",
            move || {
                memcpy_bench(mem_size, debug);
            },
            threads_supported,
        ));
    }

    let json = serde_json::to_string_pretty(&results).unwrap();
    println!("{}", json);
}
