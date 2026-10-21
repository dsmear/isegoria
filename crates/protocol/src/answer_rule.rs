//! The answer review's outcome (`docs/01` D49, `docs/05` §The answer review): a count of
//! equally weighted answers against the key, compared in integers.

use crate::answer::Answer;

/// The drawn group's size, the quorum of usable answers and the threshold `num/den`; provisional
/// (`docs/14`, D52).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConsensusRule {
    group: u32,
    quorum: u32,
    num: u32,
    den: u32,
}

impl Default for ConsensusRule {
    fn default() -> Self {
        ConsensusRule {
            group: 12,
            quorum: 9,
            num: 3,
            den: 4,
        }
    }
}

/// A round's outcome (D49).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    InsufficientParticipation,
    Accepted,
    Discarded,
    Divided,
}

impl ConsensusRule {
    /// `None` unless `1 ≤ quorum ≤ group` and `1/2 < num/den ≤ 1`: above one half, accepting
    /// and discarding cannot both hold.
    pub fn new(group: u32, quorum: u32, num: u32, den: u32) -> Option<Self> {
        let valid =
            quorum >= 1 && quorum <= group && num <= den && 2 * u64::from(num) > u64::from(den);
        valid.then_some(ConsensusRule {
            group,
            quorum,
            num,
            den,
        })
    }

    pub fn group(self) -> u32 {
        self.group
    }

    pub fn quorum(self) -> u32 {
        self.quorum
    }

    /// The threshold as `(num, den)`.
    pub fn threshold(self) -> (u32, u32) {
        (self.num, self.den)
    }

    /// The outcome of `usable` answers of which `coinciding` give the key; `coinciding ≤ usable`.
    pub fn verdict(self, usable: u32, coinciding: u32) -> Verdict {
        assert!(coinciding <= usable);
        let (v, c) = (u64::from(usable), u64::from(coinciding));
        let (num, den) = (u64::from(self.num), u64::from(self.den));
        if usable < self.quorum {
            Verdict::InsufficientParticipation
        } else if den * c >= num * v {
            Verdict::Accepted
        } else if den * (v - c) >= num * v {
            Verdict::Discarded
        } else {
            Verdict::Divided
        }
    }
}

/// A closed round's full distribution: each alternative's count, "none", the abstentions and
/// the absences (drawn reviewers with no revealed answer).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub alternatives: Vec<u32>,
    pub none: u32,
    pub abstentions: u32,
    pub absences: u32,
}

impl Tally {
    /// The tally of `answers`, each within `alternatives` options; `absences` silent reviewers.
    pub fn of(
        alternatives: usize,
        answers: impl IntoIterator<Item = Answer>,
        absences: u32,
    ) -> Self {
        let mut tally = Tally {
            alternatives: vec![0; alternatives],
            absences,
            ..Tally::default()
        };
        for answer in answers {
            match answer {
                Answer::Alternative(i) => tally.alternatives[usize::from(i)] += 1,
                Answer::NoneCorrect => tally.none += 1,
                Answer::DontKnow => tally.abstentions += 1,
            }
        }
        tally
    }

    /// `V`: the alternatives chosen and "none".
    pub fn usable(&self) -> u32 {
        self.alternatives.iter().sum::<u32>() + self.none
    }

    /// `C`: the answers that give `key`.
    pub fn coinciding(&self, key: u8) -> u32 {
        self.alternatives[usize::from(key)]
    }

    pub fn verdict(&self, key: u8, rule: ConsensusRule) -> Verdict {
        rule.verdict(self.usable(), self.coinciding(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Verdict::*;

    /// The lowest `C` accepted and the highest discarded at `V`, worked by hand from D49.
    const BOUNDS: [(u32, u32, u32); 4] = [(9, 7, 2), (10, 8, 2), (11, 9, 2), (12, 9, 3)];

    // AT-ANS-01: every (V, C) up to 12 against the hand-worked boundaries; below 9 insufficient.
    #[test]
    fn the_thresholds_boundaries() {
        let rule = ConsensusRule::default();
        for v in 0..9 {
            for c in 0..=v {
                assert_eq!(rule.verdict(v, c), InsufficientParticipation, "V={v} C={c}");
            }
        }
        for (v, accept_from, discard_to) in BOUNDS {
            for c in 0..=v {
                let expected = if c >= accept_from {
                    Accepted
                } else if c <= discard_to {
                    Discarded
                } else {
                    Divided
                };
                assert_eq!(rule.verdict(v, c), expected, "V={v} C={c}");
            }
        }
    }

    // AT-ANS-01: "none" counts against the key, and the dissent need not agree.
    #[test]
    fn none_and_scattered_dissent_count_against_the_key() {
        let rule = ConsensusRule::default();
        let scattered = Tally {
            alternatives: vec![3, 3, 3],
            none: 3,
            ..Tally::default()
        };
        assert_eq!(scattered.verdict(0, rule), Discarded);
        let none = Tally {
            alternatives: vec![3, 0],
            none: 9,
            ..Tally::default()
        };
        assert_eq!(none.verdict(0, rule), Discarded);
        let split = Tally {
            alternatives: vec![8, 0],
            none: 4,
            ..Tally::default()
        };
        assert_eq!(split.verdict(0, rule), Divided);
    }

    // AT-ANS-02: abstentions and absences are outside V and the quorum.
    #[test]
    fn abstentions_and_absences_are_outside_the_count() {
        let rule = ConsensusRule::default();
        let answers = |n: usize, a: Answer| std::iter::repeat_n(a, n);
        let nine = Tally::of(
            2,
            answers(9, Answer::Alternative(0)).chain(answers(3, Answer::DontKnow)),
            0,
        );
        assert_eq!((nine.usable(), nine.abstentions), (9, 3));
        assert_eq!(nine.verdict(0, rule), Accepted);
        let eight = Tally::of(
            2,
            answers(8, Answer::Alternative(0)).chain(answers(4, Answer::DontKnow)),
            0,
        );
        assert_eq!(eight.verdict(0, rule), InsufficientParticipation);
        let absent = Tally::of(2, answers(8, Answer::Alternative(0)), 4);
        assert_eq!((absent.usable(), absent.absences), (8, 4));
        assert_eq!(absent.verdict(0, rule), InsufficientParticipation);
        let both = Tally::of(
            2,
            answers(7, Answer::Alternative(1)).chain(answers(2, Answer::Alternative(0))),
            3,
        );
        assert_eq!(both.verdict(0, rule), Discarded);
    }

    // AT-ANS-01: a rule is refused unless its quorum fits the group and its threshold exceeds 1/2.
    #[test]
    fn a_rule_needs_a_fitting_quorum_and_a_threshold_above_one_half() {
        assert_eq!(
            ConsensusRule::new(12, 9, 3, 4),
            Some(ConsensusRule::default())
        );
        assert_eq!(ConsensusRule::new(12, 13, 3, 4), None);
        assert_eq!(ConsensusRule::new(12, 0, 3, 4), None);
        assert_eq!(ConsensusRule::new(12, 9, 1, 2), None);
        assert_eq!(ConsensusRule::new(12, 9, 5, 4), None);
        assert_eq!(ConsensusRule::new(12, 9, 0, 0), None);
        assert!(ConsensusRule::new(12, 12, 1, 1).is_some());
    }
}
