use crate::board;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Result {
    Loss,
    Draw,
    Win,
}

impl std::fmt::Display for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Result::Loss => "loss",
                Result::Draw => "draw",
                Result::Win => "win",
            }
        )
    }
}

/// The evaluation value of a state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Eval {
    /// The internal representation of the evaluation is as shown in the diagram below.
    /// L = Loss, D = Draw, W = Win, ML = MOVECOUNT_LIMIT
    /// <- MIN i16                                 0                                 MAX i16 ->
    /// <------------------------------------------------------------------------------------->
    ///    |        LOSS        | |              DRAW               | |         WIN        |
    ///    |                    / \                |                / \                    |
    ///  L in 0           L in ML D in ML        D in 0       D in ML W in ML           W in 0
    ///    |                |        |             |            |        |                 |
    ///  -2ML-1           -ML-1     -ML            0            ML      ML+1             2ML+1
    n: i16,
}

impl Eval {
    /// The worst evaluation, already lost.
    pub const MIN: Self = Self::new(-2 * board::Board::MOVECOUNT_LIMIT - 1);

    /// The best evaluation, already won.
    pub const MAX: Self = Self::new(2 * board::Board::MOVECOUNT_LIMIT + 1);

    /// Create an evaluation from the given internal representational value.
    pub const fn new(n: i16) -> Self {
        Self { n }
    }

    /// Get the raw representation of this evaluation.
    pub const fn raw(self) -> i16 {
        self.n
    }

    /// Create an evaluation from the given result and distance.
    pub fn from(result: Result, distance: i16) -> Self {
        debug_assert!(distance <= board::Board::MOVECOUNT_LIMIT);
        debug_assert!(distance >= -board::Board::MOVECOUNT_LIMIT);
        debug_assert!(distance >= 0 || result == Result::Draw);

        Self::new(match result {
            Result::Draw => distance,
            Result::Win => 2 * board::Board::MOVECOUNT_LIMIT + 1 - distance,
            Result::Loss => -2 * board::Board::MOVECOUNT_LIMIT - 1 + distance,
        })
    }

    /// Give the result of the evaluation.
    pub fn result(&self) -> Result {
        if self.n > board::Board::MOVECOUNT_LIMIT {
            Result::Win
        } else if self.n < -board::Board::MOVECOUNT_LIMIT {
            Result::Loss
        } else {
            Result::Draw
        }
    }

    /// Convert the evaluation to a human readable form returning (result, distance).
    /// Be aware that the distance can be negative if the result is a draw.
    pub fn human(&self) -> (Result, i16) {
        if self.n > board::Board::MOVECOUNT_LIMIT {
            (Result::Win, 2 * board::Board::MOVECOUNT_LIMIT + 1 - self.n)
        } else if self.n < -board::Board::MOVECOUNT_LIMIT {
            (Result::Loss, 2 * board::Board::MOVECOUNT_LIMIT + 1 + self.n)
        } else {
            (Result::Draw, self.n)
        }
    }

    /// Convert the distance of this evaluation to be relative from the rootcount to the given movecount.
    pub fn relative(&self, rootcount: i16, movecount: i16) -> Self {
        let diff = movecount - rootcount;
        if self.n > board::Board::MOVECOUNT_LIMIT {
            // win
            Self::new(self.n + diff)
        } else if self.n < -board::Board::MOVECOUNT_LIMIT {
            // loss
            Self::new(self.n - diff)
        } else if self.n >= 0 {
            // positive draw
            Self::new(self.n - diff)
        } else {
            // negative draw
            Self::new(self.n + diff)
        }
    }

    /// Convert the distance of this evaluation to be relative from the given movecount to the rootcount.
    pub fn absolute(&self, rootcount: i16, movecount: i16) -> Self {
        debug_assert!(movecount >= rootcount);
        self.relative(movecount, rootcount)
    }
}

impl std::ops::Neg for Eval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.n)
    }
}

