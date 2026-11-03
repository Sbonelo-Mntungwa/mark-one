//! Mask ROM: 16 bytes. [A0..A3] -> [D0..D7]. Contents are set per part,
//! so the ROM is placed directly rather than built.

use super::{from_num, three_valued, to_num, Model};
use crate::signal::Sig;

pub const INPUTS: [&str; 4] = ["A0", "A1", "A2", "A3"];
pub const OUTPUTS: [&str; 8] = ["D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7"];

pub struct Rom {
    data: [u8; 16],
}

impl Rom {
    pub fn new(d: &[u8]) -> Rom {
        let mut data = [0u8; 16];
        for (i, b) in d.iter().take(16).enumerate() {
            data[i] = *b;
        }
        Rom { data }
    }
}

impl Model for Rom {
    fn n_in(&self) -> usize {
        4
    }
    fn n_out(&self) -> usize {
        8
    }
    fn eval(&mut self, ins: &[Sig]) -> Vec<Sig> {
        let data = self.data;
        three_valued(ins, 8, &|b| from_num(data[to_num(b) as usize] as u32, 8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{bools, sigs};

    #[test]
    fn returns_the_byte_at_each_address() {
        let bytes: Vec<u8> = (0..16).map(|i| i * 17).collect();
        let mut r = Rom::new(&bytes);
        for a in 0..16u32 {
            let d = bools(&r.eval(&sigs(&from_num(a, 4)))).unwrap();
            assert_eq!(to_num(&d) as u8, bytes[a as usize]);
        }
    }

    #[test]
    fn short_images_are_padded_with_zero() {
        let mut r = Rom::new(&[0xAB]);
        assert_eq!(
            bools(&r.eval(&sigs(&from_num(5, 4)))).map(|d| to_num(&d)),
            Some(0)
        );
    }
}
