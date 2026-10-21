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
    let one = [(cid(b"m"), 0usize)];
    let out = |limit| -> Result<(), RecordError> { Err(RecordError::OutOfDomain(limit)) };
    assert_eq!(pool.admits(16, 41, 8, &one), Ok(()));
    assert_eq!(pool.admits(17, 41, 8, &one), out(PoolLimit::Classes));
    assert_eq!(pool.admits(2, 4096, 8, &one), Ok(()));
    assert_eq!(pool.admits(2, 4097, 8, &one), out(PoolLimit::Points));
    assert_eq!(pool.admits(2, 82, 4096, &one), Ok(()));
    assert_eq!(pool.admits(2, 82, 4097, &one), out(PoolLimit::Items));
    let room = u128::from(POOL_LIMITS.bytes - POOL_FIXED_BYTES);
    let fit = |items| FitShape {
        members: 1,
        classes: 16,
        points: 4096,
        items,
    };
    assert!(fit(253).bytes().unwrap() <= room);
    assert!(fit(254).bytes().unwrap() > room);
    assert_eq!(fit(253).buildable(), Ok(()));
    assert_eq!(fit(254).buildable(), Err(PoolLimit::Bytes));
    let (r, bytes) = peak(|| pool.admits(16, 4096, 253, &one));
    assert_eq!(r, Ok(()));
    assert!(bytes < 1024, "{bytes} bytes");
    assert_eq!(pool.admits(16, 4096, 254, &one), out(PoolLimit::Bytes));
    assert_eq!(pool.bytes(), Some(u128::from(POOL_FIXED_BYTES)));
}

/// A contested fit of `classes` equal classes, `items` items and `nodes` ability points each of
/// weight `weight`: its curves would hold `items · classes² · nodes` values.
fn fit_record(
    classes: usize,
    items: usize,
    nodes: usize,
    weight: f64,
    members: Vec<(Cid, u64)>,
) -> ResultRecord {
    ResultRecord::ContestedFit {
        pi: vec![1.0; classes],
        eta: vec![0.0; classes],
        a: vec![vec![1.0; items]; classes],
        b: vec![vec![0.0; items]; classes],
        c: vec![0.0; items],
        ability: Ability {
            nodes: (0..nodes).map(|q| q as f64 / nodes as f64).collect(),
            weights: vec![weight; nodes],
        },
        members,
    }
}

fn epoch(epoch: u64, records: Vec<ResultRecord>) -> EpochResults {
    EpochResults {
        epoch,
        inputs_root: [0; 32],
        records,
    }
}

/// `count` distinct facts, all at index 0.
fn crowd(count: u32) -> Vec<(Cid, u64)> {
    (0..count).map(|i| (cid(&i.to_le_bytes()), 0)).collect()
}

/// An empty state, and one whose pool holds a fit of four facts.
fn states() -> [ResultsState; 2] {
    let mut held = ResultsState::default();
    let four = (0..4)
        .map(|j| (cid(format!("held{j}").as_bytes()), j))
        .collect();
    held.apply(&epoch(1, vec![fit_record(2, 4, 3, 1.0, four)]))
        .unwrap();
    [ResultsState::default(), held]
}

/// D3: a fit with no members is bounded by its curves, as one with members is.
#[test]
fn admits_bounds_a_fit_without_members_by_its_curves() {
    let pool = ContestedPool::new();
    let wide = FitShape {
        members: 0,
        classes: 16,
        points: 4096,
        items: 4096,
    };
    assert_eq!(wide.buildable(), Err(PoolLimit::Bytes));
    let none: [(Cid, usize); 0] = [];
    let (r, bytes) = peak(|| pool.admits(16, 4096, 4096, &none));
    assert_eq!(r, Err(RecordError::OutOfDomain(PoolLimit::Bytes)));
    assert_eq!(bytes, 0);
}

/// D3: a record with no members is refused out of domain before its curves are built.
#[test]
fn the_node_bounds_a_record_without_members_before_its_curves() {
    for mut state in states() {
        let before = state.clone();
        let (_, clone) = peak(|| state.clone());
        let massless = fit_record(16, 4096, 256, 0.0, Vec::new());
        let (r, bytes) = peak(|| state.apply(&epoch(2, vec![massless])));
        assert_eq!(
            r,
            Err(ResultsRejected::ContestedOutOfDomain {
                record: 0,
                limit: PoolLimit::Bytes
            })
        );
        assert!(bytes - clone < 4096, "{bytes} bytes, the clone {clone}");
        assert_eq!(state, before);
    }
}

