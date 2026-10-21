//! Author ages from applied closing cuts (`docs/15` C6).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthorConfig {
    half_life_closings: u64,
}

impl Default for AuthorConfig {
    fn default() -> Self {
        Self {
            half_life_closings: 45,
        }
    }
}

impl AuthorConfig {
    pub fn new(half_life_closings: u64) -> Option<Self> {
        (half_life_closings > 0).then_some(Self { half_life_closings })
    }

    pub fn half_life_closings(self) -> u64 {
        self.half_life_closings
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthorPoint {
    pub epoch: u64,
    pub closings: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CutContext {
    pub number: u64,
    pub epoch: u64,
    pub closes: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextError {
    AlreadyOpen,
    NotOpen,
    Number,
    Epoch,
    Overflow,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AuthorClock {
    last: Option<CutContext>,
    active: Option<CutContext>,
    closings: u64,
}

impl AuthorClock {
    pub fn begin(&mut self, cut: CutContext) -> Result<(), ContextError> {
        if self.active.is_some() {
            return Err(ContextError::AlreadyOpen);
        }
        let number = match self.last {
            Some(last) => last.number.checked_add(1).ok_or(ContextError::Overflow)?,
            None => 0,
        };
        if cut.number != number {
            return Err(ContextError::Number);
        }
        if self
            .last
            .is_some_and(|last| cut.epoch < last.epoch || (last.closes && cut.epoch == last.epoch))
        {
            return Err(ContextError::Epoch);
        }
        if cut.closes && self.closings == u64::MAX {
            return Err(ContextError::Overflow);
        }
        self.active = Some(cut);
        Ok(())
    }

    pub fn end(&mut self) -> Result<(), ContextError> {
        let cut = self.active.take().ok_or(ContextError::NotOpen)?;
        self.closings += u64::from(cut.closes);
        self.last = Some(cut);
        Ok(())
    }

    pub fn point(&self) -> Option<AuthorPoint> {
        self.active.map(|cut| AuthorPoint {
            epoch: cut.epoch,
            closings: self.closings,
        })
    }

    pub fn closings(&self) -> u64 {
        self.closings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // C6: closing overflow is refused before any event in that cut can apply.
    #[test]
    fn closing_overflow_is_prevalidated() {
        let mut clock = AuthorClock {
            closings: u64::MAX,
            ..AuthorClock::default()
        };
        let before = clock.clone();
        let cut = CutContext {
            number: 0,
            epoch: 0,
            closes: true,
        };
        assert_eq!(clock.begin(cut), Err(ContextError::Overflow));
        assert_eq!(clock, before);
        clock
            .begin(CutContext {
                closes: false,
                ..cut
            })
            .unwrap();
        clock.end().unwrap();
        assert_eq!(clock.closings(), u64::MAX);
    }

    // C6: no cut follows u64::MAX; number overflow cannot wrap to zero.
    #[test]
    fn cut_number_overflow_is_prevalidated() {
        let mut clock = AuthorClock {
            last: Some(CutContext {
                number: u64::MAX,
                epoch: 0,
                closes: false,
            }),
            ..AuthorClock::default()
        };
        let before = clock.clone();
        assert_eq!(
            clock.begin(CutContext {
                number: 0,
                epoch: 1,
                closes: false
            }),
            Err(ContextError::Overflow)
        );
        assert_eq!(clock, before);
    }
}