impl std::fmt::Display for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (result, distance) = self.human();
        write!(f, "{} in {}", result, distance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO write tests for absolute() and relative()

    /// Check whether the internal representation maps correctly.
    #[test]
    fn internals() {
        const ML: i16 = board::Board::MOVECOUNT_LIMIT;

        assert_eq!(Eval::from(Result::Loss, 0).n, -2 * ML - 1);
        assert_eq!(Eval::from(Result::Loss, ML).n, -ML - 1);
        assert_eq!(-Eval::from(Result::Draw, ML).n, -ML);
        assert_eq!(-Eval::from(Result::Draw, 0).n, 0);
        assert_eq!(Eval::from(Result::Draw, 0).n, 0);
        assert_eq!(Eval::from(Result::Draw, ML).n, ML);
        assert_eq!(Eval::from(Result::Win, ML).n, ML + 1);
        assert_eq!(Eval::from(Result::Win, 0).n, 2 * ML + 1);
    }

    #[test]
    fn negate() {
        assert_eq!(-Eval::MAX, Eval::MIN);
        assert_eq!(-Eval::MIN, Eval::MAX);

        assert_eq!(-Eval::from(Result::Loss, 5), Eval::from(Result::Win, 5));
        assert_eq!(-Eval::from(Result::Win, 5), Eval::from(Result::Loss, 5));
        assert_ne!(-Eval::from(Result::Draw, 5), Eval::from(Result::Draw, 5));
    }

    #[test]
    fn max_ord() {
        assert!(Eval::MAX > Eval::from(Result::Win, 1));
        assert!(Eval::MAX > Eval::from(Result::Draw, 1));
        assert!(Eval::MAX > -Eval::from(Result::Draw, 1));
        assert!(Eval::MAX > Eval::from(Result::Loss, 1));
    }

    #[test]
    fn min_ord() {
        assert!(Eval::MIN < Eval::from(Result::Loss, 1));
        assert!(Eval::MIN < Eval::from(Result::Draw, 1));
        assert!(Eval::MIN < -Eval::from(Result::Draw, 1));
        assert!(Eval::MIN < Eval::from(Result::Win, 1));
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
        assert!(Eval::from(Result::Win, 5) == Eval::from(Result::Win, 5));
        assert!(Eval::from(Result::Win, 3) > Eval::from(Result::Win, 5));
        assert!(Eval::from(Result::Win, 5) < Eval::from(Result::Win, 3));

        assert!(Eval::from(Result::Win, 5) > Eval::from(Result::Draw, 5));
        assert!(Eval::from(Result::Win, 5) > Eval::from(Result::Draw, 3));
        assert!(Eval::from(Result::Win, 5) > Eval::from(Result::Draw, 7));

        assert!(Eval::from(Result::Win, 5) > Eval::from(Result::Loss, 5));
        assert!(Eval::from(Result::Win, 5) > Eval::from(Result::Loss, 3));
        assert!(Eval::from(Result::Win, 5) > Eval::from(Result::Loss, 7));
    }

    #[test]
    fn eval_ord_loss() {
        assert!(Eval::from(Result::Loss, 5) == Eval::from(Result::Loss, 5));
        assert!(Eval::from(Result::Loss, 5) > Eval::from(Result::Loss, 3));
        assert!(Eval::from(Result::Loss, 5) < Eval::from(Result::Loss, 7));

        assert!(Eval::from(Result::Loss, 5) < Eval::from(Result::Win, 5));
        assert!(Eval::from(Result::Loss, 5) < Eval::from(Result::Win, 3));
        assert!(Eval::from(Result::Loss, 5) < Eval::from(Result::Win, 7));

        assert!(Eval::from(Result::Loss, 5) < Eval::from(Result::Draw, 5));
        assert!(Eval::from(Result::Loss, 5) < Eval::from(Result::Draw, 3));
        assert!(Eval::from(Result::Loss, 5) < Eval::from(Result::Draw, 7));
    }

    #[test]
    fn eval_ord_draw_good() {
        assert!(Eval::from(Result::Draw, 5) == Eval::from(Result::Draw, 5));
        assert!(Eval::from(Result::Draw, 5) > Eval::from(Result::Draw, 3));
        assert!(Eval::from(Result::Draw, 5) < Eval::from(Result::Draw, 7));

        assert!(Eval::from(Result::Draw, 5) < Eval::from(Result::Win, 5));
        assert!(Eval::from(Result::Draw, 5) < Eval::from(Result::Win, 3));
        assert!(Eval::from(Result::Draw, 5) < Eval::from(Result::Win, 7));

        assert!(Eval::from(Result::Draw, 5) > Eval::from(Result::Loss, 5));
        assert!(Eval::from(Result::Draw, 5) > Eval::from(Result::Loss, 3));
        assert!(Eval::from(Result::Draw, 5) > Eval::from(Result::Loss, 7));

        assert!(Eval::from(Result::Draw, 41) >= Eval::from(Result::Draw, 41));
        assert!(Eval::from(Result::Draw, 41) <= Eval::from(Result::Draw, 41));
    }

    #[test]
    fn eval_ord_draw_bad() {
        assert!(-Eval::from(Result::Draw, 5) == -Eval::from(Result::Draw, 5));
        assert!(-Eval::from(Result::Draw, 5) < -Eval::from(Result::Draw, 3));
        assert!(-Eval::from(Result::Draw, 5) > -Eval::from(Result::Draw, 7));

        assert!(-Eval::from(Result::Draw, 5) < Eval::from(Result::Win, 5));
        assert!(-Eval::from(Result::Draw, 5) < Eval::from(Result::Win, 3));
        assert!(-Eval::from(Result::Draw, 5) < Eval::from(Result::Win, 7));

        assert!(-Eval::from(Result::Draw, 5) > Eval::from(Result::Loss, 5));
        assert!(-Eval::from(Result::Draw, 5) > Eval::from(Result::Loss, 3));
        assert!(-Eval::from(Result::Draw, 5) > Eval::from(Result::Loss, 7));

        assert!(-Eval::from(Result::Draw, 41) <= -Eval::from(Result::Draw, 41));
        assert!(-Eval::from(Result::Draw, 41) >= -Eval::from(Result::Draw, 41));
    }

    #[test]
    fn eval_ord_draw_both() {
        assert!(-Eval::from(Result::Draw, 5) < Eval::from(Result::Draw, 5));
        assert!(-Eval::from(Result::Draw, 5) < Eval::from(Result::Draw, 3));
        assert!(-Eval::from(Result::Draw, 5) < Eval::from(Result::Draw, 7));
    }

    #[test]
    fn eval_display() {
        assert_eq!(format!("{}", Eval::from(Result::Loss, 25)), "loss in 25");
        assert_eq!(format!("{}", Eval::from(Result::Draw, 0)), "draw in 0");
        assert_eq!(format!("{}", Eval::new(-5)), "draw in -5");
        assert_eq!(format!("{}", Eval::from(Result::Win, 69)), "win in 69");
    }
}
