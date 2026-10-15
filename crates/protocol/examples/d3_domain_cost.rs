//! D3's candidate measurements (`docs/15` D3): one draw on a declared shape near a frontier, the
//! pool filled to its byte limit, or records refused by the node; bytes requested through a
//! counting allocator, process memory from `/proc/self/status`. Usage: `d3_domain_cost <shape>`.

use network::cid::{cid, Cid};
use protocol::contested::{ContestedPool, DrawError, RecordError};
use protocol::results::{EpochResults, ResultRecord, ResultsState};
use scoring::dtf::{ClassCurves, DTF_MAX};
use scoring::latent::Ability;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering::Relaxed};
use std::time::Instant;

struct Counting;
static LIVE: AtomicIsize = AtomicIsize::new(0);
static PEAK: AtomicIsize = AtomicIsize::new(0);

fn moved(bytes: isize) {
    let live = LIVE.fetch_add(bytes, Relaxed) + bytes;
    PEAK.fetch_max(live, Relaxed);
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            moved(layout.size() as isize);
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        moved(-(layout.size() as isize));
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let p = unsafe { System.realloc(ptr, layout, size) };
        if !p.is_null() {
            moved(size as isize - layout.size() as isize);
        }
        p
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

/// `VmRSS` and `VmHWM` of this process, in KiB.
fn rss() -> (u64, u64) {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let field = |name: &str| {
        status
            .lines()
            .find(|l| l.starts_with(name))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    };
    (field("VmRSS:"), field("VmHWM:"))
}

fn normal() -> Ability {
    let nodes: Vec<f64> = (0..41).map(|q| -5.0 + 0.25 * f64::from(q)).collect();
    let mass: Vec<f64> = nodes.iter().map(|u| (-0.5 * u * u).exp()).collect();
    let total: f64 = mass.iter().sum();
    Ability {
        nodes,
        weights: mass.iter().map(|m| m / total).collect(),
    }
}

/// `a` and `b` of `k` items leaning ±0.9 in turn over `classes` equal classes.
fn items(k: usize, classes: usize) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let shift = [-1.0, 1.0, 0.0, 0.0];
    let b = (0..classes)
        .map(|g| {
            (0..k)
                .map(|j| -1.0 + 2.0 * j as f64 / k.max(2) as f64)
                .enumerate()
                .map(|(j, b)| b + if j % 2 == 0 { 0.9 } else { -0.9 } * shift[g % 4])
                .collect()
        })
        .collect();
    (vec![vec![1.25; k]; classes], b)
}

/// [`items`]' curves: the fixed 41-point grid for one class, else a 41-node normal ability,
/// `Q = 41 · classes`.
fn curves(k: usize, classes: usize) -> ClassCurves {
    let (a, b) = items(k, classes);
    let (pi, eta) = (vec![1.0; classes], vec![0.0; classes]);
    match classes {
        1 => ClassCurves::new(&pi, &eta, &a, &b),
        _ => ClassCurves::with_ability(&pi, &eta, &normal(), &a, &b, &vec![0.0; k]),
    }
    .expect("well-formed classes")
}

fn record(
    pool: &mut ContestedPool,
    tag: usize,
    k: usize,
    classes: usize,
) -> Result<(), RecordError> {
    let members: Vec<(Cid, usize)> = (0..k)
        .map(|j| (cid(format!("d3-{tag}-{j}").as_bytes()), j))
        .collect();
    pool.record(curves(k, classes), &members)
}

/// One draw of `n` from `fits` fits of `k` members and `classes` classes, at `tolerance`.
fn draw(name: &str, fits: usize, k: usize, classes: usize, n: usize, tolerance: f64) {
    let base = LIVE.load(Relaxed);
    let mut pool = ContestedPool::new();
    for f in 0..fits {
        record(&mut pool, f, k, classes).expect("within the pool's domain");
    }
    let held = LIVE.load(Relaxed) - base;
    let counts = pool.admission(n).expect("an admitted shape");
    let (before, _) = rss();
    let start = LIVE.load(Relaxed);
    PEAK.store(start, Relaxed);
    let clock = Instant::now();
    let outcome = pool.draw(n, tolerance, 0);
    let seconds = clock.elapsed().as_secs_f64();
    let extra = PEAK.load(Relaxed) - start;
    let (_, hwm) = rss();
    let drawn = match outcome {
        Ok(test) => format!("drew {}", test.len()),
        Err(DrawError::NoBalancedDraw(e)) => {
            format!("no balanced draw, {} feasible", e.feasible.len())
        }
        Err(DrawError::OutOfDomain(e)) => format!("out of domain {e:?}"),
    };
    println!(
        "{name}: n={n} {counts:?} {drawn}; seconds={seconds:.3}; draw requested peak={extra} B; \
         pool requested={held} B, estimate={:?} B; VmRSS before={before} KiB, VmHWM={hwm} KiB",
        pool.bytes()
    );
}

