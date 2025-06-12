use std::ops::{Add, Index, Neg, Sub};

use rand::distr::{Distribution, StandardUniform};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Coord<const N: usize> {
    inner: usize,
}

#[derive(Clone, Debug)]
pub struct PatternBlueprint<const N: usize> {
    inner: [[bool; N]; N],
}

#[derive(Clone, Debug)]
pub struct Pattern<const N: usize> {
    inner: [[bool; N]; N],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Transform<const N: usize> {
    pub origin: (Coord<N>, Coord<N>),
    pub transpose: bool,
    pub mirror: bool,
}

impl<const N: usize> Coord<N> {
    pub const ZERO: Self = Self { inner: 0 };
    pub const ONE: Self = Self { inner: 1 };

    pub fn new(n: usize) -> Self {
        Self { inner: n % N }
    }

    pub fn inner(self) -> usize {
        self.inner
    }

    pub fn iter_all() -> impl Iterator<Item = Self> {
        (0..N).map(Self::new)
    }
}

impl<const N: usize> Distribution<Coord<N>> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Coord<N> {
        Coord::new(rng.random_range(0..N))
    }
}

impl<const N: usize> Add for Coord<N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.inner + rhs.inner)
    }
}

impl<const N: usize> Neg for Coord<N> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(N - self.inner)
    }
}

impl<const N: usize> Sub for Coord<N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl<const N: usize> PatternBlueprint<N> {
    pub fn new(inner: [[bool; N]; N]) -> Self {
        Self { inner }
    }

    pub fn construct(&self, transform: Transform<N>) -> Pattern<N> {
        let Transform {
            origin,
            transpose,
            mirror,
        } = transform;
        let mut inner = [[false; N]; N];
        for o in Coord::iter_all() {
            for i in Coord::iter_all() {
                let (origin_i, origin_o) = origin;
                inner[(origin_o + o).inner()][(origin_i + i).inner()] =
                    self[match (transpose, mirror) {
                        (false, false) => (i, o),
                        (true, false) => (o, i),
                        (false, true) => (-i, -o),
                        (true, true) => (-o, -i),
                    }];
            }
        }
        Pattern { inner }
    }
}

impl<const N: usize> Index<(Coord<N>, Coord<N>)> for PatternBlueprint<N> {
    type Output = bool;

    fn index(&self, (i, o): (Coord<N>, Coord<N>)) -> &Self::Output {
        &self.inner[o.inner()][i.inner()]
    }
}

impl<const N: usize> Pattern<N> {
    pub fn new(inner: [[bool; N]; N]) -> Self {
        Self { inner }
    }

    pub fn empty() -> Self {
        Self {
            inner: [[false; N]; N],
        }
    }
}

impl<const N: usize> Index<(Coord<N>, Coord<N>)> for Pattern<N> {
    type Output = bool;

    fn index(&self, (i, o): (Coord<N>, Coord<N>)) -> &Self::Output {
        &self.inner[o.inner()][i.inner()]
    }
}
