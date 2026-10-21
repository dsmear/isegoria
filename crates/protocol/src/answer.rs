//! The answer review's question and answers (`docs/05` §The answer review, `docs/01` D49): the
//! question with its key, the view without it that reviewers get, and an answer's commitment.

use identity::nym::Nym;
use network::cid::{cid, Cid};
use network::codec::{Reader, Writer};
use sha2::{Digest, Sha256};

pub const MIN_ALTERNATIVES: usize = 2;
pub const MAX_ALTERNATIVES: usize = 256;

/// A question as its author supplies it, immutable once deposited; `key` indexes the
/// alternative the author states as correct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    pub context: Vec<u8>,
    pub stem: Vec<u8>,
    pub alternatives: Vec<Vec<u8>>,
    pub key: u8,
}

/// Why a question is not well formed: its deposit is an invalid request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Malformed {
    EmptyContext,
    EmptyStem,
    TooFewAlternatives,
    TooManyAlternatives,
    EmptyAlternative,
    RepeatedAlternative,
    KeyOutOfRange,
}

/// What a drawn reviewer is shown: the question without its key (D49).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviewerView<'a> {
    pub context: &'a [u8],
    pub stem: &'a [u8],
    pub alternatives: &'a [Vec<u8>],
}

impl Question {
    pub fn check(&self) -> Result<(), Malformed> {
        if self.context.is_empty() {
            return Err(Malformed::EmptyContext);
        }
        if self.stem.is_empty() {
            return Err(Malformed::EmptyStem);
        }
        let n = self.alternatives.len();
        if n < MIN_ALTERNATIVES {
            return Err(Malformed::TooFewAlternatives);
        }
        if n > MAX_ALTERNATIVES {
            return Err(Malformed::TooManyAlternatives);
        }
        if self.alternatives.iter().any(Vec::is_empty) {
            return Err(Malformed::EmptyAlternative);
        }
        if (1..n).any(|i| self.alternatives[..i].contains(&self.alternatives[i])) {
            return Err(Malformed::RepeatedAlternative);
        }
        if usize::from(self.key) >= n {
            return Err(Malformed::KeyOutOfRange);
        }
        Ok(())
    }

    /// The content address of the whole question, key included: a duplicate has the same.
    pub fn content_id(&self) -> Cid {
        let mut w = Writer::new();
        w.fixed(b"isegoria/question/v1");
        self.write(&mut w);
        cid(&w.finish())
    }

    pub fn view(&self) -> ReviewerView<'_> {
        ReviewerView {
            context: &self.context,
            stem: &self.stem,
            alternatives: &self.alternatives,
        }
    }

    pub fn write(&self, w: &mut Writer) {
        w.field(&self.context)
            .field(&self.stem)
            .u64(self.alternatives.len() as u64);
        for alternative in &self.alternatives {
            w.field(alternative);
        }
        w.u8(self.key);
    }

    pub fn read(r: &mut Reader) -> Option<Self> {
        let context = r.field().ok()?.to_vec();
        let stem = r.field().ok()?.to_vec();
        let n = usize::try_from(r.u64().ok()?).ok()?;
        if n > MAX_ALTERNATIVES {
            return None;
        }
        let mut alternatives = Vec::new();
        for _ in 0..n {
            alternatives.push(r.field().ok()?.to_vec());
        }
        Some(Question {
            context,
            stem,
            alternatives,
            key: r.u8().ok()?,
        })
    }
}

/// A reviewer's answer: an alternative by index, "none of the alternatives is correct", or
/// "don't know", an abstention (D49).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Answer {
    Alternative(u8),
    NoneCorrect,
    DontKnow,
}

impl Answer {
    pub fn code(self) -> [u8; 2] {
        match self {
            Answer::Alternative(i) => [0, i],
            Answer::NoneCorrect => [1, 0],
            Answer::DontKnow => [2, 0],
        }
    }

