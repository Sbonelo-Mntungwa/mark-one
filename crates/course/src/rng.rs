//! Seeded xorshift generator: diagnostics are random but reproducible.

pub(crate) struct Rng(u64);

impl Rng {
    pub(crate) fn seeded(id: &str) -> Rng {
        let h = id.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
            (h ^ b as u64).wrapping_mul(0x100_0000_01b3)
        });
        Rng(h | 1)
    }
    pub(crate) fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    pub(crate) fn bit(&mut self) -> bool {
        self.next() >> 33 & 1 == 1
    }
    pub(crate) fn below(&mut self, n: u64) -> u64 {
        (self.next() >> 11) % n
    }
}
