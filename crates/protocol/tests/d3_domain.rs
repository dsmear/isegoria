//! D3's resource domain (`docs/15` D3, `docs/02` §B.7): the draw's limits and the pool's, checked
//! before any costly allocation; refusals typed apart from an unbalanced pool, leaving no trace.
//! A counting allocator measures the bytes a call requests on its own thread.

use network::cid::{cid, Cid};
use proptest::prelude::*;
use protocol::contested::{
    draw_counts, ContestedPool, DrawCounts, DrawError, DrawLimit, DrawLimits, FitShape,
    NoBalancedDraw, OutOfDomain, PoolLimit, RecordError, DRAW_LIMITS, POOL_FIXED_BYTES,
    POOL_LIMITS,
};
use protocol::results::{EpochResults, ResultRecord, ResultsRejected, ResultsState};
use scoring::dtf::{ClassCurves, DTF_MAX};
use scoring::latent::Ability;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct Counting;

thread_local! {
    static LIVE: Cell<isize> = const { Cell::new(0) };
    static PEAK: Cell<isize> = const { Cell::new(0) };
}

fn moved(bytes: isize) {
    let _ = LIVE.try_with(|live| {
        live.set(live.get() + bytes);
        let _ = PEAK.try_with(|peak| peak.set(peak.get().max(live.get())));
    });
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            moved(layout.size() as isize);
        }
        p
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
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

/// `f`'s result and the most bytes it held at once beyond what was live when it started.
fn peak<T>(f: impl FnOnce() -> T) -> (T, isize) {
    let start = LIVE.with(Cell::get);
    PEAK.with(|p| p.set(start));
    let out = f();
    (out, PEAK.with(Cell::get) - start)
}

fn shape(members: usize, classes: usize, points: usize) -> FitShape {
    FitShape {
        members,
        classes,
        points,
        items: members,
    }
}

/// `k` items over `classes` equal classes on the fixed grid, leaning ±0.9 in turn.
fn curves(k: usize, classes: usize) -> ClassCurves {
    let shift = [-1.0, 1.0, 0.0, 0.0];
    let a = vec![vec![1.2; k]; classes];
    let b: Vec<Vec<f64>> = (0..classes)
        .map(|g| {
            (0..k)
                .map(|j| if j % 2 == 0 { 0.9 } else { -0.9 } * shift[g % 4])
                .collect()
        })
        .collect();
    ClassCurves::new(&vec![1.0; classes], &vec![0.0; classes], &a, &b).unwrap()
}

fn record(pool: &mut ContestedPool, tag: &str, k: usize, classes: usize) -> Vec<Cid> {
    let members: Vec<(Cid, usize)> = (0..k)
        .map(|j| (cid(format!("{tag}{j}").as_bytes()), j))
        .collect();
    pool.record(curves(k, classes), &members).unwrap();
    members.iter().map(|m| m.0).collect()
}

fn refused(limit: DrawLimit) -> impl Fn(Result<DrawCounts, OutOfDomain>) {
    move |r| {
        assert!(
            matches!(r, Err(OutOfDomain { limit: l, .. }) if l == limit),
            "{r:?}"
        )
    }
}

/// D3: each draw limit admits a request at its value and refuses one just past it.
#[test]
fn every_draw_limit_admits_its_frontier_and_refuses_past_it() {
    let one = [shape(2, 1, 41)];
    let at = draw_counts(one, (1 << 19) - 1, &DRAW_LIMITS).unwrap();
    assert_eq!(at.table_cells, 1 << 20);
    refused(DrawLimit::TableCells)(draw_counts(one, 1 << 19, &DRAW_LIMITS));

    let wide = vec![shape(4095, 1, 41); 256];
    assert_eq!(
        draw_counts(wide.clone(), 1, &DRAW_LIMITS).unwrap().visits,
        1 << 20
    );
    let mut past = wide;
    past[255] = shape(4096, 1, 41);
    refused(DrawLimit::Visits)(draw_counts(past, 1, &DRAW_LIMITS));

    let sixteen = [shape(16, 1, 41)];
    assert_eq!(
        draw_counts(sixteen, 1023, &DRAW_LIMITS)
            .unwrap()
            .table_scans,
        1 << 26
    );
    refused(DrawLimit::TableScans)(draw_counts(sixteen, 1024, &DRAW_LIMITS));

    let eight = vec![shape(16, 1, 41); 8];
    assert_eq!(
        draw_counts(eight.clone(), 16, &DRAW_LIMITS)
            .unwrap()
            .member_entries,
        1 << 22
    );
    let mut past = eight;
    past.push(shape(1, 1, 41));
    refused(DrawLimit::MemberEntries)(draw_counts(past, 16, &DRAW_LIMITS));

    let terms = vec![shape(16, 2, 128); 8];
    assert_eq!(
        draw_counts(terms.clone(), 16, &DRAW_LIMITS)
            .unwrap()
            .point_terms,
        1 << 29
    );
    let mut past = terms;
    past[0] = shape(16, 2, 129);
    refused(DrawLimit::PointTerms)(draw_counts(past, 16, &DRAW_LIMITS));
}