    pub fn from_code(code: [u8; 2]) -> Option<Self> {
        match code {
            [0, i] => Some(Answer::Alternative(i)),
            [1, 0] => Some(Answer::NoneCorrect),
            [2, 0] => Some(Answer::DontKnow),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnswerCommit(pub [u8; 32]);

/// `H(answer, nonce, reviewer, question)`: binds the committer and the question's number, so a
/// commitment copied onto another reviewer or question cannot be opened (INV-12).
pub fn commit_answer(
    answer: Answer,
    nonce: &[u8; 32],
    reviewer: Nym,
    question: u64,
) -> AnswerCommit {
    let mut h = Sha256::new();
    h.update(b"isegoria/answer-commit/v1");
    h.update(answer.code());
    h.update(nonce);
    h.update(reviewer.0);
    h.update(question.to_le_bytes());
    AnswerCommit(h.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn question() -> Question {
        Question {
            context: b"context".to_vec(),
            stem: b"stem".to_vec(),
            alternatives: vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()],
            key: 1,
        }
    }

    // AT-ANS-13: each malformation is named; a well-formed question passes.
    #[test]
    fn malformed_questions_are_named() {
        assert_eq!(question().check(), Ok(()));
        type Break = fn(&mut Question);
        let cases: [(Break, Malformed); 7] = [
            (|q| q.context.clear(), Malformed::EmptyContext),
            (|q| q.stem.clear(), Malformed::EmptyStem),
            (
                |q| q.alternatives.truncate(1),
                Malformed::TooFewAlternatives,
            ),
            (
                |q| q.alternatives = (0..=256u16).map(|i| i.to_le_bytes().to_vec()).collect(),
                Malformed::TooManyAlternatives,
            ),
            (|q| q.alternatives[2].clear(), Malformed::EmptyAlternative),
            (
                |q| q.alternatives[2] = b"a".to_vec(),
                Malformed::RepeatedAlternative,
            ),
            (|q| q.key = 3, Malformed::KeyOutOfRange),
        ];
        for (break_it, expected) in cases {
            let mut q = question();
            break_it(&mut q);
            assert_eq!(q.check(), Err(expected));
        }
        let mut most = question();
        most.alternatives = (0..256u16).map(|i| i.to_le_bytes().to_vec()).collect();
        most.key = 255;
        assert_eq!(most.check(), Ok(()));
    }

    // AT-ANS-12: the reviewers' view is the same whatever the key.
    #[test]
    fn the_view_does_not_depend_on_the_key() {
        let a = question();
        let b = Question {
            key: 2,
            ..question()
        };
        assert_eq!(a.view(), b.view());
        assert_ne!(a.content_id(), b.content_id());
    }

    // AT-ANS-12: a commitment opens only for its answer, nonce, reviewer and question.
    #[test]
    fn a_commitment_binds_answer_nonce_reviewer_and_question() {
        let (n, r) = ([7u8; 32], Nym([1; 32]));
        let c = commit_answer(Answer::Alternative(1), &n, r, 4);
        assert_eq!(c, commit_answer(Answer::Alternative(1), &n, r, 4));
        assert_ne!(c, commit_answer(Answer::Alternative(2), &n, r, 4));
        assert_ne!(c, commit_answer(Answer::NoneCorrect, &n, r, 4));
        assert_ne!(c, commit_answer(Answer::Alternative(1), &[8; 32], r, 4));
        assert_ne!(
            c,
            commit_answer(Answer::Alternative(1), &n, Nym([2; 32]), 4)
        );
        assert_ne!(c, commit_answer(Answer::Alternative(1), &n, r, 5));
        for a in [
            Answer::Alternative(0),
            Answer::Alternative(255),
            Answer::NoneCorrect,
            Answer::DontKnow,
        ] {
            assert_eq!(Answer::from_code(a.code()), Some(a));
        }
        assert_eq!(Answer::from_code([1, 1]), None);
        assert_eq!(Answer::from_code([3, 0]), None);
    }

    // AT-ANS-13: a question's encoding round-trips, and a count past the bound is refused.
    #[test]
    fn a_question_round_trips() {
        let q = question();
        let mut w = Writer::new();
        q.write(&mut w);
        let bytes = w.finish();
        let mut r = Reader::new(&bytes);
        assert_eq!(Question::read(&mut r), Some(q));
        assert!(r.finish().is_ok());
        let mut w = Writer::new();
        w.field(b"c").field(b"s").u64(MAX_ALTERNATIVES as u64 + 1);
        assert_eq!(Question::read(&mut Reader::new(&w.finish())), None);
    }
}
