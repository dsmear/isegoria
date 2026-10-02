//! Two-stage pilot (`docs/05` [6]/[7], `docs/01` D11): a cheap stage 1 screen, then DIF
//! in batches at stage 2 (`docs/02` §B). `screen`/`dif_batch` enforce the floors (INV-8,
//! PROTO-006); "distinct respondents" counts persons via [`NullifierSet`] (INV-9, T65).

use crate::admission::{admit, DuplicateNullifier, NullifierSet, Unproven};
use crate::lifecycle::K_MIN;
use identity::credential::IssuerPublic;
use identity::nullifier::NullifierProof;
use identity::nym::{Nym, Role};
use network::cid::{cid, Cid};
#[cfg(feature = "calibration")]
use scoring::dif::{logistic_dif, BETA2_MAX};
use scoring::irt::{kr20, point_biserial, A_MIN, B_ABS_MAX, C_EXCESS_MAX, KR20_MIN, R_PBIS_MIN};
use scoring::latent::{latent_dif_with, Format, Formats, LatentParams};
use scoring::Convergence;
#[cfg(feature = "calibration")]
use scoring::LogisticFit;

/// Stage-1 distinct-respondent floor (`docs/02` §B.6: the classic discrimination screen).
pub const N1_MIN: usize = 300;
/// Stage-2 (Variant-1, attribute DIF) respondent floor (`docs/02` §B.6).
pub const N2_MIN: usize = 1500;

/// Why a pilot batch is not admissible (`docs/08` INV-8, §B.6 sample sizes, D37).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PilotError {
    NotEnoughRespondents {
        have: usize,
        need: usize,
    },
    BatchTooSmall {
        items: usize,
    },
    UnreliableAnchors {
        kr20: f64,
        need: f64,
    },
    RowCountMismatch {
        rows: usize,
        respondents: usize,
    },
    /// Not one format per anchor and per item, or a choice among fewer than two options (D25).
    BadFormats,
    /// Not one template entry per anchor and per item (D43).
    BadTemplates,
    /// Two columns of the batch, anchors included, come from `template` (D43, `docs/05` [7]).
    SharedTemplate {
        template: Cid,
    },
}

/// Each column's template (`docs/05` [9]) in column order; `None` for an item from no template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Templates {
    pub anchors: Vec<Option<Cid>>,
    pub items: Vec<Option<Cid>>,
}

impl Templates {
    pub fn none(anchors: usize, items: usize) -> Templates {
        Templates {
            anchors: vec![None; anchors],
            items: vec![None; items],
        }
    }
}

/// One template per batch (`docs/01` D43): one entry per column, no template twice.
pub fn admit_templates(
    templates: &Templates,
    anchors: usize,
    items: usize,
) -> Result<(), PilotError> {
    if templates.anchors.len() != anchors || templates.items.len() != items {
        return Err(PilotError::BadTemplates);
    }
    let mut seen = std::collections::BTreeSet::new();
    for template in templates.anchors.iter().chain(&templates.items).flatten() {
        if !seen.insert(*template) {
            return Err(PilotError::SharedTemplate {
                template: *template,
            });
        }
    }
    Ok(())
}

pub fn batch_id(items: &[Cid]) -> Cid {
    let mut sorted: Vec<[u8; 32]> = items.iter().map(|c| c.0).collect();
    sorted.sort_unstable();
    sorted.dedup();
    let mut buf = Vec::with_capacity(32 + 32 * sorted.len());
    buf.extend_from_slice(b"isegoria/pilot-batch/v1");
    buf.extend_from_slice(&(sorted.len() as u64).to_le_bytes());
    for c in &sorted {
        buf.extend_from_slice(c);
    }
    cid(&buf)
}

/// The action context a `Respond` proof is bound to: this batch and epoch (T65).
pub fn response_context(batch: Cid, epoch: u64) -> Vec<u8> {
    let mut ctx = Vec::with_capacity(40);
    ctx.extend_from_slice(&batch.0);
    ctx.extend_from_slice(&epoch.to_le_bytes());
    ctx
}

/// Why an answer sheet was refused at the identity-gated respondent entry point.
#[derive(Debug, PartialEq, Eq)]
pub enum ResponseRejected {
    Unproven(Unproven),
    Duplicate,
}

impl From<Unproven> for ResponseRejected {
    fn from(u: Unproven) -> Self {
        ResponseRejected::Unproven(u)
    }
}

impl From<DuplicateNullifier> for ResponseRejected {
    fn from(_: DuplicateNullifier) -> Self {
        ResponseRejected::Duplicate
    }
}

