//! The characterization studies — T24's and T25's supplement — their grids of cells and the
//! seed of every run (`docs/13` §2, §4, §8).

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Study {
    DifNull,
    DifPower,
    DifMisspec,
    DifNonuniform,
    DifTwoAxes,
    DifPoisoning,
    DifPoolScale,
    DtfError,
    BridgingSweep,
    BridgingCapture,
    FloorNull,
    FloorPower,
    FloorMisspec,
    FloorDtf,
    BridgingLambda,
    BridgingExtra,
    FloorScreen,
}

pub const STUDIES: [Study; 17] = [
    Study::DifNull,
    Study::DifPower,
    Study::DifMisspec,
    Study::DifNonuniform,
    Study::DifTwoAxes,
    Study::DifPoisoning,
    Study::DifPoolScale,
    Study::DtfError,
    Study::BridgingSweep,
    Study::BridgingCapture,
    Study::FloorNull,
    Study::FloorPower,
    Study::FloorMisspec,
    Study::FloorDtf,
    Study::BridgingLambda,
    Study::BridgingExtra,
    Study::FloorScreen,
];

/// T24's studies (`docs/13` §4) and T25's supplement (§8), for `--study t24` and `--study t25`.
pub const T24: [Study; 10] = [
    Study::DifNull,
    Study::DifPower,
    Study::DifMisspec,
    Study::DifNonuniform,
    Study::DifTwoAxes,
    Study::DifPoisoning,
    Study::DifPoolScale,
    Study::DtfError,
    Study::BridgingSweep,
    Study::BridgingCapture,
];
pub const SUPPLEMENT: [Study; 7] = [
    Study::FloorNull,
    Study::FloorPower,
    Study::FloorMisspec,
    Study::FloorDtf,
    Study::BridgingLambda,
    Study::BridgingExtra,
    Study::FloorScreen,
];

/// The studies a `--study` value names: names, `t24`, `t25` or `all`, comma-separated.
pub fn parse_studies(value: &str) -> Result<Vec<Study>, String> {
    let mut studies = Vec::new();
    for name in value.split(',') {
        match name {
            "all" => studies.extend(STUDIES),
            "t24" => studies.extend(T24),
            "t25" => studies.extend(SUPPLEMENT),
            _ => studies.push(Study::parse(name).ok_or(format!("unknown study {name:?}"))?),
        }
    }
    Ok(studies)
}

/// What a study's runs produce, hence its record schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Dif,
    Dtf,
    Sweep,
    Capture,
    Extra,
    Screen,
}