/// Default fits of 16 (four classes, 41 nodes) recorded until the pool refuses one, then draws.
fn pool() {
    let base = LIVE.load(Relaxed);
    let mut pool = ContestedPool::new();
    let clock = Instant::now();
    let (mut fits, mut slowest) = (0, 0.0_f64);
    let refusal = loop {
        let one = Instant::now();
        match record(&mut pool, fits, 16, 4) {
            Ok(()) => fits += 1,
            Err(e) => break e,
        }
        slowest = slowest.max(one.elapsed().as_secs_f64());
    };
    let seconds = clock.elapsed().as_secs_f64();
    let held = LIVE.load(Relaxed) - base;
    let (rss_now, hwm) = rss();
    println!(
        "pool: {fits} fits, then {refusal:?}; {seconds:.3} s, slowest record {slowest:.4} s; \
         requested={held} B, estimate={:?} B; VmRSS={rss_now} KiB, VmHWM={hwm} KiB",
        pool.bytes()
    );
    for n in [2, 3] {
        let start = LIVE.load(Relaxed);
        PEAK.store(start, Relaxed);
        let clock = Instant::now();
        let outcome = pool.draw(n, DTF_MAX, 0).map(|t| t.len());
        let seconds = clock.elapsed().as_secs_f64();
        let extra = PEAK.load(Relaxed) - start;
        let (_, hwm) = rss();
        println!(
            "pool draw: n={n} {:?} -> {outcome:?}; seconds={seconds:.3}; draw requested peak={extra} \
             B; VmHWM={hwm} KiB",
            pool.admission(n).map(|c| c.point_terms)
        );
    }
}

/// A contested fit of `classes` equal classes over an `ability`, as [`items`] lays them out.
fn fit_record(
    classes: usize,
    k: usize,
    ability: Ability,
    members: Vec<(Cid, u64)>,
) -> ResultRecord {
    let (a, b) = items(k, classes);
    ResultRecord::ContestedFit {
        pi: vec![1.0; classes],
        eta: vec![0.0; classes],
        a,
        b,
        c: vec![0.0; k],
        ability,
        members,
    }
}

/// A state whose pool holds as many default fits of 16 as [`pool`] records, in one epoch.
fn full_state() -> ResultsState {
    let mut pool = ContestedPool::new();
    let mut fits = 0;
    while record(&mut pool, fits, 16, 4).is_ok() {
        fits += 1;
    }
    drop(pool);
    let records = (0..fits)
        .map(|f| {
            let members = (0..16)
                .map(|j| (cid(format!("d3-{f}-{j}").as_bytes()), j))
                .collect();
            fit_record(4, 16, normal(), members)
        })
        .collect();
    let mut state = ResultsState::default();
    let epoch = EpochResults {
        epoch: 1,
        inputs_root: [0; 32],
        records,
    };
    state.apply(&epoch).expect("the pool's own fits");
    state
}

/// Three records refused before their curves or copies — 4,096 items in 16 classes over 256
/// points with one member or none, and 65,536 members of one item — on two states.
fn input() {
    let wide = || Ability {
        nodes: (0..256).map(f64::from).collect(),
        weights: vec![1.0; 256],
    };
    let crowd = (0..1u32 << 16)
        .map(|i| (cid(&i.to_le_bytes()), 0))
        .collect();
    let records = [
        (
            "wide, one member",
            fit_record(16, 4096, wide(), vec![(cid(b"wide"), 0)]),
        ),
        ("wide, no members", fit_record(16, 4096, wide(), Vec::new())),
        (
            "65,536 members, one item",
            fit_record(2, 1, normal(), crowd),
        ),
    ];
    for (name, mut state) in [
        ("empty", ResultsState::default()),
        ("full pool", full_state()),
    ] {
        let start = LIVE.load(Relaxed);
        PEAK.store(start, Relaxed);
        let copy = state.clone();
        let clone = PEAK.load(Relaxed) - start;
        drop(copy);
        for (label, record) in &records {
            let results = EpochResults {
                epoch: 2,
                inputs_root: [0; 32],
                records: vec![record.clone()],
            };
            let start = LIVE.load(Relaxed);
            PEAK.store(start, Relaxed);
            let clock = Instant::now();
            let outcome = state.apply(&results);
            let seconds = clock.elapsed().as_secs_f64();
            let extra = PEAK.load(Relaxed) - start;
            let (_, hwm) = rss();
            println!(
                "input: {name} state, {label}: {outcome:?}; seconds={seconds:.6}; apply requested \
                 peak={extra} B, the state's clone {clone} B, beyond it {} B; VmHWM={hwm} KiB",
                extra - clone
            );
        }
    }
}

fn main() {
    let shape = std::env::args().nth(1).unwrap_or_default();
    match shape.as_str() {
        "kept" => draw("kept", 1, 70, 1, 4, DTF_MAX),
        "entries" => draw("entries", 8, 16, 1, 16, DTF_MAX),
        "terms" => draw("terms", 1, 16, 4, 16, DTF_MAX),
        "cells-one" => draw("cells-one", 1, 2, 1, (1 << 19) - 1, DTF_MAX),
        "cells-many" => draw("cells-many", 1023, 1, 1, 1023, DTF_MAX),
        "scans" => draw("scans", 1, 16, 1, 1023, DTF_MAX),
        "combined" => draw("combined", 8, 16, 2, 127, DTF_MAX),
        "terms-all" => draw("terms-all", 1, 16, 4, 16, f64::INFINITY),
        "combined-all" => draw("combined-all", 8, 16, 2, 127, f64::INFINITY),
        "pool" => pool(),
        "input" => input(),
        _ => {
            eprintln!("shapes: kept entries terms cells-one cells-many scans combined terms-all");
            eprintln!("        combined-all pool input");
            std::process::exit(2);
        }
    }
}
