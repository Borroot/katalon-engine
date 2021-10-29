use std::cmp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Result {
    Loss,
    Draw,
    Win,
}

/// The evaluation value of a state.
#[derive(Debug, Clone, Copy)]
pub struct Eval {
    /// Result of the state from the root player perspective.
    result: Result,
    /// Number of moves to get to the result.
    distance: u8,
}

impl Eval {
    /// The worst evaluation, already lost.
    pub const MIN: Self = Self {
        result: Result::Loss,
        distance: 0,
    };

    // The best evaluation, already won.
    pub const MAX: Self = Self {
        result: Result::Win,
        distance: 0,
    };

    pub fn new(result: Result, distance: u8) -> Self {
        Self {
            result,
            distance,
        }
    }

    /// Consumes the evaluation and reverses the result.
    pub fn reverse(mut self) -> Self {
        match self.result {
            Result::Loss => self.result = Result::Win,
            Result::Win => self.result = Result::Loss,
            Result::Draw => (),
        }
        self
    }
}

impl PartialEq for Eval {
    fn eq(&self, other: &Self) -> bool {
        self.result == other.result && self.distance == other.distance
    }
}

impl Eq for Eval {}

impl Ord for Eval {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.result.cmp(&other.result) {
            cmp::Ordering::Equal => {
                match self.result {
                    Result::Win => self.distance.cmp(&other.distance).reverse(),
                    _ => self.distance.cmp(&other.distance),
                }
            },
            result => result,
        }
    }
}

impl PartialOrd for Eval {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse() {
        assert!(Eval::MAX.reverse() == Eval::MIN);
        assert!(Eval::MIN.reverse() == Eval::MAX);

        assert!(Eval::new(Result::Loss, 5).reverse() == Eval::new(Result::Win, 5));
        assert!(Eval::new(Result::Win, 5).reverse() == Eval::new(Result::Loss, 5));
        assert!(Eval::new(Result::Draw, 5).reverse() == Eval::new(Result::Draw, 5));
    }

    #[test]
    fn max_ord() {
        assert!(Eval::MAX > Eval::new(Result::Win, 1));
        assert!(Eval::MAX > Eval::new(Result::Draw, 1));
        assert!(Eval::MAX > Eval::new(Result::Loss, 1));
    }

    #[test]
    fn min_ord() {
        assert!(Eval::MIN < Eval::new(Result::Loss, 1));
        assert!(Eval::MIN < Eval::new(Result::Draw, 1));
        assert!(Eval::MIN < Eval::new(Result::Win, 1));
    }

    #[test]
    fn result_ord() {
        assert!(Result::Win > Result::Draw);
        assert!(Result::Win > Result::Loss);

        assert!(Result::Draw < Result::Win);
        assert!(Result::Draw > Result::Loss);

        assert!(Result::Loss < Result::Win);
        assert!(Result::Loss < Result::Draw);
    }

    #[test]
    fn eval_ord_win() {
        assert!(Eval::new(Result::Win, 5) == Eval::new(Result::Win, 5));
        assert!(Eval::new(Result::Win, 3) > Eval::new(Result::Win, 5));
        assert!(Eval::new(Result::Win, 5) < Eval::new(Result::Win, 3));

        assert!(Eval::new(Result::Win, 5) > Eval::new(Result::Draw, 5));
        assert!(Eval::new(Result::Win, 5) > Eval::new(Result::Draw, 3));
        assert!(Eval::new(Result::Win, 5) > Eval::new(Result::Draw, 7));

        assert!(Eval::new(Result::Win, 5) > Eval::new(Result::Loss, 5));
        assert!(Eval::new(Result::Win, 5) > Eval::new(Result::Loss, 3));
        assert!(Eval::new(Result::Win, 5) > Eval::new(Result::Loss, 7));
    }

    #[test]
    fn eval_ord_draw() {
        assert!(Eval::new(Result::Draw, 5) == Eval::new(Result::Draw, 5));
        assert!(Eval::new(Result::Draw, 5) > Eval::new(Result::Draw, 3));
        assert!(Eval::new(Result::Draw, 5) < Eval::new(Result::Draw, 7));

        assert!(Eval::new(Result::Draw, 5) < Eval::new(Result::Win, 5));
        assert!(Eval::new(Result::Draw, 5) < Eval::new(Result::Win, 3));
        assert!(Eval::new(Result::Draw, 5) < Eval::new(Result::Win, 7));

        assert!(Eval::new(Result::Draw, 5) > Eval::new(Result::Loss, 5));
        assert!(Eval::new(Result::Draw, 5) > Eval::new(Result::Loss, 3));
        assert!(Eval::new(Result::Draw, 5) > Eval::new(Result::Loss, 7));
    }

    #[test]
    fn eval_ord_loss() {
        assert!(Eval::new(Result::Loss, 5) == Eval::new(Result::Loss, 5));
        assert!(Eval::new(Result::Loss, 5) > Eval::new(Result::Loss, 3));
        assert!(Eval::new(Result::Loss, 5) < Eval::new(Result::Loss, 7));

        assert!(Eval::new(Result::Loss, 5) < Eval::new(Result::Win, 5));
        assert!(Eval::new(Result::Loss, 5) < Eval::new(Result::Win, 3));
        assert!(Eval::new(Result::Loss, 5) < Eval::new(Result::Win, 7));

        assert!(Eval::new(Result::Loss, 5) < Eval::new(Result::Draw, 5));
        assert!(Eval::new(Result::Loss, 5) < Eval::new(Result::Draw, 3));
        assert!(Eval::new(Result::Loss, 5) < Eval::new(Result::Draw, 7));
    }
}