/// D3: the check's counts are `sim/d3_selection_counts.py`'s subsets and member terms, exactly.
#[test]
fn the_counts_are_the_selection_count_script_s() {
    let open = DrawLimits {
        visits: u64::MAX,
        member_entries: u64::MAX,
        point_terms: u64::MAX,
        table_cells: u64::MAX,
        table_scans: u64::MAX,
    };
    let fits = |ms: &[usize]| ms.iter().map(|&m| shape(m, 2, 82)).collect::<Vec<_>>();
    let cases: [(&[usize], usize, u128, u128); 12] = [
        (&[5, 3, 3, 1], 2, 32, 44),
        (&[5, 3, 3, 1], 6, 50, 105),
        (&[3, 3], 2, 14, 18),
        (&[1, 1], 2, 4, 2),
        (&[8], 8, 256, 1_024),
        (&[16], 4, 2_517, 9_216),
        (&[16], 16, 65_536, 524_288),
        (&[20], 5, 21_700, 100_720),
        (&[20], 10, 616_666, 5_242_880),
        (&[40], 5, 760_099, 3_686_840),
        (&[40], 10, 1_221_246_132, 11_710_014_720),
        (&[40], 20, 618_679_078_298, 10_995_116_277_760),
    ];
    for (ms, n, visits, entries) in cases {
        let counts = draw_counts(fits(ms), n, &open).unwrap();
        assert_eq!(
            (counts.visits, counts.member_entries),
            (visits, entries),
            "{ms:?}, n = {n}"
        );
        assert_eq!(counts.point_terms, 82 * entries);
    }
    let four = draw_counts([shape(16, 4, 164)], 16, &open).unwrap();
    assert_eq!(four.point_terms, 515_899_392);
}

/// D3: one class keeps every subset (DTF 0), so its visits bound the draw although `P = 0`.
#[test]
fn one_class_is_bounded_by_its_visits_not_its_terms() {
    let mut pool = ContestedPool::new();
    record(&mut pool, "one", 1500, 1);
    let counts = pool.admission(1).unwrap();
    assert_eq!((counts.visits, counts.point_terms), (1501, 0));
    let (result, bytes) = peak(|| pool.draw(2, DTF_MAX, 0));
    let Err(DrawError::OutOfDomain(o)) = result else {
        panic!("{result:?}")
    };
    let reached = Some(1 + 1500 + 1500 * 1499 / 2);
    assert_eq!(
        o,
        OutOfDomain {
            limit: DrawLimit::Visits,
            reached
        }
    );
    assert!(bytes < 4096, "{bytes} bytes");
    assert_eq!(pool.draw(1, f64::NAN, 1).map(|t| t.len()), Ok(1));
}

/// D3: an empty pool draws nothing at `n = 0`, has no balanced selection above it, and its table
/// still bounds `n`.
#[test]
fn an_empty_pool_and_n_zero() {
    let pool = ContestedPool::new();
    assert_eq!(pool.draw(0, DTF_MAX, 0), Ok(Vec::new()));
    let none = DrawError::NoBalancedDraw(NoBalancedDraw {
        requested: 1,
        feasible: vec![0],
    });
    assert_eq!(pool.draw(1, DTF_MAX, 0), Err(none));
    assert_eq!(pool.admission(0).unwrap().table_cells, 1);
    let (result, bytes) = peak(|| pool.draw(1 << 20, DTF_MAX, 0));
    assert!(matches!(
        result,
        Err(DrawError::OutOfDomain(OutOfDomain {
            limit: DrawLimit::TableCells,
            ..
        }))
    ));
    assert!(bytes < 4096, "{bytes} bytes");
    let mut one = ContestedPool::new();
    record(&mut one, "z", 3, 2);
    assert_eq!(one.draw(0, DTF_MAX, 5), Ok(Vec::new()));
    assert_eq!(one.admission(0).unwrap().visits, 1);
}