impl Study {
    pub fn name(self) -> &'static str {
        match self {
            Study::DifNull => "dif-null",
            Study::DifPower => "dif-power",
            Study::DifMisspec => "dif-misspec",
            Study::DifNonuniform => "dif-nonuniform",
            Study::DifTwoAxes => "dif-two-axes",
            Study::DifPoisoning => "dif-poisoning",
            Study::DifPoolScale => "dif-pool-scale",
            Study::DtfError => "dtf-error",
            Study::BridgingSweep => "bridging-sweep",
            Study::BridgingCapture => "bridging-capture",
            Study::FloorNull => "floor-null",
            Study::FloorPower => "floor-power",
            Study::FloorMisspec => "floor-misspec",
            Study::FloorDtf => "floor-dtf",
            Study::BridgingLambda => "bridging-lambda",
            Study::BridgingExtra => "bridging-extra",
            Study::FloorScreen => "floor-screen",
        }
    }

    pub fn parse(name: &str) -> Option<Study> {
        STUDIES.into_iter().find(|s| s.name() == name)
    }

    pub fn kind(self) -> Kind {
        match self {
            Study::DtfError | Study::FloorDtf => Kind::Dtf,
            Study::BridgingSweep | Study::BridgingLambda => Kind::Sweep,
            Study::BridgingCapture => Kind::Capture,
            Study::BridgingExtra => Kind::Extra,
            Study::FloorScreen => Kind::Screen,
            _ => Kind::Dif,
        }
    }

    /// The `docs/08` claims and tests the study measures (`docs/13` §4).
    pub fn claims(self) -> &'static str {
        match self {
            Study::DifNull => "AT-DIF-01, DIF-008, the KR-20 floor (T53)",
            Study::DifPower => "AT-DIF-02, SC-2, STAT-001, the DIF cut (T35/T54)",
            Study::DifMisspec => "robustness to impact and guessing (docs/07 §14, D25)",
            Study::DifNonuniform => "AT-DIF-03, the discrimination-gap threshold (T40)",
            Study::DifTwoAxes => "AT-DIF-04",
            Study::DifPoisoning => "AT-DIF-07, SC-7",
            Study::DifPoolScale => "AT-DIF-09, DIF-009",
            Study::DtfError => "DIF-011, the sampling error of the DTF bound (T55)",
            Study::BridgingSweep => "SC-3, BRIDGE-002, BRIDGE-003, the gate's τ and ε",
            Study::BridgingCapture => "AT-BR-02, BRIDGE-005",
            Study::FloorNull => "AT-DIF-13, DIF-008 with a guessing floor (D25), the KR-20 floor",
            Study::FloorPower => "SC-2, STAT-001 with a guessing floor (D25), N_LATENT_MIN",
            Study::FloorMisspec => "robustness of the floor model (docs/07 §14, D25)",
            Study::FloorDtf => "DIF-011 with a guessing floor, DTF_MAX",
            Study::BridgingLambda => {
                "SC-3, BRIDGE-002, BRIDGE-003: the verdicts against (λ_b, λ_f)"
            }
            Study::BridgingExtra => "BRIDGE-006, PROTO-008: the band's extra round, k_extra and ε",
            Study::FloorScreen => {
                "IRT-003, PROTO-005: the pilot's stage-1 screen and its thresholds"
            }
        }
    }

    /// Whether the study's records carry the fitted guessing floors (`docs/13` §8.3).
    pub fn floors(self) -> bool {
        matches!(
            self,
            Study::FloorNull | Study::FloorPower | Study::FloorMisspec
        )
    }
}

/// Which trial items lean, and how (`docs/13` §3.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Layout {
    /// The first `n` trial items lean the same way on one axis.
    Campaign(usize),
    /// Two mirror pairs on one axis, `+ − + −`, the other items clean.
    Mirror,
    /// `n` items lean on each of two independent axes.
    TwoAxes(usize),
}

/// Coordinated respondents: the last `fraction` of the sample (`docs/13` §3.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Attack {
    None,
    /// They answer the last `targets` trial items wrong, the rest honestly.
    Inject {
        fraction: f64,
        targets: usize,
    },
    /// They answer the leaning items as if they leaned on nothing.
    Mask {
        fraction: f64,
    },
}

/// A latent-DIF batch: `n` respondents, `anchors` anchors, `k` trial items (`docs/13` §3.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DifDesign {
    pub n: usize,
    pub anchors: usize,
    pub k: usize,
    pub layout: Layout,
    pub delta: f64,
    pub alpha: f64,
    pub pi: f64,
    pub impact: f64,
    pub guess: f64,
    pub attack: Attack,
    /// The format the fit is told: 0 an open answer, otherwise `m` options (`docs/02` §B.1).
    pub options: u8,
    /// Each column's floor is drawn from `guess ± spread`.
    pub spread: f64,
    /// The skew-normal shape of the ability distribution, standardized; 0 for a normal.
    pub skew: f64,
    /// The spread of a respondent's effect shared by each pair of trial items (a template).
    pub testlet: f64,
}

impl Default for DifDesign {
    fn default() -> Self {
        DifDesign {
            n: 3000,
            anchors: 60,
            k: 8,
            layout: Layout::Campaign(0),
            delta: 0.0,
            alpha: 0.0,
            pi: 0.5,
            impact: 0.0,
            guess: 0.0,
            attack: Attack::None,
            options: 0,
            spread: 0.0,
            skew: 0.0,
            testlet: 0.0,
        }
    }
}

/// The Level A mirror design (`docs/13` §3.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SweepDesign {
    pub n: usize,
    pub share: f64,
    pub per_reviewer: usize,
    pub noise: f64,
    /// `(λ_b, λ_f)` of the fit; None for the production values.
    pub lambda: Option<(f64, f64)>,
}

