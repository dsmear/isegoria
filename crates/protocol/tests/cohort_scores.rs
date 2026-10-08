//! B-b's scorer (`docs/17` §14.9): `π_j` below 1, selection apart from `I` and pending, counts
//! and denominator, the weighted interval, refused inputs, the supported baseline compositions,
//! and a finite enumeration with shared draws against the reference expectation.

use identity::nym::Nym;
use protocol::cohort_scores::{
    score, Case, Cause, Cohort, Counts, Extra, InputError, Item, Outcome, Panelist, Report,
    Selection, Undefined, Value,
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
        sum: (-2.0, 2.0),
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
    assert!(close(k.known, q(3, 16)));
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
    assert!(close(k.known, q(3, 16)));
    let (low, high) = (q(3, 16) - 6.0, q(3, 16) + 6.0);
    let want = Value::Bound {
        sum: (low, high),
        mean: (low / 3.0, high / 3.0),
    };
    assert_eq!(k.value, want);
    let Value::Bound { sum, .. } = k.value else {
        unreachable!()
    };
    assert!(!close(sum.1 - k.known, 2.0), "the count of pending members");
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
    let none = Err(Undefined::NoBaseline);
    let s = score(&[item(1.0, Outcome::R)], &[cohort(&[0])]).unwrap();
    let read = (Some(0.75), Ok(Some(1.0)));
    assert_eq!(
        terms(&s),
        vec![(None, none), read, (Some(0.75), Ok(Some(q(5, 8))))]
    );
    let k = &s.cohorts[0];
    assert_eq!((k.size, k.o, k.v, k.known), (1, 1, 1, 0.0));
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
    assert!(close(k.known, q(3, 8)));
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
