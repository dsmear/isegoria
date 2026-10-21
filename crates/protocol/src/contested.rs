//! The contested-facts pool (`docs/05` [7b], `docs/02` §B.7, `docs/01` D38): DIF items whose
//! key the source check established, drawn only in selections whose admission cost — each fit's
//! estimated DTF, summed — is within the tolerance; no bound on a test's DTF (`docs/15` A2).

use crate::randomness::{Beacon, CONTESTED};
use network::cid::Cid;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use scoring::dtf::ClassCurves;

/// `2³²`: the cost is summed in whole units of `2⁻³²` score points, each DTF rounded up.
const SCALE: f64 = 4_294_967_296.0;

fn units(dtf: f64) -> u64 {
    (dtf * SCALE).ceil() as u64
}

/// The draw's resource domain (`docs/15` D3): the most it may visit, copy, evaluate and tabulate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawLimits {
    pub visits: u64,
    pub member_entries: u64,
    pub point_terms: u64,
    pub table_cells: u64,
    pub table_scans: u64,
}

pub const DRAW_LIMITS: DrawLimits = DrawLimits {
    visits: 1 << 20,
    member_entries: 1 << 22,
    point_terms: 1 << 29,
    table_cells: 1 << 20,
    table_scans: 1 << 26,
};

/// The pool's domain (`docs/15` D3): per fit its classes, points and items; the fits, and the
/// pool's estimated bytes, [`FitShape::bytes`] summed over the fits plus [`POOL_FIXED_BYTES`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolLimits {
    pub classes: usize,
    pub points: usize,
    pub items: usize,
    pub fits: usize,
    pub bytes: u64,
}

pub const POOL_LIMITS: PoolLimits = PoolLimits {
    classes: 16,
    points: 1 << 12,
    items: 1 << 12,
    fits: 1 << 12,
    bytes: 1 << 27,
};

/// The buffer of fits at the fit limit, 80 bytes a fit with twice the capacity and an old copy.
pub const POOL_FIXED_BYTES: u64 = 240 * POOL_LIMITS.fits as u64;

/// A fit's dimensions: members `M`, classes `G`, ability points `Q`, items `K`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FitShape {
    pub members: usize,
    pub classes: usize,
    pub points: usize,
    pub items: usize,
}

impl FitShape {
    /// The pool's estimate of the fit's heap bytes, glibc's chunk overhead included (`docs/15`
    /// D3): `8KGQ + 40KG + 40K + 8Q + 42M + 48`; `None` past `u128`.
    pub fn bytes(&self) -> Option<u128> {
        let [m, g, q, k] = [self.members, self.classes, self.points, self.items].map(wide);
        let (m, g, q, k) = (m?, g?, q?, k?);
        let rows = k.checked_mul(g)?;
        rows.checked_mul(q)?
            .checked_mul(8)?
            .checked_add(rows.checked_mul(40)?)?
            .checked_add(k.checked_mul(40)?)?
            .checked_add(q.checked_mul(8)?)?
            .checked_add(m.checked_mul(42)?)?
            .checked_add(48)
    }

    /// The fit alone, in an empty pool, within [`POOL_LIMITS`]: what building its curves may cost,
    /// whatever its members (`docs/15` D3). It allocates nothing.
    pub fn buildable(&self) -> Result<(), PoolLimit> {
        within_fit(self)?;
        within_bytes(
            self.bytes()
                .and_then(|b| b.checked_add(u128::from(POOL_FIXED_BYTES))),
        )
    }
}

fn wide(x: usize) -> Option<u128> {
    u128::try_from(x).ok()
}

/// The draw limit a request exceeds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawLimit {
    TableCells,
    Visits,
    TableScans,
    MemberEntries,
    PointTerms,
}

/// A request outside the draw's domain: the pool was not evaluated, so it says nothing of whether
/// a balanced selection exists. `reached`: a count past `limit` so far, `None` past `u128`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutOfDomain {
    pub limit: DrawLimit,
    pub reached: Option<u128>,
}