/// The mirror design plus probe items near `τ`, each rated by a panel (`docs/13` §8.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtraDesign {
    pub n: usize,
    pub share: f64,
    pub per_reviewer: usize,
    pub noise: f64,
    pub panel: usize,
}

/// A stage-1 pilot of `n` respondents, `anchors` anchors and fixed trial items, every column
/// a choice among `options` options (`docs/13` §8.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenDesign {
    pub n: usize,
    pub anchors: usize,
    pub options: u8,
}

/// Boosters on one partisan item of the reference fixture (`docs/13` §3.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaptureDesign {
    pub item: usize,
    pub own: usize,
    pub step: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Dif(DifDesign),
    Sweep(SweepDesign),
    Capture(CaptureDesign),
    Extra(ExtraDesign),
    Screen(ScreenDesign),
}

fn layout_key(layout: Layout) -> String {
    match layout {
        Layout::Campaign(n) => format!("c{n}"),
        Layout::Mirror => "m".to_string(),
        Layout::TwoAxes(n) => format!("x{n}"),
    }
}

fn parse_layout(s: &str) -> Option<Layout> {
    match s.split_at_checked(1)? {
        ("c", n) => Some(Layout::Campaign(n.parse().ok()?)),
        ("m", "") => Some(Layout::Mirror),
        ("x", n) => Some(Layout::TwoAxes(n.parse().ok()?)),
        _ => None,
    }
}

fn attack_key(attack: Attack) -> String {
    match attack {
        Attack::None => "none".to_string(),
        Attack::Inject { fraction, targets } => format!("inj:{fraction}:{targets}"),
        Attack::Mask { fraction } => format!("mask:{fraction}"),
    }
}

fn parse_attack(s: &str) -> Option<Attack> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.as_slice() {
        ["none"] => Some(Attack::None),
        ["inj", f, t] => Some(Attack::Inject {
            fraction: f.parse().ok()?,
            targets: t.parse().ok()?,
        }),
        ["mask", f] => Some(Attack::Mask {
            fraction: f.parse().ok()?,
        }),
        _ => None,
    }
}

/// `key=value` pairs separated by spaces, exactly the `names` given, in any order.
fn fields<'a>(key: &'a str, names: &[&str]) -> Option<BTreeMap<&'a str, &'a str>> {
    let map: BTreeMap<&str, &str> = key
        .split(' ')
        .map(|kv| kv.split_once('='))
        .collect::<Option<_>>()?;
    let expected = map.len() == names.len() && names.iter().all(|n| map.contains_key(n));
    expected.then_some(map)
}

/// The optional fields of a latent-DIF key, present only when not at their default.
const DIF_OPTIONAL: [&str; 4] = ["m", "gs", "sk", "tl"];

impl Cell {
    /// The cell's name in records and seeds: stable, space-separated `key=value` pairs.
    pub fn key(&self) -> String {
        match self {
            Cell::Dif(d) => {
                let mut key = format!(
                    "n={} a={} k={} lay={} d={} al={} pi={} im={} g={} atk={}",
                    d.n,
                    d.anchors,
                    d.k,
                    layout_key(d.layout),
                    d.delta,
                    d.alpha,
                    d.pi,
                    d.impact,
                    d.guess,
                    attack_key(d.attack)
                );
                let extra = [
                    ("m", d.options != 0, d.options.to_string()),
                    ("gs", d.spread != 0.0, d.spread.to_string()),
                    ("sk", d.skew != 0.0, d.skew.to_string()),
                    ("tl", d.testlet != 0.0, d.testlet.to_string()),
                ];
                for (name, present, value) in extra {
                    if present {
                        key.push_str(&format!(" {name}={value}"));
                    }
                }
                key
            }
            Cell::Sweep(s) => {
                let key = format!("n={} s={} r={} e={}", s.n, s.share, s.per_reviewer, s.noise);
                match s.lambda {
                    None => key,
                    Some((b, f)) => format!("{key} lb={b} lf={f}"),
                }
            }
            Cell::Capture(c) => format!("item={} own={} step={}", c.item, c.own, c.step),
            Cell::Extra(e) => format!(
                "n={} s={} r={} e={} panel={}",
                e.n, e.share, e.per_reviewer, e.noise, e.panel
            ),
            Cell::Screen(s) => format!("n={} a={} m={}", s.n, s.anchors, s.options),
        }
    }

