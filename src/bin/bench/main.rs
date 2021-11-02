//! To properly to micro-benchmarks we need to dig a bit in the hardware available to do this.
//! For reasons explained leater, we are going to use the TimeStamp Counter to benchmark our code.
//!
//! Read the TSC (TimeStamp Counter), this is a MSR (Machine Specific Register)
//! which read the current value in the TSC.
//! The frequency of the TSC changes between different cpus,
//! On linux you can get it using:
//! ```bash
//! $ sudo journalctl --boot | grep 'kernel: tsc:' -i | cut -d' ' -f5-
//! kernel: tsc: Fast TSC calibration using PIT
//! kernel: tsc: Detected 3999.850 MHz processor
//! kernel: tsc: Refined TSC clocksource calibration: 3999.980 MHz
//! ```
//! Therefore, each unit of my TSC is around 0.25ns.
//!
//! My cpu has min frequency of 2.2Ghz, and max ~4.9Ghz (Even tho under load it's usually at 4Ghz).
//! This can be checked using:
//! ```bash
//! $ sudo cat /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_{min,max}_freq
//! 2200000
//! 4917968
//! ```
//!
//! Therefore, since the TSC frequency is constant, while the CPU frequency might change, due
//! to power / termal / usage reasons, a TSC unit might range from less than a cycle to a bit less than two.
//!
//! Ideally we would like to count the cycles, instead of the time, but that requires
//! reading the performance couter IA32_TIME_STAMP_COUNTER which requires to be in ring0 (Kernel)
//! so it's not as easy.
//!
//! TLDR: due to the CPU frequency scaling cycles taks different times, so reading the TSC
//! is not perfect, but it's an easy high-precision approximation.
//!
//! The RDTSC instruction is not serializing, so instructions could be executed out of order and thus
//! maybe do not measure any difference in time between two RDTSC even if there were instructions between them.
//! Therefore we should add fenches (mfence, lfence) before the first measurement, and after the first measurement.
//! [Reference](https://newbedev.com/rdtscp-versus-rdtsc-cpuid)
//!
//! RDTSC writes the lower 32 bits of the TSC in the 32 lower bits of `rax`, and the remaining bits in `rdx`.
//! Therefore the before and after code looks like:
//! ```
//! ; Before
//! prefetch rax ; prefetch the data we will need to read from memory
//! mfence
//! lfence
//! rdtsc
//! shl rdx, 0x20
//! or  rax, rdx
//! mov rax, r12 ; this is free
//!
//! ; code to bench
//!
//! ; After
//! rdtscp
//! lfence
//! shl rdx, 0x20
//! or  rax, rdx
//!
//! ; compute the difference in tsc cycles
//! sub rax, r12
//! ```
//! `lfence` and `mfence` stops the CPU while all the loads are finished.
//!
//! RDTSCP is the serializing version, of RDTSC which should be roughtly equivalent to:
//! ```
//! rdtsc
//! mfence
//! ```
//! The free instruction is due to [Register Renaming](https://en.wikipedia.org/wiki/Register_renaming), basically
//! to be able to execute the instructions out of order, the cpu has around 200 registers (instead of the common 16).
//! So the register scheduler just changes it's table and can do it in 0 cycles.
//!
//! While we might think that we need to compensate for the `shl` and `or` instruction, these are completely undrelated
//! to the code we want to bench, so they will be execute in parallel, during the same cycles of the application.
//! They might introduce a bit of noise if the code to bench highly exploit out of order execution, but it shouldn't
//! be more than a single cycle of noise.
//!
//! Also we must use Prefetch to ensure that the data we will read / write are already in L1 cache, so that we don't
//! measure the time needed to load the data from RAM or L2, L3 Caches.
//!
use std::arch::x86_64::{__rdtscp, _mm_lfence, _mm_mfence, _mm_prefetch, _rdtsc, _MM_HINT_T0};

use simd_parsing::*;
use std::env;
use std::process;

// Wrapper function, these are unsafe because the cpu might not have
// one of the  instruction. If this happens it will crash with
// Illegal Instruction (SigIll).

#[inline(always)]
fn rdtsc() -> u64 {
    unsafe { _rdtsc() }
}

#[inline(always)]
fn rdtscp() -> u64 {
    // this also read IA32_TSC_AUX but I don't think that
    // we need it
    let mut x: u32 = 0;
    unsafe { __rdtscp((&mut x) as _) }
}

#[inline(always)]
fn lfence() {
    unsafe { _mm_lfence() }
}

#[inline(always)]
fn mfence() {
    unsafe { _mm_mfence() }
}

#[inline(always)]
fn prefetch(p: *const i8) {
    unsafe { _mm_prefetch::<_MM_HINT_T0>(p) }
}

const TRIALS: usize = 2_000_000;

