//! Input sequences diagnostics drive into a circuit.

use mark_one_engine::components::from_num;
use mark_one_engine::sim::CHARGE_SETTLES;

use crate::level::{Level, Test};
use crate::rng::Rng;

pub struct Step {
    pub ins: Vec<bool>,
    pub check: bool,
}

/// Long enough for any stored charge to leak away: a design that only holds
/// state dynamically fails these steps.
const HOLD: usize = CHARGE_SETTLES as usize + 2;

pub fn steps(l: &Level) -> Vec<Step> {
    let n = l.inputs.len();
    let mut rng = Rng::seeded(l.id);
    let chk = |ins: Vec<bool>| Step { ins, check: true };
    let quiet = |ins: Vec<bool>| Step { ins, check: false };
    match &l.test {
        Test::Comb if n <= 12 => (0..1u32 << n).map(|m| chk(from_num(m, n))).collect(),
        Test::Comb => {
            let mut v = vec![chk(vec![false; n]), chk(vec![true; n])];
            for _ in 0..3000 {
                v.push(chk((0..n).map(|_| rng.bit()).collect()));
            }
            v
        }
        Test::Sr => {
            let mut v = vec![quiet(vec![false, true]), quiet(vec![false, false])];
            let opts = [[true, false], [false, true], [false, false]];
            for _ in 0..120 {
                v.push(chk(opts[rng.below(3) as usize].to_vec()));
            }
            // Set, hold; reset, hold.
            for s in [[true, false], [false, true]] {
                v.push(chk(s.to_vec()));
                v.extend((0..HOLD).map(|_| chk(vec![false, false])));
            }
            v
        }
        Test::Clocked { ctrl, init, steps } => {
            let mut v: Vec<Step> = init.iter().map(|i| quiet(i.clone())).collect();
            let mut cur = init.last().cloned().unwrap_or_else(|| vec![false; n]);
            for _ in 0..*steps {
                if rng.below(10) < 4 {
                    cur[*ctrl] = !cur[*ctrl];
                } else {
                    for (i, b) in cur.iter_mut().enumerate() {
                        if i != *ctrl {
                            *b = rng.bit();
                        }
                    }
                }
                v.push(chk(cur.clone()));
            }
            // Hold with the control line low, then high, then low again.
            for s in [false, true, false] {
                if cur[*ctrl] != s {
                    cur[*ctrl] = s;
                    v.push(chk(cur.clone()));
                }
                v.extend((0..HOLD).map(|_| chk(cur.clone())));
            }
            v
        }
        Test::Cpu | Test::Machine | Test::Program(_) | Test::Sandbox => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::level;

    #[test]
    fn small_combinational_marks_are_exhaustive() {
        let l = level("full_adder").unwrap();
        assert_eq!(steps(l).len(), 8);
    }

    #[test]
    fn sequences_are_reproducible() {
        let l = level("dff").unwrap();
        let a: Vec<Vec<bool>> = steps(l).into_iter().map(|s| s.ins).collect();
        let b: Vec<Vec<bool>> = steps(l).into_iter().map(|s| s.ins).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn clocked_steps_never_change_clock_and_data_together() {
        let l = level("reg8").unwrap();
        let all = steps(l);
        for w in all.windows(2) {
            let clk_changed = w[0].ins[9] != w[1].ins[9];
            let data_changed = w[0].ins[..9] != w[1].ins[..9];
            assert!(!(clk_changed && data_changed));
        }
    }
}
