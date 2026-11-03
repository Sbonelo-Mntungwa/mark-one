//! Disjoint sets over net indices, with path halving.

pub(crate) struct Uf(Vec<usize>);

impl Uf {
    pub(crate) fn new(n: usize) -> Self {
        Uf((0..n).collect())
    }
    pub(crate) fn find(&mut self, mut a: usize) -> usize {
        while self.0[a] != a {
            self.0[a] = self.0[self.0[a]];
            a = self.0[a];
        }
        a
    }
    pub(crate) fn union(&mut self, a: usize, b: usize) {
        let (a, b) = (self.find(a), self.find(b));
        if a != b {
            self.0[a] = b;
        }
    }
}