/// Exact counts of one draw (`docs/15` D3): subsets visited, their member entries, DTF point
/// terms, the least-cost table's cells, and `(n + 1) · visits` bounding the table's scans.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawCounts {
    pub visits: u128,
    pub member_entries: u128,
    pub point_terms: u128,
    pub table_cells: u128,
    pub table_scans: u128,
}

fn bounded(limit: DrawLimit, value: Option<u128>, max: u64) -> Result<u128, OutOfDomain> {
    match value {
        Some(count) if count <= u128::from(max) => Ok(count),
        reached => Err(OutOfDomain { limit, reached }),
    }
}

/// [`DrawCounts`] of drawing `n` from fits of these shapes, checked against `limits` in the order
/// [`DrawLimit`] lists them and stopped at the first exceeded; it allocates nothing.
pub fn draw_counts<I>(fits: I, n: usize, limits: &DrawLimits) -> Result<DrawCounts, OutOfDomain>
where
    I: IntoIterator<Item = FitShape>,
    I::IntoIter: ExactSizeIterator,
{
    let fits = fits.into_iter();
    let sizes = wide(n).and_then(|n| n.checked_add(1));
    let cells = wide(fits.len())
        .and_then(|m| m.checked_add(1))
        .zip(sizes)
        .and_then(|(m, s)| m.checked_mul(s));
    let table_cells = bounded(DrawLimit::TableCells, cells, limits.table_cells)?;
    let (mut visits, mut entries, mut terms, mut scans) = (0u128, 0u128, 0u128, 0u128);
    for fit in fits {
        let members = wide(fit.members);
        let classes = wide(fit.classes);
        let pairs = classes.and_then(|g| g.checked_mul(g.saturating_sub(1)));
        let per_entry = pairs
            .zip(wide(fit.points))
            .and_then(|(p, q)| (p / 2).checked_mul(q));
        let mut subsets = Some(1u128);
        for r in 0..=fit.members.min(n) {
            let size = wide(r);
            let count = subsets.and_then(|c| visits.checked_add(c));
            visits = bounded(DrawLimit::Visits, count, limits.visits)?;
            let count = sizes.and_then(|s| s.checked_mul(visits));
            scans = bounded(DrawLimit::TableScans, count, limits.table_scans)?;
            let added = subsets.zip(size).and_then(|(c, r)| c.checked_mul(r));
            let count = added.and_then(|a| entries.checked_add(a));
            entries = bounded(DrawLimit::MemberEntries, count, limits.member_entries)?;
            let added = added.and_then(|a| match a {
                0 => Some(0),
                a => per_entry.and_then(|k| a.checked_mul(k)),
            });
            let count = added.and_then(|a| terms.checked_add(a));
            terms = bounded(DrawLimit::PointTerms, count, limits.point_terms)?;
            let left = members.zip(size).and_then(|(m, r)| m.checked_sub(r));
            subsets = subsets
                .zip(left)
                .and_then(|(c, l)| c.checked_mul(l))
                .zip(size.and_then(|r| r.checked_add(1)))
                .map(|(c, d)| c / d);
        }
    }
    Ok(DrawCounts {
        visits,
        member_entries: entries,
        point_terms: terms,
        table_cells,
        table_scans: scans,
    })
}

fn within_fit(shape: &FitShape) -> Result<(), PoolLimit> {
    if shape.classes > POOL_LIMITS.classes {
        Err(PoolLimit::Classes)
    } else if shape.points > POOL_LIMITS.points {
        Err(PoolLimit::Points)
    } else if shape.items > POOL_LIMITS.items {
        Err(PoolLimit::Items)
    } else {
        Ok(())
    }
}

fn within_bytes(bytes: Option<u128>) -> Result<(), PoolLimit> {
    match bytes {
        Some(b) if b <= u128::from(POOL_LIMITS.bytes) => Ok(()),
        _ => Err(PoolLimit::Bytes),
    }
}