/// D3: a 2 GiB record, one member or none, refused on any state with nothing past the clone.
#[test]
fn wide_records_are_refused_before_their_curves_on_any_state() {
    for mut state in states() {
        let before = state.clone();
        let (_, clone) = peak(|| state.clone());
        for members in [vec![(cid(b"w"), 0)], Vec::new()] {
            let wide = epoch(2, vec![fit_record(16, 4096, 256, 1.0, members)]);
            let (r, bytes) = peak(|| state.apply(&wide));
            assert_eq!(
                r,
                Err(ResultsRejected::ContestedOutOfDomain {
                    record: 0,
                    limit: PoolLimit::Bytes
                })
            );
            assert_eq!(bytes, clone);
            assert_eq!(state, before);
        }
    }
}

/// D3: a record with no members passes the same checks as any other, then records nothing.
#[test]
fn records_without_members_are_checked_and_record_nothing() {
    for state in states() {
        let mut valid = state.clone();
        let empty = fit_record(2, 4, 3, 1.0, Vec::new());
        valid.apply(&epoch(2, vec![empty])).unwrap();
        assert_eq!(valid.contested(), state.contested());
        assert_eq!(valid.epoch_root(2), Some([0; 32]));
        let refusals = [
            (
                fit_record(2, 4, 3, 0.0, Vec::new()),
                ResultsRejected::BadClasses { record: 0 },
            ),
            (
                fit_record(17, 1, 3, 1.0, Vec::new()),
                ResultsRejected::ContestedOutOfDomain {
                    record: 0,
                    limit: PoolLimit::Classes,
                },
            ),
        ];
        for (record, why) in refusals {
            let mut refused = state.clone();
            assert_eq!(refused.apply(&epoch(2, vec![record])), Err(why));
            assert_eq!(refused, state);
        }
    }
    let mut pool = ContestedPool::new();
    record(&mut pool, "e", 3, 2);
    let before = pool.clone();
    assert_eq!(pool.record(curves(2, 2), &[]), Ok(()));
    assert_eq!(pool, before);
    let wide = pool.record(curves(1, 17), &[]);
    assert_eq!(wide, Err(RecordError::OutOfDomain(PoolLimit::Classes)));
    assert_eq!(pool, before);
}

/// D3: repeats and indices past the fit refused alike by `admits`, `record` and the node, uncopied.
#[test]
fn malformed_members_are_refused_before_their_copies() {
    let (x, y) = (cid(b"x"), cid(b"y"));
    let cases: [(Vec<(Cid, u64)>, RecordError); 5] = [
        (vec![(x, 0), (x, 1)], RecordError::DuplicateMember),
        (vec![(x, 0), (y, 0)], RecordError::DuplicateMember),
        (
            vec![(x, 0), (y, 1), (cid(b"z"), 0)],
            RecordError::DuplicateMember,
        ),
        (
            vec![(x, 2)],
            RecordError::IndexOutOfRange { index: 2, items: 2 },
        ),
        (
            vec![(x, 0), (y, u64::MAX)],
            RecordError::IndexOutOfRange {
                index: usize::MAX,
                items: 2,
            },
        ),
    ];
    let mut pool = ContestedPool::new();
    record(&mut pool, "m", 3, 2);
    let held = pool.clone();
    for (members, why) in cases {
        assert_eq!(peak(|| pool.admits(2, 41, 2, &members)), (Err(why), 0));
        let indexed: Vec<(Cid, usize)> = members
            .iter()
            .map(|&(c, j)| (c, usize::try_from(j).unwrap()))
            .collect();
        let two = curves(2, 2);
        let (r, bytes) = peak(|| pool.record(two, &indexed));
        assert_eq!((r, bytes), (Err(why), 0));
        assert_eq!(pool, held);
        for mut state in states() {
            let before = state.clone();
            let (_, clone) = peak(|| state.clone());
            let record = epoch(2, vec![fit_record(2, 2, 3, 1.0, members.clone())]);
            let (r, bytes) = peak(|| state.apply(&record));
            assert_eq!(r, Err(ResultsRejected::BadMembers { record: 0 }));
            assert_eq!(bytes, clone);
            assert_eq!(state, before);
        }
    }
    let many: Vec<(Cid, usize)> = crowd(1 << 16).iter().map(|&(c, _)| (c, 0)).collect();
    let two = curves(2, 2);
    let (r, bytes) = peak(|| pool.record(two, &many));
    assert_eq!((r, bytes), (Err(RecordError::DuplicateMember), 0));
    assert_eq!(pool, held);
}