/// D3: an enormous `n` and counts past `u128` are refused, without enumerating or allocating.
#[test]
fn an_enormous_request_and_overflow_are_refused_without_allocating() {
    let mut pool = ContestedPool::new();
    record(&mut pool, "h", 4, 2);
    let (result, bytes) = peak(|| pool.draw(usize::MAX, DTF_MAX, 0));
    let Err(DrawError::OutOfDomain(o)) = result else {
        panic!("{result:?}")
    };
    assert_eq!(o.limit, DrawLimit::TableCells);
    assert_eq!(o.reached, Some(2 * (u128::from(u64::MAX) + 1)));
    assert!(bytes < 4096, "{bytes} bytes");

    let every = std::iter::repeat_n(shape(1, 1, 1), usize::MAX);
    let all = draw_counts(every, usize::MAX, &DRAW_LIMITS);
    assert_eq!(
        all,
        Err(OutOfDomain {
            limit: DrawLimit::TableCells,
            reached: None
        })
    );
    let huge = [shape(usize::MAX, usize::MAX, usize::MAX)];
    let r = draw_counts(huge, 0, &DRAW_LIMITS).unwrap();
    assert_eq!((r.visits, r.point_terms), (1, 0));
    let r = draw_counts(huge, 1, &DRAW_LIMITS);
    assert_eq!(
        r,
        Err(OutOfDomain {
            limit: DrawLimit::Visits,
            reached: Some(1 << 64)
        })
    );
    let terms = [shape(3, usize::MAX, usize::MAX)];
    let r = draw_counts(terms, 3, &DRAW_LIMITS);
    assert_eq!(
        r,
        Err(OutOfDomain {
            limit: DrawLimit::PointTerms,
            reached: None
        })
    );
    let (r, bytes) = peak(|| draw_counts([shape(64, 1, 41)], 64, &DRAW_LIMITS));
    assert_eq!(r.unwrap_err().limit, DrawLimit::Visits);
    assert_eq!(bytes, 0);
}

/// D3: many small fits draw as before up to the table's frontier, and the pool holds at most
/// `POOL_LIMITS.fits` of them; the refused record changes nothing.
#[test]
fn many_small_fits() {
    let mut pool = ContestedPool::new();
    for f in 0..300 {
        record(&mut pool, &format!("s{f}-"), 1, 2);
    }
    let drawn = pool.draw(2, 1.0, 3).unwrap();
    assert_eq!(drawn.len(), 2);
    assert_eq!(pool.admission(3482).unwrap().table_cells, 301 * 3483);
    let past = pool.draw(3483, 1.0, 3);
    let Err(DrawError::OutOfDomain(o)) = past else {
        panic!("{past:?}")
    };
    assert_eq!(o.limit, DrawLimit::TableCells);
    for f in 300..POOL_LIMITS.fits {
        record(&mut pool, &format!("s{f}-"), 1, 2);
    }
    let before = pool.clone();
    let extra = (cid(b"one too many"), 0);
    let refusal = pool.record(curves(1, 2), &[extra]);
    assert_eq!(refusal, Err(RecordError::OutOfDomain(PoolLimit::Fits)));
    assert_eq!(pool, before);
    let first = cid(b"s0-0");
    pool.record(curves(1, 2), &[(first, 0)]).unwrap();
    assert_eq!(
        pool.len(),
        before.len(),
        "a re-measurement within the limit moves the fact"
    );
}

/// D3: the pool's frontiers per fit and in bytes, each admitted at its value and refused past it.
#[test]
fn the_pool_limits_admit_their_frontier_and_refuse_past_it() {
    let pool = ContestedPool::new();
    let fit = |members, classes, points, items| FitShape {
        members,
        classes,
        points,
        items,
    };
    let one = [cid(b"m")];
    assert_eq!(pool.admits(&fit(1, 16, 41, 8), &one), Ok(()));
    assert_eq!(
        pool.admits(&fit(1, 17, 41, 8), &one),
        Err(PoolLimit::Classes)
    );
    assert_eq!(pool.admits(&fit(1, 2, 4096, 8), &one), Ok(()));
    assert_eq!(
        pool.admits(&fit(1, 2, 4097, 8), &one),
        Err(PoolLimit::Points)
    );
    assert_eq!(pool.admits(&fit(1, 2, 82, 4096), &one), Ok(()));
    assert_eq!(
        pool.admits(&fit(1, 2, 82, 4097), &one),
        Err(PoolLimit::Items)
    );
    let room = u128::from(POOL_LIMITS.bytes - POOL_FIXED_BYTES);
    let largest = fit(1, 16, 4096, 253);
    assert!(largest.bytes().unwrap() <= room);
    assert!(fit(1, 16, 4096, 254).bytes().unwrap() > room);
    let (r, bytes) = peak(|| pool.admits(&largest, &one));
    assert_eq!(r, Ok(()));
    assert!(bytes < 1024, "{bytes} bytes");
    assert_eq!(
        pool.admits(&fit(1, 16, 4096, 254), &one),
        Err(PoolLimit::Bytes)
    );
    assert_eq!(pool.bytes(), Some(u128::from(POOL_FIXED_BYTES)));
}