fn sorted(items: impl Iterator<Item = Cid>) -> Vec<[u8; 32]> {
    let mut ids: Vec<[u8; 32]> = items.map(|c| c.0).collect();
    ids.sort_unstable();
    ids
}

#[derive(Clone, Debug, PartialEq)]
struct Fit {
    curves: ClassCurves,
    members: Vec<(Cid, usize)>,
}

impl Fit {
    fn shape(&self) -> FitShape {
        FitShape {
            members: self.members.len(),
            classes: self.curves.classes(),
            points: self.curves.points(),
            items: self.curves.items(),
        }
    }
}

/// A fit's subset the draw may take: positions in `Fit::members`, and its DTF in units.
struct Candidate {
    members: Vec<usize>,
    units: u64,
}

/// Contested facts grouped by the latent fit that last measured them, in canonical order:
/// members by content id within a fit, fits by their least member (`docs/02` §B.7).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContestedPool {
    fits: Vec<Fit>,
}

/// Why a fit's members were not recorded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordError {
    /// `index` is `usize::MAX` for an index past `usize`.
    IndexOutOfRange { index: usize, items: usize },
    /// The same item, or the same index, listed twice (more members than items always do).
    DuplicateMember,
    /// The fit, or the pool it would leave, is outside [`POOL_LIMITS`]; nothing was recorded.
    OutOfDomain(PoolLimit),
}

/// The pool limit a record exceeds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoolLimit {
    Classes,
    Points,
    Items,
    Fits,
    Bytes,
}

/// No selection of `requested` facts is balanced; `feasible` lists the sizes up to it that are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoBalancedDraw {
    pub requested: usize,
    pub feasible: Vec<usize>,
}

/// Why a draw returned no selection: the pool evaluated in full, or the request not evaluated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawError {
    NoBalancedDraw(NoBalancedDraw),
    OutOfDomain(OutOfDomain),
}