macro_rules! bench {
    ($data:expr, $trials:expr, $func:expr, $file:expr) => {
        let mut min = u64::MAX;
        let mut max = 0;
        let mut delta_sum = 0;
        let mut squared_delta_sum = 0;

        // warmup the function
        let _ = $func($data);

        for _ in 0..$trials {
            prefetch($data.as_ptr() as _);
            mfence();
            lfence();
            let start = rdtsc();

            let _ = $func($data);

            let end = rdtscp();
            lfence();

            let delta = (end - start);
            delta_sum += delta;
            squared_delta_sum += (delta * delta);
            min = min.min(delta);
            max = max.max(delta);
        }

        let mean = (delta_sum as f64) / (TRIALS as f64);
        let second_moment = (squared_delta_sum as f64) / (TRIALS as f64);
        let variance = second_moment - (mean * mean);
        write!($file, "{},{},{:.4},{:.4},", min, max, mean, variance.sqrt()).expect("error in writing to file...");
    };

    ($data:expr, $sep:expr, $eol:expr, $trials:expr, $func:expr, $file:expr) => {
        let mut min = u64::MAX;
        let mut max = 0;
        let mut delta_sum = 0;
        let mut squared_delta_sum = 0;

        // warmup the function
        let _ = $func($data, $sep, $eol);

        for _ in 0..$trials {
            prefetch($data.as_ptr() as _);
            mfence();
            lfence();
            let start = rdtsc();

            let _ = $func($data, $sep, $eol);

            let end = rdtscp();
            lfence();

            let delta = (end - start);
            delta_sum += delta;
            squared_delta_sum += (delta * delta);
            min = min.min(delta);
            max = max.max(delta);
        }

        let mean = (delta_sum as f64) / (TRIALS as f64);
        let second_moment = (squared_delta_sum as f64) / (TRIALS as f64);
        let variance = second_moment - (mean * mean);
        write!($file, "{},{},{:.4},{:.4},", min, max, mean, variance.sqrt()).expect("error in writing to file...");
    };
}

fn pin_to_core(core_id: usize) {
    unsafe {
        let mut tmp = std::mem::zeroed::<libc::cpu_set_t>();
        libc::CPU_SET(core_id, &mut tmp);
        libc::sched_setaffinity(libc::getpid(), std::mem::size_of::<libc::cpu_set_t>(), &tmp);
    }
}

/// useless busy loop, we use volatile so that the compiler can't optimize it out
pub fn get_hot() -> usize {
    let mut x = 0;
    // this should take about 2 seconds
    for i in 0_usize..3_000_000_000_usize {
        unsafe {
            let data = std::ptr::read_volatile(&x as _);
            std::ptr::write_volatile((&mut x) as _, i + data);
        }
    }
    x
}

fn print_usage() {
    eprintln!("\n===== USAGE =====");
    eprintln!("cargo run --bin bench INTRINSIC");
    eprintln!("  INTRINSIC: sse41    run benchmark for SSE4.1 instruction set");
    eprintln!("             sse42    run benchmark for SSE4.2 instruction set");
    eprintln!("             avx2     run benchmark for AVX2 instruction set");
}

////////////////////////////////////////////////////////////////////////////////////////////////////

use std::fs;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        eprintln!("Error: not enough arguments. Specify one of [sse41, sse42, avx2] to bench the program.");
        process::exit(1);
    }
    let isa = &args[1].to_lowercase();

    if isa != "sse41" && isa != "sse42" && isa != "avx2" {
        print_usage();
        eprintln!("Unkwnow input paramter");
        process::exit(1);
    }
    
    let file_handler = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(format!("{}.csv", isa));
    let mut file: Box<dyn Write> = match file_handler {
        Ok(f) => Box::new(f),
        Err(_) => Box::new(io::stdout()),
    };
    // pin to a core to try to avoid that that the scheduler
    // move us to another core, potentially during a measurement
    // set this to a relatively free core
    pin_to_core(4);
    // do some useless computation, to force the scheduler to remove
    // other processes from this core.
    get_hot();

    file.write(b"len,").expect("error in writing to file...");
    write!(file,
           "{func_name}_min,{func_name}_max,{func_name}_mean,{func_name}_std,",
           func_name = "std"
    ).expect("error in writing to file...");
    write!(file,
           "{func_name}_min,{func_name}_max,{func_name}_mean,{func_name}_std,",
           func_name = "parse_integer_no_simd"
    ).expect("error in writing to file...");
    write!(file,
           "{func_name}_min,{func_name}_max,{func_name}_mean,{func_name}_std,",
           func_name = "std_delimeter"
    ).expect("error in writing to file...");
    write!(file,
           "{func_name}_min,{func_name}_max,{func_name}_mean,{func_name}_std,",
           func_name = "parse_integer_no_simd_delimeter"
    ).expect("error in writing to file...");
    write!(file,
           "{func_name}_min,{func_name}_max,{func_name}_mean,{func_name}_std,",
           func_name = "parse_integer_simd_delimeter"
    ).expect("error in writing to file...");
    file.write(b"\n").expect("error in writing to file...");

    if isa == "sse41" {
        bench_sse41(10, &mut file);
    } else if isa == "sse42" {
        bench_sse42(10, &mut file);
    } else { // avx2 branch
        bench_avx2(10, &mut file);
    }
}

