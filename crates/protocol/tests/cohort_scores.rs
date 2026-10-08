//! B-b's scorer (`docs/17` §14.9): `π_j` below 1, selection apart from `I` and pending, counts and
//! denominator, the weighted interval, refused inputs, the supported baseline compositions, a
//! finite enumeration with shared draws against the reference expectation, and `f64`'s range.

use identity::nym::Nym;
use protocol::cohort_scores::{
    score, Case, Cause, Cohort, Counts, Extra, InputError, Item, NoTerm, OutOfRange, Outcome,
    Panelist, Report, Selection, Undefined, Value,
};

const U: u8 = 1;

fn nym(n: u8) -> Nym {
    Nym([n; 32])
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

fn q(num: i64, den: i64) -> f64 {
    num as f64 / den as f64
}

fn seat(n: u8, prob: f64, weight: f64) -> Panelist {
    Panelist {
        nym: nym(n),
        report: Report::Revealed(prob),
        weight,
    }
}

/// A frozen item: `u` reporting `p` at weight 1, then `r` (nym 2) reporting ½ at weight 1.
fn pair(p: f64, selection: Selection) -> Item {
    Item {
        first: vec![seat(U, p, 1.0), seat(2, 0.5, 1.0)],
        extra: Vec::new(),
        frozen: true,
        selection,
    }
}

fn selected(inclusion: f64, outcome: Option<Outcome>) -> Selection {
    Selection::Selected { inclusion, outcome }
}

fn cohort(items: &[usize]) -> Cohort {
    Cohort {
        nym: nym(U),
        items: items.to_vec(),
    }
}

fn one(item: Item) -> Result<protocol::cohort_scores::Scores, InputError> {
    score(&[item], &[cohort(&[0])])
}

/// A1 B-b, `17` §8.1 and D3: `g/π` below 1, positive and negative, beyond `[−1, 1]` at ¼.
#[test]
fn a_verdict_s_term_is_divided_by_its_inclusion_probability() {
    for (pi, a, r) in [
        (1.0, q(3, 16), q(-5, 16)),
        (0.5, q(3, 8), q(-5, 8)),
        (0.25, 0.75, -1.25),
    ] {
        for (outcome, want) in [(Outcome::A, a), (Outcome::R, r)] {
            let s = one(pair(0.75, selected(pi, Some(outcome)))).unwrap();
            let m = &s.cohorts[0].members[0];
            assert_eq!((m.baseline, m.inclusion), (Some(0.5), Some(pi)));
            assert!(close(m.term.unwrap().unwrap(), want), "{pi}, {outcome:?}");
            assert!(matches!(s.cohorts[0].value, Value::Final(x) if close(x, want)));
        }
    }
}

/// A1 B-b, `17` §8.2, §8.4: a non-selection adds 0 out of `O`, `I` 0 in `O`, a pending item none.
#[test]
fn a_non_selection_is_neither_an_inconclusive_outcome_nor_a_pending_one() {
    let items = [
        pair(0.75, Selection::NotSelected { inclusion: 0.5 }),
        pair(0.75, selected(0.5, Some(Outcome::I))),
        pair(0.75, selected(0.5, None)),
    ];
    let s = score(&items, &[cohort(&[0]), cohort(&[0, 1]), cohort(&[0, 1, 2])]).unwrap();
    let mine: Vec<_> = s.assignments.iter().filter(|a| a.nym == nym(U)).collect();
    let got: Vec<_> = mine.iter().map(|a| (a.case, a.term)).collect();
    assert_eq!(
        got,
        vec![
            (Case::NotSelected, Ok(Some(0.0))),
            (Case::Inconclusive, Ok(Some(0.0))),
            (Case::Pending, Ok(None)),
        ]
    );
    assert_eq!(s.counts(nym(U)), Counts { n: 3, o: 1, v: 0 });
    let k = |i: usize| {
        (
            s.cohorts[i].size,
            s.cohorts[i].o,
            s.cohorts[i].value.clone(),
        )
    };
    assert_eq!(k(0), (1, 0, Value::Final(0.0)));
    assert_eq!(k(1), (2, 1, Value::Final(0.0)));
    let bound = Value::Bound {
        sum: Ok((-2.0, 2.0)),
        mean: (q(-2, 3), q(2, 3)),
    };
    assert_eq!(k(2), (3, 1, bound));
}

/// A1 B-b, `17` §8.4, §8.6 (1): `N_u` counts every assignment; a prefixed `K` divides by `|K|`.
#[test]
fn the_counts_cover_every_assignment_and_a_cohort_divides_by_its_size() {
    let unfrozen = |report| Item {
        first: vec![
            Panelist {
                nym: nym(U),
                report,
                weight: 1.0,
            },
            seat(2, 0.5, 1.0),
        ],
        extra: Vec::new(),
        frozen: false,
        selection: selected(1.0, None),
    };
    let items = [
        pair(0.75, selected(1.0, Some(Outcome::A))),
        Item {
            first: vec![seat(2, 0.75, 1.0), seat(3, 0.25, 3.0)],
            extra: vec![Extra {
                nym: nym(U),
                report: Report::Revealed(0.25),
            }],
            frozen: true,
            selection: selected(1.0, Some(Outcome::R)),
        },
        unfrozen(Report::Missing),
        unfrozen(Report::Revealed(0.5)),
        pair(0.5, selected(1.0, Some(Outcome::A))),
    ];
    let mut items = items.to_vec();
    items[4].first[0].nym = nym(4);
    let s = score(&items, &[cohort(&[0, 1]), cohort(&[0, 2, 3])]).unwrap();
    assert_eq!(s.counts(nym(U)), Counts { n: 4, o: 2, v: 2 });
    assert_eq!(s.counts(nym(9)), Counts::default());
    let k = &s.cohorts[0];
    assert_eq!((k.size, k.o, k.v), (2, 2, 2));
    assert!(close(k.members[1].baseline.unwrap(), q(3, 8)));
    assert!(matches!(k.value, Value::Final(x) if close(x, q(17, 128))));
    let k = &s.cohorts[1];
    assert_eq!((k.size, k.o, k.v), (3, 1, 1));
    assert!(close(k.known.unwrap(), q(3, 16)));
    let causes = vec![(2, Cause::MissingReport), (3, Cause::Unfrozen)];
    assert_eq!(k.value, Value::Unavailable(causes));
}

/// A1 B-b, `17` §8.3 (D3): pending members widen the sum by `Σ 1/π_j`, not by their number.
#[test]
fn the_pending_interval_is_weighted_by_inclusion() {
    let items = [
        pair(0.75, selected(1.0, Some(Outcome::A))),
        pair(0.75, selected(0.5, None)),
        pair(0.25, selected(0.25, None)),
    ];
    let s = score(&items, &[cohort(&[0, 1, 2])]).unwrap();
    let k = &s.cohorts[0];
    assert!(close(k.known.unwrap(), q(3, 16)));
    let (low, high) = (q(3, 16) - 6.0, q(3, 16) + 6.0);
    let want = Value::Bound {
        sum: Ok((low, high)),
        mean: (low / 3.0, high / 3.0),
    };
    assert_eq!(k.value, want);
    let Value::Bound { sum: Ok(sum), .. } = k.value else {
        unreachable!()
    };
    assert!(
        !close(sum.1 - k.known.unwrap(), 2.0),
        "the count of pending members"
    );
}

/// A1 B-b, `17` §14.9: probabilities, weights and inclusions outside their domain are refused.
#[test]
fn numbers_outside_their_domain_are_refused() {
    let at = |p, w, pi| {
        let mut i = pair(0.5, selected(pi, None));
        i.first[0] = seat(U, p, w);
        one(i)
    };
    for p in [f64::NAN, -0.1, 1.5, f64::INFINITY] {
        let e = InputError::Probability {
            item: 0,
            nym: nym(U),
        };
        assert_eq!(at(p, 1.0, 1.0), Err(e), "{p}");
    }
    for w in [-1.0, f64::NAN, f64::INFINITY] {
        let e = InputError::Weight {
            item: 0,
            nym: nym(U),
        };
        assert_eq!(at(0.5, w, 1.0), Err(e), "{w}");
    }
    for pi in [0.0, -0.5, 1.5, f64::NAN, f64::INFINITY] {
        assert_eq!(
            at(0.5, 1.0, pi),
            Err(InputError::Inclusion { item: 0 }),
            "{pi}"
        );
        let not = one(pair(0.5, Selection::NotSelected { inclusion: pi }));
        assert_eq!(not, Err(InputError::Inclusion { item: 0 }), "{pi}");
    }
    for (p, w, pi) in [(0.0, 0.0, 1.0), (1.0, 2.5, 1e-9)] {
        assert!(at(p, w, pi).is_ok(), "{p}, {w}, {pi}");
    }
    let mut extra = pair(0.5, selected(1.0, None));
    extra.extra.push(Extra {
        nym: nym(3),
        report: Report::Revealed(-0.5),
    });
    let e = InputError::Probability {
        item: 0,
        nym: nym(3),
    };
    assert_eq!(one(extra), Err(e));
}

/// A1 B-b, `17` §14.9: the incoherent combinations the contract names are refused, not repaired.
#[test]
fn incoherent_inputs_are_refused() {
    let first = |n: u8, report| Panelist {
        nym: nym(n),
        report,
        weight: 1.0,
    };
    let certain = pair(0.5, Selection::NotSelected { inclusion: 1.0 });
    assert_eq!(
        one(certain),
        Err(InputError::NotSelectedAtCertainty { item: 0 })
    );
    let mut early = pair(0.5, selected(1.0, Some(Outcome::A)));
    early.frozen = false;
    assert_eq!(one(early), Err(InputError::OutcomeBeforeFreeze { item: 0 }));
    let mut open = pair(0.5, selected(1.0, None));
    open.first[1] = first(2, Report::Open);
    let e = InputError::OpenAtFreeze {
        item: 0,
        nym: nym(2),
    };
    assert_eq!(one(open.clone()), Err(e));
    open.frozen = false;
    assert_eq!(one(open).unwrap().assignments[1].case, Case::Unfrozen);
    let mut missing = pair(0.5, selected(1.0, None));
    missing.first[1] = first(2, Report::Missing);
    let e = InputError::MissingFirstAtFreeze {
        item: 0,
        nym: nym(2),
    };
    assert_eq!(one(missing), Err(e));
    let mut late = pair(0.5, selected(1.0, None));
    late.extra.push(Extra {
        nym: nym(3),
        report: Report::Open,
    });
    let e = InputError::OpenAtFreeze {
        item: 0,
        nym: nym(3),
    };
    assert_eq!(one(late), Err(e));
    let mut twice = pair(0.5, selected(1.0, None));
    twice.first.push(seat(2, 0.25, 1.0));
    let e = InputError::Reassigned {
        item: 0,
        nym: nym(2),
    };
    assert_eq!(one(twice), Err(e));
    let mut both = pair(0.5, selected(1.0, None));
    both.extra.push(Extra {
        nym: nym(U),
        report: Report::Revealed(0.5),
    });
    let e = InputError::Reassigned {
        item: 0,
        nym: nym(U),
    };
    assert_eq!(one(both), Err(e));
    let items = [pair(0.5, selected(1.0, None)), {
        let mut other = pair(0.5, selected(1.0, None));
        other.first[0].nym = nym(4);
        other
    }];
    for (k, e) in [
        (cohort(&[]), InputError::EmptyCohort { cohort: 1 }),
        (cohort(&[2]), InputError::UnknownItem { cohort: 1, item: 2 }),
        (cohort(&[1]), InputError::Unassigned { cohort: 1, item: 1 }),
        (
            cohort(&[0, 0]),
            InputError::RepeatedMember { cohort: 1, item: 0 },
        ),
    ] {
        assert_eq!(score(&items, &[cohort(&[0]), k]), Err(e));
    }
    let mut bad = items.to_vec();
    bad[1].first[1].weight = -1.0;
    let e = InputError::Weight {
        item: 1,
        nym: nym(2),
    };
    assert_eq!(score(&bad, &[cohort(&[1])]), Err(e), "items before cohorts");
}

fn panel() -> Vec<Panelist> {
    [(0.8, 2.0), (0.3, 0.5), (0.6, 1.5), (0.1, 0.0)]
        .iter()
        .enumerate()
        .map(|(k, &(p, w))| seat(10 + k as u8, p, w))
        .collect()
}

/// A1 C5, `16` §4.6: first panelists against the others, extra reviewers against the first panel.
#[test]
fn the_supported_compositions_are_panel_scores() {
    let extras = |ps: &[f64]| -> Vec<Extra> {
        let of = ps.iter().enumerate().map(|(k, &p)| Extra {
            nym: nym(20 + k as u8),
            report: Report::Revealed(p),
        });
        of.collect()
    };
    let item = |extra| Item {
        first: panel(),
        extra,
        frozen: true,
        selection: selected(1.0, Some(Outcome::A)),
    };
    let first = [0.525, 2.5 / 3.5, 0.7, 0.6625];
    let alone = score(&[item(Vec::new())], &[]).unwrap();
    for ps in [vec![0.2], vec![0.9, 0.05], vec![0.0, 1.0, 0.5]] {
        let s = score(&[item(extras(&ps))], &[]).unwrap();
        assert_eq!(s.assignments[..4], alone.assignments[..4]);
        for (k, a) in s.assignments.iter().enumerate() {
            let (p, b) = match k {
                0..=3 => ([0.8, 0.3, 0.6, 0.1][k], first[k]),
                _ => (ps[k - 4], 0.6625),
            };
            assert!(close(a.baseline.unwrap(), b), "{k}");
            let by_hand = (b - 1.0f64).powi(2) - (p - 1.0f64).powi(2);
            assert!(close(a.term.unwrap().unwrap(), by_hand), "{k}");
        }
    }
    let mut absent = item(extras(&[0.4]));
    absent.extra[0].report = Report::Missing;
    let s = score(&[absent], &[]).unwrap();
    assert_eq!(s.assignments[..4], alone.assignments[..4]);
    assert_eq!(
        (s.assignments[4].case, s.assignments[4].term),
        (Case::MissingReport, Ok(None))
    );
    let mut unfrozen = item(extras(&[0.4]));
    (unfrozen.frozen, unfrozen.selection) = (false, selected(1.0, None));
    let s = score(&[unfrozen], &[]).unwrap();
    assert!(s
        .assignments
        .iter()
        .all(|a| a.baseline.is_none() && a.case == Case::Unfrozen));
}

/// A1 `17` §14.4: no weight left is no baseline; a verdict's term undefined, `I`'s still 0.
#[test]
fn no_weight_left_leaves_a_verdict_undefined_and_an_inconclusive_term_zero() {
    let item = |weight, outcome| Item {
        first: vec![seat(U, 0.75, weight), seat(2, 0.25, 0.0)],
        extra: vec![Extra {
            nym: nym(3),
            report: Report::Revealed(0.5),
        }],
        frozen: true,
        selection: selected(0.5, Some(outcome)),
    };
    let terms = |s: &protocol::cohort_scores::Scores| -> Vec<_> {
        s.assignments.iter().map(|a| (a.baseline, a.term)).collect()
    };
    let none = Err(NoTerm::Undefined(Undefined::NoBaseline));
    let s = score(&[item(1.0, Outcome::R)], &[cohort(&[0])]).unwrap();
    let read = (Some(0.75), Ok(Some(1.0)));
    assert_eq!(
        terms(&s),
        vec![(None, none), read, (Some(0.75), Ok(Some(q(5, 8))))]
    );
    let k = &s.cohorts[0];
    assert_eq!((k.size, k.o, k.v, k.known), (1, 1, 1, Ok(0.0)));
    assert_eq!(k.value, Value::Unavailable(vec![(0, Cause::NoBaseline)]));
    let s = score(&[item(0.0, Outcome::R)], &[]).unwrap();
    assert_eq!(terms(&s), vec![(None, none); 3]);
    let s = score(&[item(0.0, Outcome::I)], &[cohort(&[0])]).unwrap();
    assert_eq!(terms(&s), vec![(None, Ok(Some(0.0))); 3]);
    assert_eq!(s.cohorts[0].value, Value::Final(0.0));
}

/// A1 B-b, `17` §8.3, §14.4: every cause of no value is kept, in the cohort's order.
#[test]
fn coexisting_causes_are_all_named() {
    let mut items = vec![pair(0.75, selected(1.0, None)); 6];
    items[0].first[0].report = Report::Missing;
    items[0].frozen = false;
    items[1].frozen = false;
    items[2].selection = Selection::Undrawn;
    items[3].first[1].weight = 0.0;
    items[3].selection = selected(1.0, Some(Outcome::A));
    items[5].selection = selected(0.5, Some(Outcome::A));
    let s = score(&items, &[cohort(&[5, 4, 3, 2, 1, 0])]).unwrap();
    let k = &s.cohorts[0];
    let causes = vec![
        (3, Cause::NoBaseline),
        (2, Cause::Undrawn),
        (1, Cause::Unfrozen),
        (0, Cause::MissingReport),
    ];
    assert_eq!(k.value, Value::Unavailable(causes));
    assert_eq!((k.size, k.o, k.v), (6, 2, 2));
    assert!(close(k.known.unwrap(), q(3, 8)));
    let undrawn = &s.assignments[4];
    assert_eq!(
        (undrawn.case, undrawn.inclusion, undrawn.term),
        (Case::Undrawn, None, Ok(None))
    );
    let mut before = pair(0.75, Selection::Undrawn);
    before.frozen = false;
    assert_eq!(one(before).unwrap().assignments[0].case, Case::Unfrozen);
}

/// `u`'s reports on j0, j1 (one group, one draw at `π`) and j2 (its own draw at ¼); `r` (nym 2)
/// reports ¾ if `z` else ¼ at weight 1, `r′` (nym 3) ½ at weight 2.
fn audited(
    ps: [f64; 3],
    z: bool,
    ys: [Outcome; 3],
    draws: (bool, bool),
    pis: (f64, f64),
) -> Vec<Item> {
    let r = if z { 0.75 } else { 0.25 };
    (0..3)
        .map(|j| {
            let (drawn, pi) = if j < 2 {
                (draws.0, pis.0)
            } else {
                (draws.1, pis.1)
            };
            Item {
                first: vec![seat(U, ps[j], 1.0), seat(2, r, 1.0), seat(3, 0.5, 2.0)],
                extra: Vec::new(),
                frozen: true,
                selection: if drawn {
                    selected(pi, Some(ys[j]))
                } else {
                    Selection::NotSelected { inclusion: pi }
                },
            }
        })
        .collect()
}

/// The construction's outcomes: probability, `z`, `(Y0, Y1, Y2)`; G1 concludes with ½, both or
/// neither; on conclusion `Y0 = A` iff `z`, `Y1 = A` with ¾ or ¼; `Y2` (½, ¼, ¼) or (¼, ½, ¼).
fn outcomes() -> Vec<(f64, bool, [Outcome; 3])> {
    use Outcome::{A, I, R};
    let mut states = Vec::new();
    for z in [false, true] {
        let (hi, lo) = if z { (0.75, 0.25) } else { (0.25, 0.75) };
        let g1 = [
            (0.5 * hi, [if z { A } else { R }, A]),
            (0.5 * lo, [if z { A } else { R }, R]),
            (0.5, [I, I]),
        ];
        let y2 = if z {
            [(A, 0.5), (R, 0.25), (I, 0.25)]
        } else {
            [(A, 0.25), (R, 0.5), (I, 0.25)]
        };
        for (p1, [y0, y1]) in g1 {
            for (y, p2) in y2 {
                states.push((0.5 * p1 * p2, z, [y0, y1, y]));
            }
        }
    }
    states
}

/// D1's reference, apart from the scorer: `(1/3) Σ_j E[g(Y_j)]`, no draw, `b = (r + 1)/3`.
fn reference(ps: [f64; 3]) -> f64 {
    let g = |p: f64, b: f64, y| match y {
        Outcome::A => (b - 1.0) * (b - 1.0) - (p - 1.0) * (p - 1.0),
        Outcome::R => b * b - p * p,
        Outcome::I => 0.0,
    };
    let term = |&(pr, z, ys): &(f64, bool, [Outcome; 3])| {
        let b = (if z { 0.75 } else { 0.25 } + 1.0) / 3.0;
        pr * (0..3).map(|j| g(ps[j], b, ys[j])).sum::<f64>()
    };
    outcomes().iter().map(term).sum::<f64>() / 3.0
}

/// `E[Ŝ_K]` through the scorer: draws of effective probability `effective`, `π` recorded `pis`.
fn through_scorer(ps: [f64; 3], effective: (f64, f64), pis: (f64, f64)) -> f64 {
    let mut total = 0.0;
    for (pr, z, ys) in outcomes() {
        for (d0, d1) in [(false, false), (false, true), (true, false), (true, true)] {
            let p0 = if d0 { effective.0 } else { 1.0 - effective.0 };
            let p1 = if d1 { effective.1 } else { 1.0 - effective.1 };
            let s = score(&audited(ps, z, ys, (d0, d1), pis), &[cohort(&[0, 1, 2])]).unwrap();
            let Value::Final(x) = s.cohorts[0].value else {
                panic!("{:?}", s.cohorts[0].value)
            };
            total += pr * p0 * p1 * x;
        }
    }
    total
}

/// A1 B-b, `17` §8.1 D1–D2, §8.6 (5): shared draws at ½ and ¼; the scorer's mean is D1's.
#[test]
fn a_shared_draw_s_weighted_mean_matches_the_reference() {
    assert_eq!(outcomes().len(), 18);
    assert!(close(outcomes().iter().map(|s| s.0).sum(), 1.0));
    let grid = [0.25, 0.5, 0.75];
    let truth = (0.5, 0.25);
    for ps in grid
        .iter()
        .flat_map(|&a| grid.iter().flat_map(move |&b| grid.map(|c| [a, b, c])))
    {
        let (path, want) = (through_scorer(ps, truth, truth), reference(ps));
        assert!(close(path, want), "{ps:?}: {path} against {want}");
    }
    assert!(close(reference([0.5; 3]), q(-41, 1728)));
    assert!(close(
        through_scorer([0.75, 0.75, 0.25], truth, truth),
        q(-13, 216)
    ));
    let off = through_scorer([0.5; 3], truth, (0.75, 0.25));
    assert!(
        close(off, q(-91, 5184)),
        "a recorded π off the effective one: {off}"
    );
}

const EPS: f64 = f64::EPSILON;

/// `17` §14.9's bounds: a baseline over `n` reports within `2nε` of its value, a verdict's term on
/// it within `(4n + 5)ε/π`.
fn baseline_near(b: Option<f64>, want: f64, n: usize) -> bool {
    matches!(b, Some(b) if (b - want).abs() <= 2.0 * n as f64 * EPS)
}

fn term_near<E>(t: Result<Option<f64>, E>, want: f64, n: usize, pi: f64) -> bool {
    matches!(t, Ok(Some(t)) if (t - want).abs() <= (4 * n + 5) as f64 * EPS / pi)
}

/// A frozen item with verdict `A` at `pi`: `u` reports `p` at weight 1, nyms 2.. `others`, each
/// (report, weight).
fn among(p: f64, others: &[(f64, f64)], pi: f64) -> Item {
    let others = others
        .iter()
        .zip(2..)
        .map(|(&(prob, w), n)| seat(n, prob, w));
    Item {
        first: std::iter::once(seat(U, p, 1.0)).chain(others).collect(),
        extra: Vec::new(),
        frozen: true,
        selection: selected(pi, Some(Outcome::A)),
    }
}

/// A1 B-b, `17` §14.9 (A): others at weight 1e308 each leave `u`'s baseline ½ and its term 0.
#[test]
fn weights_near_the_largest_f64_leave_the_baseline_the_others_mean() {
    let m = &one(among(0.5, &[(0.5, 1e308); 2], 1.0))
        .unwrap()
        .assignments[0];
    assert!(baseline_near(m.baseline, 0.5, 2), "{:?}", m.baseline);
    assert!(term_near(m.term, 0.0, 2, 1.0), "{:?}", m.term);
}

/// A1 B-b, `17` §14.9 (B): others reporting 1 at weight 1e308 each give 1 and −¼, no NaN.
#[test]
fn weights_whose_total_overflows_give_no_nan() {
    let m = &one(among(0.5, &[(1.0, 1e308); 2], 1.0))
        .unwrap()
        .assignments[0];
    assert!(baseline_near(m.baseline, 1.0, 2), "{:?}", m.baseline);
    assert!(term_near(m.term, -0.25, 2, 1.0), "{:?}", m.term);
}

/// A1 B-b, `17` §14.9 (C): a report at the smallest positive weight is `u`'s baseline, ½, not 0.
#[test]
fn the_smallest_positive_weight_still_weighs() {
    let m = &one(among(0.75, &[(0.5, f64::from_bits(1))], 1.0))
        .unwrap()
        .assignments[0];
    assert!(baseline_near(m.baseline, 0.5, 1), "{:?}", m.baseline);
    assert!(term_near(m.term, q(3, 16), 1, 1.0), "{:?}", m.term);
}

/// A1 B-b, `17` §14.9 (D): 3/16 at the smallest positive `π`, beyond `f64`, is no infinite term.
#[test]
fn a_term_beyond_f64_is_not_infinite() {
    let s = one(pair(0.75, selected(f64::from_bits(1), Some(Outcome::A)))).unwrap();
    let k = &s.cohorts[0];
    let term = k.members[0].term;
    assert!(!matches!(term, Ok(Some(t)) if !t.is_finite()), "{term:?}");
    assert!(
        !matches!(k.value, Value::Final(x) if !x.is_finite()),
        "{:?}",
        k.value
    );
    assert_eq!(term, Err(NoTerm::OutOfRange));
    assert_eq!(k.known, Err(OutOfRange));
    assert_eq!(k.value, Value::Unavailable(vec![(0, Cause::OutOfRange)]));
    assert_eq!((k.size, k.o, k.v), (1, 1, 1));
}

/// A1 B-b, `17` §14.9 (D): a pending member at the smallest positive `π` gives no infinite ends.
#[test]
fn a_half_width_beyond_f64_gives_no_infinite_interval() {
    let s = one(pair(0.75, selected(f64::from_bits(1), None))).unwrap();
    let k = &s.cohorts[0];
    assert_eq!(k.value, Value::Unavailable(vec![(0, Cause::OutOfRange)]));
    assert_eq!(
        (k.members[0].case, k.members[0].term, k.known),
        (Case::Pending, Ok(None), Ok(0.0))
    );
}

/// A1 B-b, `17` §14.9 (E): two terms near 1e308 have their mean, though their sum leaves `f64`.
#[test]
fn a_mean_in_range_is_given_when_the_sum_is_not() {
    let pi = 1e-308;
    let item = among(1.0, &[(0.0, 1.0)], pi);
    let s = score(&[item.clone(), item], &[cohort(&[0, 1])]).unwrap();
    let (t, k) = (1.0 / pi, &s.cohorts[0]);
    assert!(
        k.members.iter().all(|m| term_near(m.term, t, 1, pi)),
        "{k:?}"
    );
    let near = |x: f64| (x - t).abs() <= 11.0 * EPS * t;
    assert!(
        matches!(k.value, Value::Final(x) if near(x)),
        "{:?}",
        k.value
    );
    assert_eq!(k.known, Err(OutOfRange));
}

/// A1 B-b, `17` §14.9 (E): two pending at 1e-308 bound the mean within `±1/π`, the sum beyond.
#[test]
fn a_mean_interval_in_range_is_given_when_the_sum_s_is_not() {
    let item = pair(0.75, selected(1e-308, None));
    let s = score(&[item.clone(), item], &[cohort(&[0, 1])]).unwrap();
    let r = 1.0 / 1e-308;
    let near = |x: f64, want: f64| (x - want).abs() <= 2.0 * EPS * r;
    let value = &s.cohorts[0].value;
    let bound = matches!(value, Value::Bound { mean, .. } if near(mean.0, -r) && near(mean.1, r));
    assert!(bound, "{value:?}");
    assert!(
        matches!(
            value,
            Value::Bound {
                sum: Err(OutOfRange),
                ..
            }
        ),
        "{value:?}"
    );
}

/// A1 B-b, `17` §14.9: out of range, a term or a half-width is named beside the other causes.
#[test]
fn out_of_range_is_named_apart_from_the_other_causes() {
    let tiny = f64::from_bits(1);
    let mut items = vec![
        pair(0.75, selected(tiny, Some(Outcome::A))),
        pair(0.75, selected(tiny, None)),
        pair(0.75, selected(1.0, Some(Outcome::A))),
        pair(0.75, selected(1.0, None)),
    ];
    items[2].first[1].weight = 0.0;
    items[3].first[0].report = Report::Missing;
    items[3].frozen = false;
    let s = score(&items, &[cohort(&[3, 2, 1, 0])]).unwrap();
    let k = &s.cohorts[0];
    let causes = vec![
        (3, Cause::MissingReport),
        (2, Cause::NoBaseline),
        (1, Cause::OutOfRange),
        (0, Cause::OutOfRange),
    ];
    assert_eq!(k.value, Value::Unavailable(causes));
    let terms: Vec<_> = k.members.iter().map(|m| m.term).collect();
    let none = Err(NoTerm::Undefined(Undefined::NoBaseline));
    assert_eq!(terms, [Ok(None), none, Ok(None), Err(NoTerm::OutOfRange)]);
    assert_eq!((k.known, k.size, k.o, k.v), (Err(OutOfRange), 4, 2, 2));
}

/// A1 B-b, `17` §14.9: at the range's edge a sum in range is given and a mean stays finite.
#[test]
fn at_the_range_s_edge_sums_in_range_are_given_and_means_stay_finite() {
    let edge = among(1.0, &[(0.0, 1.0)], f64::MIN_POSITIVE);
    let k = score(&[edge.clone(), edge], &[cohort(&[0, 1])])
        .unwrap()
        .cohorts
        .remove(0);
    assert_eq!(k.known, Ok(2f64.powi(1023)));
    assert_eq!(k.value, Value::Final(2f64.powi(1022)));
    let pi = f64::from_bits((1 << 50) + 1);
    let k = score(
        &vec![among(1.0, &[(0.0, 1.0)], pi); 3],
        &[cohort(&[0, 1, 2])],
    )
    .unwrap();
    let (t, k) = (1.0 / pi, &k.cohorts[0]);
    assert!(t.is_finite() && t > f64::MAX / 2.0);
    assert_eq!(k.known, Err(OutOfRange));
    assert!(
        matches!(k.value, Value::Final(x) if (x - t).abs() <= 2.0 * EPS * t),
        "{:?}",
        k.value
    );
}