/// The identity-gated respondent entry point (`docs/08` §9.1 `Pilot1` row, INV-9, T65):
/// records the proven `Respond` id in `respondents`, rejecting a repeat sheet (AT-PRO-09).
pub fn submit_response(
    proof: &NullifierProof,
    issuer: &IssuerPublic,
    batch: Cid,
    epoch: u64,
    respondents: &mut NullifierSet,
) -> Result<Nym, ResponseRejected> {
    let id = admit(
        proof,
        issuer,
        Role::Respond,
        &response_context(batch, epoch),
    )?;
    respondents.spend(id)?;
    Ok(id)
}

#[cfg(feature = "calibration")]
pub(crate) fn respondent_rows(
    respondents: &NullifierSet,
    theta: &[f64],
    columns: impl IntoIterator<Item = usize>,
) -> Result<(), PilotError> {
    let n = respondents.len();
    let mismatch = |rows: usize| PilotError::RowCountMismatch {
        rows,
        respondents: n,
    };
    if theta.len() != n {
        return Err(mismatch(theta.len()));
    }
    for len in columns {
        if len != n {
            return Err(mismatch(len));
        }
    }
    Ok(())
}

/// INV-8 batch admission: never a single item, and enough respondents (`n_min`).
pub fn admit_dif_batch(
    n_items: usize,
    n_respondents: usize,
    n_min: usize,
) -> Result<(), PilotError> {
    if n_items < K_MIN {
        return Err(PilotError::BatchTooSmall { items: n_items });
    }
    if n_respondents < n_min {
        return Err(PilotError::NotEnoughRespondents {
            have: n_respondents,
            need: n_min,
        });
    }
    Ok(())
}

/// Anchor-reliability precondition of the latent re-check (`docs/01` D37, T53): refused
/// below `KR20_MIN` or when undefined. `anchors` is respondents × anchors, 0/1.
pub fn admit_anchors(anchors: &[Vec<f64>]) -> Result<f64, PilotError> {
    let r = kr20(anchors);
    if r.is_nan() || r < KR20_MIN {
        return Err(PilotError::UnreliableAnchors {
            kr20: r,
            need: KR20_MIN,
        });
    }
    Ok(r)
}

/// Outcome of the attribute-based DIF screen for one item (calibration-only, D20).
#[cfg(feature = "calibration")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DifVerdict {
    Pass,
    Reject,
    Undetermined,
}

/// Stage-1 admission gate: runs [`stage1_screen`] once admitted `respondents` meet the
/// floor `N1_MIN`, with one anchors row and one responses row each, every row of one length.
pub fn screen(
    respondents: &NullifierSet,
    anchors: &[Vec<f64>],
    responses: &[Vec<f64>],
    formats: &Formats,
) -> Result<Vec<Screening>, PilotError> {
    if respondents.len() < N1_MIN {
        return Err(PilotError::NotEnoughRespondents {
            have: respondents.len(),
            need: N1_MIN,
        });
    }
    person_rows(respondents, anchors)?;
    person_rows(respondents, responses)?;
    stage1_screen(anchors, responses, formats)
}

/// One row per admitted respondent, every row of the first row's length.
fn person_rows(respondents: &NullifierSet, rows: &[Vec<f64>]) -> Result<(), PilotError> {
    if rows.len() != respondents.len() {
        return Err(PilotError::RowCountMismatch {
            rows: rows.len(),
            respondents: respondents.len(),
        });
    }
    let len = rows.first().map_or(0, Vec::len);
    match rows.iter().find(|row| row.len() != len) {
        Some(row) => Err(PilotError::RowCountMismatch {
            rows: row.len(),
            respondents: len,
        }),
        None => Ok(()),
    }
}

/// Stage-2 admission gate: runs [`stage2_dif`] on ≥ `K_MIN` items with ≥ `N2_MIN` respondents.
#[cfg(feature = "calibration")]
pub fn dif_batch(
    respondents: &NullifierSet,
    theta: &[f64],
    group: &[f64],
    item_responses: &[Vec<f64>],
) -> Result<Vec<DifVerdict>, PilotError> {
    admit_dif_batch(item_responses.len(), respondents.len(), N2_MIN)?;
    respondent_rows(
        respondents,
        theta,
        std::iter::once(group.len()).chain(item_responses.iter().map(Vec::len)),
    )?;
    Ok(stage2_dif(theta, group, item_responses))
}

/// Stage 1 per trial item (`docs/02` §B.2): `r_pbis` on the anchors' total, and `a`, `b`, `c`
/// of the one-class fit, NaN for an item left out of it.
#[derive(Clone, Debug, PartialEq)]
pub struct Stage1Fit {
    pub status: Convergence,
    pub rpb: Vec<f64>,
    pub a: Vec<f64>,
    pub b: Vec<f64>,
    pub c: Vec<f64>,
}