fn safe_parse_integer_sse41(s: &str, separator: u8, eol: u8) -> Option<u32> {
    unsafe { return parse_integer_sse41(s, separator, eol); }
}

fn safe_parse_integer_avx2(s: &str, separator: u8, eol: u8) -> Option<u32> {
    unsafe { return parse_integer_avx2(s, separator, eol); }
}

fn bench_sse41(times: usize, file: &mut dyn Write) {
    for l in 1..=times {
        write!(file, "{},", l).expect("error in writing to file...");
        // generate a number to parse
        let number_to_parse = (0..l).map(|_| "1").collect::<Vec<_>>().join("");
        bench!(number_to_parse.as_str(), TRIALS, std_test, file);
        bench!(&number_to_parse, b',', b'\n', TRIALS, safe_parse_integer_sse41, file);
        // generate a number of 15 digits with a comma. In this way, no SIMD is used
        let mut vec = (0..15).map(|_| "1").collect::<Vec<_>>();
        vec[l] = ",";
        let number_to_parse = vec.join("");
        bench!(&number_to_parse, TRIALS, std_delimeter_test, file);
        bench!(&number_to_parse, b',', b'\n', TRIALS, safe_parse_integer_sse41, file);
        // generate a 16 chars string to use SIMD and place a comma
        let mut vec = (0..16).map(|_| "1").collect::<Vec<_>>();
        vec[l] = ",";
        let number_to_parse = vec.join("");
        bench!(&number_to_parse, b',', b'\n', TRIALS, safe_parse_integer_sse41, file);
        file.write(b"\n").expect("error in writing to file...");
    }
}

fn bench_sse42(times: usize, file: &mut dyn Write) {
    panic!("not implemented");
}

fn bench_avx2(times: usize, file: &mut dyn Write) {
    for l in 1..=times {
        write!(file, "{},", l).expect("error in writing to file...");
        // generate a number to parse
        let number_to_parse = (0..l).map(|_| "1").collect::<Vec<_>>().join("");
        bench!(number_to_parse.as_str(), TRIALS, std_test, file);
        bench!(&number_to_parse, b',', b'\n', TRIALS, safe_parse_integer_avx2, file);
        // generate a number of 31 digits with a comma. In this way, no SIMD is used
        let mut vec = (0..31).map(|_| "1").collect::<Vec<_>>();
        vec[l] = ",";
        let number_to_parse = vec.join("");
        bench!(&number_to_parse, TRIALS, std_delimeter_test, file);
        bench!(&number_to_parse, b',', b'\n', TRIALS, safe_parse_integer_avx2, file);
        // generate a 32 chars string to use SIMD and place a comma
        let mut vec = (0..32).map(|_| "1").collect::<Vec<_>>();
        vec[l] = ",";
        let number_to_parse = vec.join("");
        bench!(&number_to_parse, b',', b'\n', TRIALS, safe_parse_integer_avx2, file);
        file.write(b"\n").expect("error in writing to file...");
    }
}

#[inline(always)]
fn std_test(number: &str) -> u32 {
    number.parse().unwrap()
}

fn std_delimeter_test(number: &str) -> u32 {
    let to_parse = number.split(',').next().unwrap();
    to_parse.parse().unwrap()
}

fn bench_len(l: usize, file: &mut dyn Write) {
    write!(file, "{},", l).expect("error in writing to file...");
    // generate a number to parse
    let number_to_parse = (0..l).map(|_| "1").collect::<Vec<_>>().join("");
    bench!(number_to_parse.as_str(), TRIALS, std_test, file);
    bench!(&number_to_parse, b',', b'\n', TRIALS, parse_integer, file);
    // generate a number of 15 digits with a comma. In this way, no SIMD is used
    let mut vec = (0..15).map(|_| "1").collect::<Vec<_>>();
    vec[l] = ",";
    let number_to_parse = vec.join("");
    bench!(&number_to_parse, TRIALS, std_delimeter_test, file);
    bench!(&number_to_parse, b',', b'\n', TRIALS, parse_integer, file);
    // generate a 16 chars string to use SIMD and place a comma
    let mut vec = (0..16).map(|_| "1").collect::<Vec<_>>();
    vec[l] = ",";
    let number_to_parse = vec.join("");
    bench!(&number_to_parse, b',', b'\n', TRIALS, parse_integer, file);
    file.write(b"\n").expect("error in writing to file...");
}
