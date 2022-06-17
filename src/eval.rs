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

// TODO refactor to be an integer
/// The evaluation value of a state.
#[derive(Debug, Clone, Copy)]
pub struct Eval {
    /// Result of the state from the root player perspective.
    pub result: Result,
    /// Number of moves to get to the result.
    pub distance: u8,
}

impl Eval {
    /// The worst evaluation, already lost.
    pub const MIN: Self = Self {
        result: Result::Loss,
        distance: 0,
    };

    /// The best evaluation, already won.
    pub const MAX: Self = Self {
        result: Result::Win,
        distance: 0,
    };

    pub fn new(result: Result, distance: u8) -> Self {
        Self { result, distance }
    }

    /// Creates a new evaluation and reverses the result.
    pub fn rev(self) -> Self {
        Self {
            result: match self.result {
                Result::Loss => Result::Win,
                Result::Win => Result::Loss,
                Result::Draw => Result::Draw,
            },
            distance: self.distance,
        }
    }
}

impl PartialEq for Eval {
    fn eq(&self, other: &Self) -> bool {
        if self.result == other.result {
            match self.result {
                Result::Win | Result::Loss => self.distance == other.distance,
                Result::Draw => true,
            }
        } else {
            false
        }
        //self.result == other.result && self.distance == other.distance
    }
}

impl Eq for Eval {}

impl Ord for Eval {
    // TODO draw scores around zero https://www.chessprogramming.org/Score#cite_note-7
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.result.cmp(&other.result) {
            std::cmp::Ordering::Equal => match self.result {
                Result::Win => self.distance.cmp(&other.distance).reverse(),
                Result::Loss => self.distance.cmp(&other.distance),
                Result::Draw => std::cmp::Ordering::Equal,
                //Result::Loss | Result::Draw => self.distance.cmp(&other.distance),
            },
            result => result,
        }
    }
}

impl PartialOrd for Eval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} in {}", self.result, self.distance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse() {
        assert!(Eval::MAX.rev() == Eval::MIN);
        assert!(Eval::MIN.rev() == Eval::MAX);

        assert!(Eval::new(Result::Loss, 5).rev() == Eval::new(Result::Win, 5));
        assert!(Eval::new(Result::Win, 5).rev() == Eval::new(Result::Loss, 5));
        assert!(Eval::new(Result::Draw, 5).rev() == Eval::new(Result::Draw, 5));
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
        assert!(Eval::new(Result::Draw, 5) == Eval::new(Result::Draw, 3));
        assert!(Eval::new(Result::Draw, 5) == Eval::new(Result::Draw, 7));
        //assert!(Eval::new(Result::Draw, 5) > Eval::new(Result::Draw, 3));
        //assert!(Eval::new(Result::Draw, 5) < Eval::new(Result::Draw, 7));

        assert!(Eval::new(Result::Draw, 5) < Eval::new(Result::Win, 5));
        assert!(Eval::new(Result::Draw, 5) < Eval::new(Result::Win, 3));
        assert!(Eval::new(Result::Draw, 5) < Eval::new(Result::Win, 7));

        assert!(Eval::new(Result::Draw, 5) > Eval::new(Result::Loss, 5));
        assert!(Eval::new(Result::Draw, 5) > Eval::new(Result::Loss, 3));
        assert!(Eval::new(Result::Draw, 5) > Eval::new(Result::Loss, 7));

        assert!(Eval::new(Result::Draw, 41) >= Eval::new(Result::Draw, 41));
        assert!(Eval::new(Result::Draw, 41) <= Eval::new(Result::Draw, 41));
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

    #[test]
    fn eval_display() {
        assert_eq!(format!("{}", Eval::new(Result::Loss, 25)), "loss in 25");
        assert_eq!(format!("{}", Eval::new(Result::Draw, 0)), "draw in 0");
        assert_eq!(format!("{}", Eval::new(Result::Win, 255)), "win in 255");
    }
}