    pub fn parse(study: Study, key: &str) -> Option<Cell> {
        match study.kind() {
            Kind::Dif | Kind::Dtf => {
                let base = ["n", "a", "k", "lay", "d", "al", "pi", "im", "g", "atk"];
                let present: Vec<&str> = DIF_OPTIONAL
                    .into_iter()
                    .filter(|name| key.contains(&format!(" {name}=")))
                    .collect();
                let names: Vec<&str> = base.iter().copied().chain(present).collect();
                let f = fields(key, &names)?;
                let optional = |name: &str| f.get(name).copied();
                let options = optional("m").map_or(Some(0), |v| v.parse().ok())?;
                let spread = optional("gs").map_or(Some(0.0), |v| v.parse().ok())?;
                let skew = optional("sk").map_or(Some(0.0), |v| v.parse().ok())?;
                let testlet = optional("tl").map_or(Some(0.0), |v| v.parse().ok())?;
                let default = |v: f64, present: &str| v == 0.0 && f.contains_key(present);
                if options < 2 && f.contains_key("m")
                    || default(spread, "gs")
                    || default(skew, "sk")
                    || default(testlet, "tl")
                {
                    return None;
                }
                Some(Cell::Dif(DifDesign {
                    n: f["n"].parse().ok()?,
                    anchors: f["a"].parse().ok()?,
                    k: f["k"].parse().ok()?,
                    layout: parse_layout(f["lay"])?,
                    delta: f["d"].parse().ok()?,
                    alpha: f["al"].parse().ok()?,
                    pi: f["pi"].parse().ok()?,
                    impact: f["im"].parse().ok()?,
                    guess: f["g"].parse().ok()?,
                    attack: parse_attack(f["atk"])?,
                    options,
                    spread,
                    skew,
                    testlet,
                }))
            }
            Kind::Sweep => {
                let tuned = key.contains(" lb=");
                let names: &[&str] = if tuned {
                    &["n", "s", "r", "e", "lb", "lf"]
                } else {
                    &["n", "s", "r", "e"]
                };
                let f = fields(key, names)?;
                let lambda = if tuned {
                    Some((f["lb"].parse().ok()?, f["lf"].parse().ok()?))
                } else {
                    None
                };
                Some(Cell::Sweep(SweepDesign {
                    n: f["n"].parse().ok()?,
                    share: f["s"].parse().ok()?,
                    per_reviewer: f["r"].parse().ok()?,
                    noise: f["e"].parse().ok()?,
                    lambda,
                }))
            }
            Kind::Capture => {
                let f = fields(key, &["item", "own", "step"])?;
                Some(Cell::Capture(CaptureDesign {
                    item: f["item"].parse().ok()?,
                    own: f["own"].parse().ok()?,
                    step: f["step"].parse().ok()?,
                }))
            }
            Kind::Extra => {
                let f = fields(key, &["n", "s", "r", "e", "panel"])?;
                Some(Cell::Extra(ExtraDesign {
                    n: f["n"].parse().ok()?,
                    share: f["s"].parse().ok()?,
                    per_reviewer: f["r"].parse().ok()?,
                    noise: f["e"].parse().ok()?,
                    panel: f["panel"].parse().ok()?,
                }))
            }
            Kind::Screen => {
                let f = fields(key, &["n", "a", "m"])?;
                Some(Cell::Screen(ScreenDesign {
                    n: f["n"].parse().ok()?,
                    anchors: f["a"].parse().ok()?,
                    options: f["m"].parse().ok().filter(|&m: &u8| m >= 2)?,
                }))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grid {
    /// The grid `docs/13` §4 specifies.
    Full,
    /// One tiny cell per study: for tests and a first check of a machine.
    Smoke,
}

fn dif(d: DifDesign) -> Cell {
    Cell::Dif(d)
}

/// Every column with the floor `guess`, declared with `options` options (`docs/13` §8.2).
fn floored(guess: f64, options: u8, d: DifDesign) -> DifDesign {
    DifDesign {
        guess,
        options,
        ..d
    }
}

/// The `(λ_b, λ_f)` of `bridging-lambda`: a third, the production value and three times it,
/// `λ_b > λ_f` (`docs/08` §0).
pub const LAMBDAS: [(f64, f64); 8] = [
    (0.05, 0.01),
    (0.05, 0.03),
    (0.15, 0.01),
    (0.15, 0.03),
    (0.15, 0.09),
    (0.45, 0.01),
    (0.45, 0.03),
    (0.45, 0.09),
];

fn biased(n: usize, k: usize, count: usize, delta: f64, pi: f64) -> DifDesign {
    DifDesign {
        n,
        k,
        layout: Layout::Campaign(count),
        delta,
        pi,
        ..DifDesign::default()
    }
}

fn full(study: Study) -> Vec<Cell> {
    let mut out = Vec::new();
    match study {
        Study::DifNull => {
            for n in [1500, 3000, 6000] {
                for k in [4, 8, 16] {
                    for anchors in [20, 40, 60] {
                        out.push(dif(DifDesign {
                            anchors,
                            ..biased(n, k, 0, 0.0, 0.5)
                        }));
                    }
                }
            }
        }
        Study::DifPower => {
            for n in [1500, 3000, 6000] {
                for delta in [0.3, 0.5, 0.7, 0.9] {
                    for count in [1, 2, 3] {
                        for pi in [0.5, 0.3, 0.1] {
                            out.push(dif(biased(n, 8, count, delta, pi)));
                        }
                    }
                }
            }
            for k in [4, 16] {
                for delta in [0.5, 0.9] {
                    for count in [1, 2, 3] {
                        for pi in [0.5, 0.3] {
                            out.push(dif(biased(3000, k, count, delta, pi)));
                        }
                    }
                }
            }
            for anchors in [20, 40] {
                for delta in [0.7, 0.9] {
                    for count in [2, 3] {
                        out.push(dif(DifDesign {
                            anchors,
                            ..biased(3000, 8, count, delta, 0.5)
                        }));
                    }
                }
            }
        }
        Study::DifMisspec => {
            for impact in [0.0, 0.5, 1.0] {
                for guess in [0.0, 0.2] {
                    for count in [0, 2] {
                        for pi in [0.5, 0.2] {
                            if impact == 0.0 && count == 0 && pi != 0.5 {
                                continue;
                            }
                            let delta = if count > 0 { 0.9 } else { 0.0 };
                            out.push(dif(DifDesign {
                                impact,
                                guess,
                                ..biased(3000, 8, count, delta, pi)
                            }));
                        }
                    }
                }
            }
        }
        Study::DifNonuniform => {
            for (count, alphas, deltas) in
                [(2, [0.4, 0.8], [0.0, 0.5]), (3, [0.8, 1.6], [0.0, 0.9])]
            {
                for n in [3000, 6000] {
                    for alpha in alphas {
                        for delta in deltas {
                            out.push(dif(DifDesign {
                                alpha,
                                ..biased(n, 8, count, delta, 0.5)
                            }));
                        }
                    }
                }
            }
        }
        Study::DifTwoAxes => {
            for n in [3000, 6000] {
                for delta in [0.5, 0.9] {
                    out.push(dif(DifDesign {
                        layout: Layout::TwoAxes(2),
                        ..biased(n, 8, 0, delta, 0.5)
                    }));
                }
            }
        }
        Study::DifPoisoning => {
            for fraction in [0.0, 0.01, 0.02, 0.05, 0.1] {
                for targets in [1, 2] {
                    out.push(dif(DifDesign {
                        attack: Attack::Inject { fraction, targets },
                        ..DifDesign::default()
                    }));
                }
            }
            for fraction in [0.0, 0.01, 0.02, 0.05, 0.1] {
                out.push(dif(DifDesign {
                    attack: Attack::Mask { fraction },
                    ..biased(3000, 8, 2, 0.9, 0.5)
                }));
            }
        }
        Study::DifPoolScale => {
            for k in [32, 100] {
                out.push(dif(biased(3000, k, 0, 0.0, 0.5)));
                out.push(dif(biased(3000, k, k / 10, 0.9, 0.5)));
            }
        }
        Study::DtfError => {
            for n in [3000, 6000] {
                for delta in [0.5, 0.9] {
                    for pi in [0.5, 0.3] {
                        out.push(dif(DifDesign {
                            layout: Layout::Mirror,
                            ..biased(n, 8, 0, delta, pi)
                        }));
                    }
                }
            }
        }
        Study::BridgingSweep => {
            for n in [100, 200, 800] {
                for share in [0.5, 0.6, 0.8] {
                    for per_reviewer in [5, 9] {
                        for noise in [0.07, 0.15] {
                            out.push(Cell::Sweep(SweepDesign {
                                n,
                                share,
                                per_reviewer,
                                noise,
                                lambda: None,
                            }));
                        }
                    }
                }
            }
        }
        Study::BridgingCapture => {
            for item in [7, 8] {
                for own in [0, 40] {
                    out.push(Cell::Capture(CaptureDesign { item, own, step: 5 }));
                }
            }
        }
        Study::FloorNull => {
            for (guess, options, spread) in
                [(0.2, 5, 0.0), (0.25, 4, 0.0), (0.5, 2, 0.0), (0.2, 5, 0.1)]
            {
                for n in [3000, 6000] {
                    out.push(dif(DifDesign {
                        spread,
                        ..floored(guess, options, biased(n, 8, 0, 0.0, 0.5))
                    }));
                }
            }
            for (guess, options) in [(0.2, 5), (0.5, 2)] {
                for anchors in [20, 40] {
                    out.push(dif(DifDesign {
                        anchors,
                        ..floored(guess, options, biased(3000, 8, 0, 0.0, 0.5))
                    }));
                }
            }
        }
        Study::FloorPower => {
            let (n3, n6, n12) = (
                [(0.9, 2), (0.9, 3), (0.7, 3)],
                [(0.7, 2), (0.9, 2), (0.7, 3), (0.9, 3)],
                [(0.7, 2), (0.9, 2)],
            );
            for (guess, options) in [(0.2, 5), (0.5, 2)] {
                for (n, shifts) in [(3000, &n3[..]), (6000, &n6[..]), (12000, &n12[..])] {
                    for &(delta, count) in shifts {
                        out.push(dif(floored(
                            guess,
                            options,
                            biased(n, 8, count, delta, 0.5),
                        )));
                    }
                }
            }
            for n in [6000, 12000] {
                out.push(dif(floored(0.25, 4, biased(n, 8, 2, 0.9, 0.5))));
            }
            for count in [2, 3] {
                out.push(dif(DifDesign {
                    anchors: 40,
                    ..floored(0.2, 5, biased(6000, 8, count, 0.9, 0.5))
                }));
            }
            out.push(dif(floored(0.2, 5, biased(12000, 8, 2, 0.9, 0.3))));
        }
        Study::FloorMisspec => {
            let base = |count: usize| {
                let delta = if count > 0 { 0.9 } else { 0.0 };
                floored(0.2, 5, biased(3000, 8, count, delta, 0.5))
            };
            for count in [0, 3] {
                out.push(dif(DifDesign {
                    impact: 1.0,
                    ..base(count)
                }));
                for skew in [-4.0, -2.0, 4.0] {
                    out.push(dif(DifDesign {
                        skew,
                        ..base(count)
                    }));
                }
                for guess in [0.1, 0.3] {
                    out.push(dif(DifDesign {
                        guess,
                        ..base(count)
                    }));
                }
                out.push(dif(DifDesign {
                    testlet: 1.0,
                    ..base(count)
                }));
            }
            out.push(dif(DifDesign {
                testlet: 0.5,
                ..base(0)
            }));
            out.push(dif(DifDesign {
                spread: 0.1,
                ..base(3)
            }));
            let open = biased(3000, 8, 0, 0.0, 0.5);
            out.push(dif(DifDesign { skew: -4.0, ..open }));
            out.push(dif(DifDesign {
                testlet: 1.0,
                ..open
            }));
        }
        Study::FloorDtf => {
            for (guess, options, deltas) in [(0.2, 5, &[0.5, 0.9][..]), (0.5, 2, &[0.9][..])] {
                for n in [3000, 6000] {
                    for &delta in deltas {
                        out.push(dif(DifDesign {
                            layout: Layout::Mirror,
                            ..floored(guess, options, biased(n, 8, 0, delta, 0.5))
                        }));
                    }
                }
            }
        }
        Study::BridgingLambda => {
            for n in [100, 200, 800] {
                for share in [0.6, 0.8] {
                    for lambda in LAMBDAS {
                        out.push(Cell::Sweep(SweepDesign {
                            n,
                            share,
                            per_reviewer: 5,
                            noise: 0.15,
                            lambda: Some(lambda),
                        }));
                    }
                }
            }
        }
        Study::BridgingExtra => {
            for n in [100, 200, 800] {
                for share in [0.5, 0.8] {
                    for panel in [7, 11] {
                        out.push(Cell::Extra(ExtraDesign {
                            n,
                            share,
                            per_reviewer: 5,
                            noise: 0.15,
                            panel,
                        }));
                    }
                }
            }
        }
        Study::FloorScreen => {
            for n in [300, 600, 1500] {
                for anchors in [30, 60] {
                    for options in [2, 4, 5] {
                        out.push(Cell::Screen(ScreenDesign {
                            n,
                            anchors,
                            options,
                        }));
                    }
                }
            }
        }
    }
    out
}

fn smoke(study: Study) -> Vec<Cell> {
    let tiny = |layout, delta| DifDesign {
        n: 400,
        anchors: 10,
        k: 4,
        layout,
        delta,
        ..DifDesign::default()
    };
    let one = |d: DifDesign| vec![dif(d)];
    match study {
        Study::DifNull => one(tiny(Layout::Campaign(0), 0.0)),
        Study::DifPower => one(tiny(Layout::Campaign(2), 0.9)),
        Study::DifMisspec => one(DifDesign {
            impact: 0.5,
            guess: 0.2,
            pi: 0.2,
            ..tiny(Layout::Campaign(2), 0.9)
        }),
        Study::DifNonuniform => one(DifDesign {
            alpha: 0.8,
            ..tiny(Layout::Campaign(2), 0.0)
        }),
        Study::DifTwoAxes => one(tiny(Layout::TwoAxes(1), 0.9)),
        Study::DifPoisoning => vec![
            dif(DifDesign {
                attack: Attack::Inject {
                    fraction: 0.1,
                    targets: 1,
                },
                ..tiny(Layout::Campaign(0), 0.0)
            }),
            dif(DifDesign {
                attack: Attack::Mask { fraction: 0.1 },
                ..tiny(Layout::Campaign(2), 0.9)
            }),
        ],
        Study::DifPoolScale => one(DifDesign {
            k: 12,
            ..tiny(Layout::Campaign(0), 0.0)
        }),
        Study::DtfError => one(DifDesign {
            k: 8,
            ..tiny(Layout::Mirror, 0.9)
        }),
        Study::BridgingSweep => vec![Cell::Sweep(SweepDesign {
            n: 40,
            share: 0.6,
            per_reviewer: 5,
            noise: 0.07,
            lambda: None,
        })],
        Study::BridgingCapture => vec![Cell::Capture(CaptureDesign {
            item: 7,
            own: 40,
            step: 40,
        })],
        Study::FloorNull => one(floored(0.2, 5, tiny(Layout::Campaign(0), 0.0))),
        Study::FloorPower => one(floored(0.2, 5, tiny(Layout::Campaign(2), 0.9))),
        Study::FloorMisspec => one(DifDesign {
            impact: 0.5,
            spread: 0.1,
            skew: 4.0,
            testlet: 1.0,
            ..floored(0.2, 5, tiny(Layout::Campaign(2), 0.9))
        }),
        Study::FloorDtf => one(DifDesign {
            k: 8,
            ..floored(0.2, 5, tiny(Layout::Mirror, 0.9))
        }),
        Study::BridgingLambda => vec![Cell::Sweep(SweepDesign {
            n: 40,
            share: 0.6,
            per_reviewer: 5,
            noise: 0.07,
            lambda: Some((0.05, 0.01)),
        })],
        Study::BridgingExtra => vec![Cell::Extra(ExtraDesign {
            n: 40,
            share: 0.6,
            per_reviewer: 5,
            noise: 0.07,
            panel: 7,
        })],
        Study::FloorScreen => vec![Cell::Screen(ScreenDesign {
            n: 300,
            anchors: 30,
            options: 4,
        })],
    }
}

pub fn cells(study: Study, grid: Grid) -> Vec<Cell> {
    match grid {
        Grid::Full => full(study),
        Grid::Smoke => smoke(study),
    }
}

pub fn replicates(study: Study, grid: Grid) -> u32 {
    match (grid, study) {
        (Grid::Full, Study::DifPoolScale) => 3,
        (Grid::Full, Study::FloorPower | Study::FloorMisspec) => 100,
        (Grid::Full, _) => 200,
        (Grid::Smoke, _) => 2,
    }
}

/// One run: a replicate of a cell of a study.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub study: Study,
    pub cell: Cell,
    pub replicate: u32,
}

impl Task {
    pub fn seed(&self) -> u64 {
        seed(self.study, &self.cell.key(), self.replicate)
    }
}

fn first_eight(digest: &[u8]) -> u64 {
    let mut first = [0u8; 8];
    first.copy_from_slice(&digest[..8]);
    u64::from_le_bytes(first)
}

/// The run's seed: SHA-256 over the study, the cell key and the replicate, length-prefixed.
pub fn seed(study: Study, key: &str, replicate: u32) -> u64 {
    let mut h = Sha256::new();
    h.update(b"isegoria/characterization/v1");
    for part in [study.name().as_bytes(), key.as_bytes()] {
        h.update((part.len() as u64).to_le_bytes());
        h.update(part);
    }
    h.update(replicate.to_le_bytes());
    first_eight(&h.finalize())
}

/// The seed of the engine's own random starts, drawn apart from the population's stream.
pub fn engine_seed(seed: u64) -> u64 {
    let mut h = Sha256::new();
    h.update(b"isegoria/characterization/engine/v1");
    h.update(seed.to_le_bytes());
    first_eight(&h.finalize())
}

/// The seed of the `index`-th draw of `domain` inside a run (a panel, an extra round), apart
/// from the population's stream.
pub fn draw_seed(domain: &str, seed: u64, index: u64) -> u64 {
    let mut h = Sha256::new();
    h.update(b"isegoria/characterization/draw/v1");
    h.update((domain.len() as u64).to_le_bytes());
    h.update(domain.as_bytes());
    h.update(seed.to_le_bytes());
    h.update(index.to_le_bytes());
    first_eight(&h.finalize())
}

/// The runs of `studies`, replicate by replicate so that a partial run covers every cell.
/// `replicates` caps each study's own count; `filter` keeps the cells whose key contains it.
pub fn tasks(
    studies: &[Study],
    grid: Grid,
    replicates_cap: Option<u32>,
    filter: Option<&str>,
) -> Vec<Task> {
    let mut distinct: Vec<Study> = Vec::new();
    for &s in studies {
        if !distinct.contains(&s) {
            distinct.push(s);
        }
    }
    let plan: Vec<(Study, Vec<Cell>, u32)> = distinct
        .iter()
        .map(|&s| {
            let kept = cells(s, grid)
                .into_iter()
                .filter(|c| filter.is_none_or(|f| c.key().contains(f)))
                .collect();
            let own = replicates(s, grid);
            (s, kept, replicates_cap.map_or(own, |cap| cap.min(own)))
        })
        .collect();
    let most = plan.iter().map(|p| p.2).max().unwrap_or(0);
    let mut out = Vec::new();
    for replicate in 0..most {
        for (study, kept, reps) in &plan {
            if replicate < *reps {
                out.extend(kept.iter().map(|&cell| Task {
                    study: *study,
                    cell,
                    replicate,
                }));
            }
        }
    }
    out
}