impl ContestedPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.fits.iter().map(|f| f.members.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.fits.is_empty()
    }

    pub fn contains(&self, item: &Cid) -> bool {
        self.fits
            .iter()
            .any(|f| f.members.iter().any(|(c, _)| c == item))
    }

    /// Records `curves` as the latest fit of `members` (an item and its index in the fit) once
    /// [`ContestedPool::admits`] passes: a member already in the pool moves to it (`docs/05` [7b]);
    /// with no members nothing is recorded. A refusal changes nothing.
    pub fn record(
        &mut self,
        curves: ClassCurves,
        members: &[(Cid, usize)],
    ) -> Result<(), RecordError> {
        self.admits(curves.classes(), curves.points(), curves.items(), members)?;
        self.take_out(members.iter().map(|&(c, _)| c));
        if !members.is_empty() {
            self.fits.push(Fit {
                curves,
                members: members.to_vec(),
            });
            self.canonicalize();
        }
        Ok(())
    }

    /// Whether a fit of `classes`, `points` and `items` may record `members` (`docs/15` D3): their
    /// indices and count, the fit alone ([`FitShape::buildable`]), repeats, then the pool it would
    /// leave with the facts it moves counted out; only that last check copies the members' ids.
    pub fn admits<J>(
        &self,
        classes: usize,
        points: usize,
        items: usize,
        members: &[(Cid, J)],
    ) -> Result<(), RecordError>
    where
        J: Copy + PartialEq + TryInto<usize>,
    {
        let position = |j: J| j.try_into().ok();
        let outside = |&&(_, j): &&(Cid, J)| position(j).is_none_or(|j| j >= items);
        if let Some(&(_, j)) = members.iter().find(outside) {
            let index = position(j).unwrap_or(usize::MAX);
            return Err(RecordError::IndexOutOfRange { index, items });
        }
        // Every index is below `items` here, so more members than items repeat one.
        if members.len() > items {
            return Err(RecordError::DuplicateMember);
        }
        let shape = FitShape {
            members: members.len(),
            classes,
            points,
            items,
        };
        shape.buildable().map_err(RecordError::OutOfDomain)?;
        let repeated = (1..members.len()).any(|i| {
            members[..i]
                .iter()
                .any(|(c, j)| *c == members[i].0 || *j == members[i].1)
        });
        if repeated {
            return Err(RecordError::DuplicateMember);
        }
        self.leaves(&shape, members.iter().map(|&(c, _)| c))
            .map_err(RecordError::OutOfDomain)
    }

    /// The pool once a fit of `shape` records `members`, those already in it moved out, within
    /// [`POOL_LIMITS`]'s fits and bytes.
    fn leaves(
        &self,
        shape: &FitShape,
        members: impl Iterator<Item = Cid>,
    ) -> Result<(), PoolLimit> {
        let moved = sorted(members);
        let stays = |c: &Cid| moved.binary_search(&c.0).is_err();
        let mut fits = usize::from(shape.members > 0);
        let mut bytes = match shape.members {
            0 => Some(u128::from(POOL_FIXED_BYTES)),
            _ => shape
                .bytes()
                .and_then(|b| b.checked_add(u128::from(POOL_FIXED_BYTES))),
        };
        for fit in &self.fits {
            let left = fit.members.iter().filter(|(c, _)| stays(c)).count();
            if left > 0 {
                fits += 1;
                let kept = FitShape {
                    members: left,
                    ..fit.shape()
                };
                bytes = bytes.zip(kept.bytes()).and_then(|(t, b)| t.checked_add(b));
            }
        }
        if fits > POOL_LIMITS.fits {
            return Err(PoolLimit::Fits);
        }
        within_bytes(bytes)
    }

    /// The pool's estimated bytes, as [`PoolLimits::bytes`] bounds them.
    pub fn bytes(&self) -> Option<u128> {
        self.fits
            .iter()
            .try_fold(u128::from(POOL_FIXED_BYTES), |total, f| {
                total.checked_add(f.shape().bytes()?)
            })
    }

    /// Takes `item` out of the pool (to the active pool, or retired); false if absent.
    pub fn remove(&mut self, item: &Cid) -> bool {
        let before = self.len();
        self.take_out(std::iter::once(*item));
        self.canonicalize();
        self.len() != before
    }

    /// Takes `items` out of their fits, releasing the spare capacity the estimate does not count.
    fn take_out(&mut self, items: impl Iterator<Item = Cid>) {
        let gone = sorted(items);
        for fit in &mut self.fits {
            let before = fit.members.len();
            fit.members
                .retain(|(c, _)| gone.binary_search(&c.0).is_err());
            if fit.members.len() != before {
                fit.members.shrink_to_fit();
            }
        }
        self.fits.retain(|f| !f.members.is_empty());
    }

    fn canonicalize(&mut self) {
        for fit in &mut self.fits {
            fit.members.sort_unstable_by_key(|(c, _)| c.0);
        }
        self.fits
            .sort_unstable_by_key(|f| f.members.first().map(|(c, _)| c.0));
    }

    /// The admission cost `D(T)` of `selection`, rounded up to `2⁻³²` points; `None` if an item is
    /// not in the pool or listed twice.
    pub fn dtf(&self, selection: &[Cid]) -> Option<f64> {
        let distinct = (1..selection.len()).all(|i| !selection[..i].contains(&selection[i]));
        let mut found = 0;
        let mut total: u64 = 0;
        for fit in &self.fits {
            let set: Vec<usize> = fit
                .members
                .iter()
                .filter(|(c, _)| selection.contains(c))
                .map(|&(_, j)| j)
                .collect();
            found += set.len();
            total = total.checked_add(units(fit.curves.dtf(&set)?))?;
        }
        (distinct && found == selection.len()).then(|| total as f64 / SCALE)
    }

    /// The counts of drawing `n` from this pool, or why that is outside [`DRAW_LIMITS`]: a
    /// function of the fits' shapes and `n` alone (`docs/15` D3).
    pub fn admission(&self, n: usize) -> Result<DrawCounts, OutOfDomain> {
        draw_counts(self.fits.iter().map(Fit::shape), n, &DRAW_LIMITS)
    }

    /// Draws `n` contested facts whose cost is at most `tolerance`, uniformly per fit among the
    /// balanced selections (`docs/02` §B.7), from the pool's content and `seed` alone, once
    /// [`ContestedPool::admission`] admits `n`. A negative or NaN `tolerance` admits only 0.
    pub fn draw(&self, n: usize, tolerance: f64, seed: u64) -> Result<Vec<Cid>, DrawError> {
        self.admission(n).map_err(DrawError::OutOfDomain)?;
        let budget = (tolerance * SCALE).floor() as u64;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut order: Vec<usize> = (0..self.fits.len()).collect();
        order.shuffle(&mut rng);
        let options: Vec<Vec<Candidate>> = order
            .iter()
            .map(|&f| self.candidates(f, n, budget))
            .collect();

        // `least[i][s]`: the least cost that takes exactly `s` facts from the fits `order[i..]`.
        let m = order.len();
        let mut least = vec![vec![None; n + 1]; m + 1];
        least[m][0] = Some(0u64);
        for i in (0..m).rev() {
            for s in 0..=n {
                least[i][s] = options[i]
                    .iter()
                    .filter(|c| c.members.len() <= s)
                    .filter_map(|c| least[i + 1][s - c.members.len()]?.checked_add(c.units))
                    .min();
            }
        }
        let within = |bound: Option<u64>| bound.is_some_and(|b| b <= budget);
        if !within(least[0][n]) {
            return Err(DrawError::NoBalancedDraw(NoBalancedDraw {
                requested: n,
                feasible: (0..=n).filter(|&s| within(least[0][s])).collect(),
            }));
        }

        let (mut left, mut room) = (n, budget);
        let mut drawn = Vec::with_capacity(n);
        for (i, &f) in order.iter().enumerate() {
            let open: Vec<&Candidate> = options[i]
                .iter()
                .filter(|c| c.members.len() <= left)
                .filter(|c| {
                    least[i + 1][left - c.members.len()]
                        .and_then(|rest| rest.checked_add(c.units))
                        .is_some_and(|b| b <= room)
                })
                .collect();
            let pick = open[rng.gen_range(0..open.len())];
            room -= pick.units;
            left -= pick.members.len();
            drawn.extend(pick.members.iter().map(|&k| self.fits[f].members[k].0));
        }
        drawn.shuffle(&mut rng);
        Ok(drawn)
    }

    /// [`ContestedPool::draw`] seeded from the epoch's beacon (INV-10), keyed on the test's
    /// index, never on its content; the same domain.
    pub fn draw_from_beacon(
        &self,
        n: usize,
        tolerance: f64,
        beacon: &Beacon,
        test: u64,
    ) -> Result<Vec<Cid>, DrawError> {
        self.draw(n, tolerance, beacon.seed(CONTESTED, test))
    }

    /// Fit `f`'s subsets of at most `n` members with a DTF within `budget`, the empty one first.
    fn candidates(&self, f: usize, n: usize, budget: u64) -> Vec<Candidate> {
        let fit = &self.fits[f];
        let mut out = Vec::new();
        let mut stack: Vec<Vec<usize>> = vec![Vec::new()];
        while let Some(members) = stack.pop() {
            if members.len() < n {
                let next = members.last().map_or(0, |&k| k + 1);
                for k in (next..fit.members.len()).rev() {
                    let mut grown = members.clone();
                    grown.push(k);
                    stack.push(grown);
                }
            }
            let set: Vec<usize> = members.iter().map(|&k| fit.members[k].1).collect();
            if let Some(cost) = fit.curves.dtf(&set).map(units).filter(|&u| u <= budget) {
                out.push(Candidate {
                    members,
                    units: cost,
                });
            }
        }
        out
    }
}
