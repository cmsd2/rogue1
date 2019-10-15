use std::ops;
use std::cmp;
use std::f32;
use std::f64;

pub trait MinMax {
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
}

impl MinMax for i32 {
    fn min(self, other: Self) -> Self {
        cmp::min(self, other)
    }

    fn max(self, other: Self) -> Self {
        cmp::max(self, other)
    }
}

impl MinMax for f32 {
    fn min(self, other: Self) -> Self {
        f32::min(self, other)
    }

    fn max(self, other: Self) -> Self {
        f32::max(self, other)
    }
}

impl MinMax for f64 {
    fn min(self, other: Self) -> Self {
        f64::min(self, other)
    }

    fn max(self, other: Self) -> Self {
        f64::max(self, other)
    }
}

pub trait TweenableValue: Sized + Copy + PartialEq + PartialOrd + MinMax + ops::Add<Output=Self> + ops::Sub<Output=Self> + ops::Mul<Output=Self> + ops::Div<Output=Self> {
    fn signum(self) -> Self;
    fn abs(self) -> Self;
}

impl TweenableValue for f32 {
    fn signum(self) -> Self {
        self.signum()
    }

    fn abs(self) -> Self {
        self.abs()
    }
}

impl TweenableValue for f64 {
    fn signum(self) -> Self {
        self.signum()
    }

    fn abs(self) -> Self {
        self.abs()
    }
}

impl TweenableValue for i32 {
    fn signum(self) -> Self {
        self.signum()
    }

    fn abs(self) -> Self {
        self.abs()
    }
}

pub trait Tweenable<T> where T: TweenableValue {
    fn tween(&mut self, t: &Tweener<T>);
}

impl <T> Tweenable<T> for T where T: TweenableValue {
    fn tween(&mut self, t: &Tweener<T>) {
        *self = t.apply(*self);
    }
}

#[derive(Debug, Clone)]
pub enum Tweener<T> where T: TweenableValue {
    Increment {
        to: T,
        step: T,
    },
    Decrement {
        to: T,
        step: T,
    },
    Decay {
        to: T,
        factor_step: T,
        min_step: T,
    }
}

impl <T> Tweener<T> where T: TweenableValue {
    pub fn apply(&self, value: T) -> T {
        match self {
            Tweener::Increment { to, step } => {
                if value < (*to - *step) {
                    value + *step
                } else {
                    *to
                }
            },
            Tweener::Decrement { to, step } => {
                if value > (*to + *step) {
                    value - *step
                } else {
                    *to
                }
            },
            Tweener::Decay { to, factor_step, min_step } => {
                let gap = *to - value;
                let signum = gap.signum();
                let gap = gap.abs();
                let step = gap.min(min_step.max(gap * *factor_step)) * signum;
                value + step
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_decay() {
        let t = Tweener::Decay { to: 100.0, factor_step: 0.2, min_step: 1.0 };
        assert_eq!(t.apply(50.0) as i32, 60);
        assert_eq!(t.apply(98.0) as i32, 99);
        assert_eq!(t.apply(99.0) as i32, 100);
        assert_eq!(t.apply(100.0) as i32, 100);
        assert_eq!(t.apply(101.0) as i32, 100);
        assert_eq!(t.apply(150.0) as i32, 140);

        let mut f: f32 = 50.0;
        f.tween(&t);
        assert_eq!(f as i32, 60);
    }
}