/// Stage 1 screen (`docs/02` §B.2): [`stage1_verdicts`] of [`stage1_fit`].
pub fn stage1_screen(
    anchors: &[Vec<f64>],
    responses: &[Vec<f64>],
    formats: &Formats,
) -> Result<Vec<Screening>, PilotError> {
    let fit = stage1_fit(anchors, responses, formats)?;
    Ok(stage1_verdicts(&fit, formats))
}

/// The one-class target model, ability held normal, on the anchors and the items whose
/// `r_pbis` reaches `R_PBIS_MIN` (`docs/02` §B.2); [`PilotError::BadFormats`] as the fit's.
pub fn stage1_fit(
    anchors: &[Vec<f64>],
    responses: &[Vec<f64>],
    formats: &Formats,
) -> Result<Stage1Fit, PilotError> {
    let na = anchors.iter().map(Vec::len).min().unwrap_or(0);
    let k = responses.iter().map(Vec::len).min().unwrap_or(0);
    if !formats.describes(na, k) {
        return Err(PilotError::BadFormats);
    }
    let total: Vec<f64> = anchors.iter().map(|row| row.iter().sum()).collect();
    let rpb: Vec<f64> = (0..k)
        .map(|j| {
            let item: Vec<f64> = responses.iter().map(|row| row[j]).collect();
            point_biserial(&item, &total)
        })
        .collect();
    let fitted: Vec<usize> = (0..k).filter(|&j| rpb[j] >= R_PBIS_MIN).collect();
    let x: Vec<Vec<f64>> = responses
        .iter()
        .map(|row| fitted.iter().map(|&j| row[j]).collect())
        .collect();
    let kept_formats = Formats {
        anchors: formats.anchors.clone(),
        items: fitted.iter().map(|&j| formats.items[j]).collect(),
    };
    let params = LatentParams {
        max_classes: 1,
        estimate_shape: false,
        ..LatentParams::default()
    };
    let fit =
        latent_dif_with(anchors, &x, &kept_formats, &params).map_err(|_| PilotError::BadFormats)?;
    let mut out = Stage1Fit {
        status: fit.status,
        rpb,
        a: vec![f64::NAN; k],
        b: vec![f64::NAN; k],
        c: vec![f64::NAN; k],
    };
    for (i, &j) in fitted.iter().enumerate() {
        out.a[j] = fit.item_a[0][i];
        out.b[j] = fit.item_b[0][i];
        out.c[j] = fit.item_c[i];
    }
    Ok(out)
}

/// A trial item's stage-1 reading (`docs/02` §B.2, `docs/15` A11): kept, dropped, or
/// indeterminate because the fit did not converge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screening {
    Pass,
    Fail,
    Indeterminate,
}

/// Per item of a converged fit, `Pass` iff `r_pbis ≥ R_PBIS_MIN`, `a ≥ A_MIN`, `|b| ≤ B_ABS_MAX`
/// and its floor at most `C_EXCESS_MAX` over chance (never an item left out of the fit); every
/// item `Indeterminate` when the fit did not converge.
pub fn stage1_verdicts(fit: &Stage1Fit, formats: &Formats) -> Vec<Screening> {
    if fit.status != Convergence::Converged {
        return vec![Screening::Indeterminate; formats.items.len()];
    }
    formats
        .items
        .iter()
        .enumerate()
        .map(|(j, format)| {
            let ceiling = match *format {
                Format::Open => 0.0,
                Format::Choice(m) => 1.0 / f64::from(m) + C_EXCESS_MAX,
            };
            let kept = fit.rpb[j] >= R_PBIS_MIN
                && fit.a[j] >= A_MIN
                && fit.b[j].abs() <= B_ABS_MAX
                && fit.c[j] <= ceiling;
            if kept {
                Screening::Pass
            } else {
                Screening::Fail
            }
        })
        .collect()
}

/// Stage 2, run on the surviving batch: rejects items with uniform DIF against the axis
/// (Variant 1, calibration-only, D20; production uses `revalidate_pool_latent`, Variant 2).
#[cfg(feature = "calibration")]
pub fn stage2_dif(theta: &[f64], group: &[f64], item_responses: &[Vec<f64>]) -> Vec<DifVerdict> {
    item_responses
        .iter()
        .map(|item| {
            let c = logistic_dif(item, theta, group);
            match c.status {
                LogisticFit::Separated => DifVerdict::Undetermined,
                _ if c.beta2.abs() <= BETA2_MAX => DifVerdict::Pass,
                _ => DifVerdict::Reject,
            }
        })
        .collect()
}