/// D3: a record whose curves would be large is refused before they exist, and leaves no trace.
#[test]
fn inputs_that_would_expand_are_refused_before_their_curves() {
    let mut state = ResultsState::default();
    let (classes, items, nodes) = (16, 4096, 256);
    let wide = ResultRecord::ContestedFit {
        pi: vec![1.0; classes],
        eta: vec![0.0; classes],
        a: vec![vec![1.0; items]; classes],
        b: vec![vec![0.0; items]; classes],
        c: vec![0.0; items],
        ability: Ability {
            nodes: (0..nodes).map(|q| q as f64).collect(),
            weights: vec![1.0; nodes],
        },
        members: vec![(cid(b"w"), 0)],
    };
    let results = EpochResults {
        epoch: 1,
        inputs_root: [0; 32],
        records: vec![wide],
    };
    let before = state.clone();
    let (r, bytes) = peak(|| state.apply(&results));
    assert_eq!(
        r,
        Err(ResultsRejected::ContestedOutOfDomain {
            record: 0,
            limit: PoolLimit::Bytes
        })
    );
    assert!(bytes < 1 << 16, "{bytes} bytes against 2 GiB of curves");
    assert_eq!(state, before);
}

/// D3: a refused record leaves every fact in its fit, even those it would have moved.
#[test]
fn a_refused_record_moves_nothing() {
    let mut pool = ContestedPool::new();
    let ids = record(&mut pool, "r", 4, 2);
    let before = pool.clone();
    let wide = curves(2, 17);
    let refusal = pool.record(wide, &[(ids[0], 0), (ids[1], 1)]);
    assert_eq!(refusal, Err(RecordError::OutOfDomain(PoolLimit::Classes)));
    assert_eq!(pool, before);
    assert_eq!(pool.draw(4, 10.0, 0).map(|t| t.len()), Ok(4));
}

/// D3: out of domain is no verdict on balance: a one-class pool balances any size, yet is refused.
#[test]
fn out_of_domain_is_not_a_missing_balanced_selection() {
    let mut mirror = ContestedPool::new();
    let fit = |lean: f64| {
        let a = vec![vec![1.25]; 2];
        let b = vec![vec![-lean], vec![lean]];
        ClassCurves::new(&[0.5, 0.5], &[0.0, 0.0], &a, &b).unwrap()
    };
    mirror.record(fit(0.9), &[(cid(b"x"), 0)]).unwrap();
    mirror.record(fit(-0.9), &[(cid(b"y"), 0)]).unwrap();
    assert!(matches!(
        mirror.draw(2, DTF_MAX, 0),
        Err(DrawError::NoBalancedDraw(_))
    ));
    let mut flat = ContestedPool::new();
    record(&mut flat, "f", 21, 1);
    assert!(matches!(
        flat.draw(21, DTF_MAX, 0),
        Err(DrawError::OutOfDomain(_))
    ));
    assert_eq!(flat.draw(5, DTF_MAX, 0).map(|t| t.len()), Ok(5));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// D3: admission is a function of the shapes and `n`, whatever the seed and the tolerance.
    #[test]
    fn admission_ignores_the_seed_and_the_tolerance(
        sizes in prop::collection::vec((1usize..6, 1usize..4), 0..5),
        wide in any::<bool>(),
        pick in 0usize..12,
        seeds in prop::collection::vec(any::<u64>(), 3),
        tolerance in prop_oneof![
            Just(f64::NAN), Just(f64::INFINITY), Just(-1.0), 0.0f64..1.0, 1.0f64..1e6
        ],
    ) {
        let mut pool = ContestedPool::new();
        for (f, &(k, classes)) in sizes.iter().enumerate() {
            record(&mut pool, &format!("p{f}-"), k, classes);
        }
        if wide {
            record(&mut pool, "wide", 1500, 1);
        }
        let n = [0, 1, 2, 3, 5, 8, 21, 1 << 20, 1 << 40, usize::MAX - 1, usize::MAX, 13][pick];
        let admission = pool.admission(n);
        for seed in seeds {
            let (result, bytes) = peak(|| pool.draw(n, tolerance, seed));
            match (&admission, result) {
                (Err(o), Err(DrawError::OutOfDomain(r))) => {
                    prop_assert_eq!(*o, r);
                    prop_assert!(bytes < 4096);
                }
                (Ok(_), Err(DrawError::OutOfDomain(r))) => prop_assert!(false, "{:?}", r),
                (Err(o), other) => prop_assert!(false, "{:?} then {:?}", o, other),
                (Ok(_), _) => {}
            }
        }
    }
}
