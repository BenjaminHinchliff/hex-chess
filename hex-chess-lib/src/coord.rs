use std::{
    fmt,
    ops::{Add, Div, Mul, Rem, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord {
    pub q: i32,
    pub r: i32,
}

impl Coord {
    pub const ZERO: Coord = Coord::new(0, 0);

    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn s(&self) -> i32 {
        -self.q - self.r
    }

    /// gives the hexagonal manhattan distance
    /// for euclidean length see `norm`
    pub fn length(&self) -> i32 {
        self.q.abs().max(self.r.abs().max(self.s().abs()))
    }

    /// gives the square of the euclidean norm
    pub fn norm_squared(&self) -> i32 {
        let Self { q, r } = self;
        q * q + r * r + q * r
    }

    pub fn reflect_q(self) -> Self {
        Self::new(self.q, self.s())
    }

    pub fn is_axis(&self) -> bool {
        (self.q == 0 && self.r != 0 && self.s() != 0)
            || (self.r == 0 && self.q != 0 && self.s() != 0)
            || (self.s() == 0 && self.q != 0 && self.r != 0)
    }
}

impl From<(i32, i32)> for Coord {
    fn from((q, r): (i32, i32)) -> Self {
        Self::new(q, r)
    }
}

impl Add for Coord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.q + rhs.q, self.r + rhs.r)
    }
}

impl Sub for Coord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.q - rhs.q, self.r - rhs.r)
    }
}

impl Div for Coord {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.q / rhs.q, self.r / rhs.r)
    }
}

impl Div<i32> for Coord {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.q / rhs, self.r / rhs)
    }
}

impl Mul<i32> for Coord {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.q * rhs, self.r * rhs)
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.q, self.r, self.s())
    }
}