/// D3: re-measured facts are counted out of their fits; a refused re-measurement moves nothing.
#[test]
fn replacements_count_the_facts_they_move_out() {
    let mut pool = ContestedPool::new();
    let held = record(&mut pool, "r", 16, 4);
    let others = record(&mut pool, "s", 4, 2);
    let room = u128::from(POOL_LIMITS.bytes - POOL_FIXED_BYTES);
    let bytes = |members, classes, points, items| {
        let fit = FitShape {
            members,
            classes,
            points,
            items,
        };
        fit.bytes().unwrap()
    };
    let (rest, left) = (bytes(4, 2, 41, 4), bytes(1, 4, 41, 16));
    assert!(bytes(16, 16, 4091, 254) + rest <= room);
    assert!(bytes(15, 16, 4091, 254) + rest + left > room);
    let all: Vec<(Cid, usize)> = held.iter().enumerate().map(|(j, &c)| (c, j)).collect();
    assert_eq!(pool.admits(16, 4091, 254, &all), Ok(()));
    let short = pool.admits(16, 4091, 254, &all[..15]);
    assert_eq!(short, Err(RecordError::OutOfDomain(PoolLimit::Bytes)));

    let fits = |pool: &ContestedPool| pool.admission(0).unwrap().table_cells - 1;
    assert_eq!(fits(&pool), 2);
    let size = pool.len();
    pool.record(curves(2, 2), &[(held[0], 0), (others[0], 1)])
        .unwrap();
    assert_eq!((fits(&pool), pool.len()), (3, size));
    let before = pool.clone();
    let refusals = [
        (
            curves(2, 2),
            vec![(held[1], 0), (held[1], 1)],
            RecordError::DuplicateMember,
        ),
        (
            curves(2, 2),
            vec![(held[1], 0), (others[1], 2)],
            RecordError::IndexOutOfRange { index: 2, items: 2 },
        ),
        (
            curves(1, 17),
            vec![(held[1], 0)],
            RecordError::OutOfDomain(PoolLimit::Classes),
        ),
    ];
    for (fit, members, why) in refusals {
        assert_eq!(pool.record(fit, &members), Err(why));
        assert_eq!(pool, before);
    }

    let [_, mut state] = states();
    let fact = |j: u8| cid(format!("held{j}").as_bytes());
    let moved = fit_record(2, 2, 3, 1.0, vec![(fact(0), 0), (cid(b"new"), 1)]);
    state.apply(&epoch(2, vec![moved])).unwrap();
    assert_eq!((fits(state.contested()), state.contested().len()), (2, 5));
    let before = state.clone();
    let out = |limit| ResultsRejected::ContestedOutOfDomain { record: 0, limit };
    let refusals = [
        (
            fit_record(2, 2, 3, 1.0, vec![(fact(1), 0), (fact(1), 1)]),
            ResultsRejected::BadMembers { record: 0 },
        ),
        (
            fit_record(17, 1, 3, 1.0, vec![(fact(1), 0)]),
            out(PoolLimit::Classes),
        ),
        (
            fit_record(16, 4096, 256, 1.0, vec![(fact(1), 0)]),
            out(PoolLimit::Bytes),
        ),
    ];
    for (record, why) in refusals {
        assert_eq!(state.apply(&epoch(3, vec![record])), Err(why));
        assert_eq!(state, before);
    }
}

/// D3: more members than items are refused before they are copied.
#[test]
fn admits_refuses_more_members_than_items_before_copying_them() {
    let pool = ContestedPool::new();
    let members = crowd(1 << 16);
    let (r, bytes) = peak(|| pool.admits(2, 82, 1, &members));
    assert_eq!(r, Err(RecordError::DuplicateMember));
    assert_eq!(bytes, 0);
}

/// D3: the node refuses more members than items before copying them, beyond its state's clone.
#[test]
fn the_node_refuses_more_members_than_items_before_copying_them() {
    for mut state in states() {
        let before = state.clone();
        let (_, clone) = peak(|| state.clone());
        let crowded = fit_record(2, 1, 3, 1.0, crowd(1 << 16));
        let (r, bytes) = peak(|| state.apply(&epoch(2, vec![crowded])));
        assert_eq!(r, Err(ResultsRejected::BadMembers { record: 0 }));
        assert!(bytes - clone < 4096, "{bytes} bytes, the clone {clone}");
        assert_eq!(state, before);
    }
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
