//! Question lifecycle orchestration (`docs/05-question-lifecycle.md`): ties together
//! `scoring` (the two filters), `identity` (role pseudonyms, rate limits) and `network`
//! (the tamper-evident log); the deterministic pieces are seeded for reproducibility.

pub mod admission;
pub mod appeal;
pub mod blueprint;
pub mod cohort_scores;
pub mod cohort_snapshot;
pub mod contested;
pub mod deposit;
pub mod events;
pub mod exploration;
pub mod exposure;
pub mod gate;
pub mod governance;
pub mod honeypot;
pub mod ledger;
pub mod lifecycle;
pub mod lottery;
pub mod node;
pub mod orchestrator;
pub mod panel_scores;
pub mod pilot;
pub mod probation;
pub mod randomness;
pub mod results;
pub mod revalidation;
pub mod review;
