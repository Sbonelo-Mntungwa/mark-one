//! Reference solutions for every level. Each must pass its test bench at
//! exactly par, which proves the progression is solvable with the parts
//! that level unlocks.

use std::collections::BTreeMap;

use mark_one_course::{self as game, Ctx};
use mark_one_engine::circuit::*;

type P = (String, usize);

struct B {
    c: Circuit,
    n: usize,
}

impl B {
    fn new(level: &str) -> B {
        let l = game::level(level).unwrap();
        let mut b = B {
            c: Circuit::default(),
            n: 0,
        };
        for i in 0..l.inputs.len() {
            b.io(format!("in{i}"), PartKind::In, i);
        }
        for i in 0..l.outputs.len() {
            b.io(format!("out{i}"), PartKind::Out, i);
        }
        b
    }
    fn io(&mut self, id: String, kind: PartKind, pin: usize) {
        self.c.parts.push(Part {
            id,
            kind,
            chip: None,
            pin: Some(pin),
            data: None,
            src: None,
            rot: 0,
            label: None,
            x: 0.0,
            y: 0.0,
        });
    }
    fn add(&mut self, kind: PartKind, chip: Option<&str>) -> String {
        self.n += 1;
        let id = format!("p{}", self.n);
        self.c.parts.push(Part {
            id: id.clone(),
            kind,
            chip: chip.map(String::from),
            pin: None,
            data: None,
            src: None,
            rot: 0,
            label: None,
            x: 0.0,
            y: 0.0,
        });
        id
    }
    fn chip(&mut self, id: &str) -> String {
        self.add(PartKind::Chip, Some(id))
    }
    fn w(&mut self, a: P, b: P) {
        self.c.wires.push(Wire {
            from: End {
                part: a.0,
                pin: a.1,
            },
            to: End {
                part: b.0,
                pin: b.1,
            },
        });
    }
    fn g1(&mut self, chip: &str, a: P) -> P {
        let g = self.chip(chip);
        self.w(a, p(&g, 0));
        p(&g, 1)
    }
    fn g2(&mut self, chip: &str, a: P, b: P) -> P {
        let g = self.chip(chip);
        self.w(a, p(&g, 0));
        self.w(b, p(&g, 1));
        p(&g, 2)
    }
}

fn i(k: usize) -> P {
    (format!("in{k}"), 0)
}
fn o(k: usize) -> P {
    (format!("out{k}"), 0)
}
fn p(id: &str, k: usize) -> P {
    (id.to_string(), k)
}

fn run(level: &str, b: &B) -> game::VerifyResult {
    let done = BTreeMap::new();
    let ctx = Ctx {
        done: &done,
        unlock_all: true,
    };
    game::verify(game::index_of(level).unwrap(), &b.c, &ctx)
}

/// The engine's delay table must match the critical path of the reference
/// design, which is what each delay is defined to be.
fn check_delay(level: &str, b: &B) {
    if let Some(want) = mark_one_engine::timing::delay(level) {
        let lib = mark_one_engine::workbench::Library::open();
        let got = mark_one_engine::analysis::critical_path(&b.c, &lib)
            .unwrap()
            .depth
            .unwrap_or(0);
        println!("DELAY {level} {got}");
        if std::env::var("M1_DELAY_SURVEY").is_err() {
            assert_eq!(
                got, want,
                "{level}: timing::delay vs the reference design's critical path"
            );
        }
    }
}

fn check(level: &str, b: B) {
    check_delay(level, &b);
    let r = run(level, &b);
    assert!(r.pass, "{level} failed: {r:?}");
    assert_eq!(
        r.cost,
        game::level(level).unwrap().par,
        "{level} cost vs par"
    );
}

fn not_gate(b: &mut B) {
    let (v, g) = (b.add(PartKind::Vcc, None), b.add(PartKind::Gnd, None));
    let (pm, nm) = (b.add(PartKind::Pmos, None), b.add(PartKind::Nmos, None));
    b.w(i(0), p(&pm, 0));
    b.w(i(0), p(&nm, 0));
    b.w(p(&v, 0), p(&pm, 1));
    b.w(p(&pm, 2), o(0));
    b.w(o(0), p(&nm, 1));
    b.w(p(&nm, 2), p(&g, 0));
}

#[test]
fn every_level_has_a_solution_at_par() {
    // Charge
    let mut b = B::new("power");
    let (v, g) = (b.add(PartKind::Vcc, None), b.add(PartKind::Gnd, None));
    b.w(p(&v, 0), o(0));
    b.w(p(&g, 0), o(1));
    check("power", b);

    let mut b = B::new("relay");
    b.w(i(0), o(0));
    check("relay", b);

    // Transistors
    let mut b = B::new("not");
    not_gate(&mut b);
    check("not", b);

    let mut b = B::new("nand");
    let (v, g) = (b.add(PartKind::Vcc, None), b.add(PartKind::Gnd, None));
    let (p1, p2) = (b.add(PartKind::Pmos, None), b.add(PartKind::Pmos, None));
    let (n1, n2) = (b.add(PartKind::Nmos, None), b.add(PartKind::Nmos, None));
    for (x, gate) in [(&p1, 0), (&n1, 0), (&p2, 1), (&n2, 1)] {
        b.w(i(gate), p(x, 0));
    }
    b.w(p(&v, 0), p(&p1, 1));
    b.w(p(&v, 0), p(&p2, 1));
    b.w(p(&p1, 2), o(0));
    b.w(p(&p2, 2), o(0));
    b.w(o(0), p(&n1, 1));
    b.w(p(&n1, 2), p(&n2, 1));
    b.w(p(&n2, 2), p(&g, 0));
    check("nand", b);

    let mut b = B::new("nor");
    let (v, g) = (b.add(PartKind::Vcc, None), b.add(PartKind::Gnd, None));
    let (p1, p2) = (b.add(PartKind::Pmos, None), b.add(PartKind::Pmos, None));
    let (n1, n2) = (b.add(PartKind::Nmos, None), b.add(PartKind::Nmos, None));
    for (x, gate) in [(&p1, 0), (&n1, 0), (&p2, 1), (&n2, 1)] {
        b.w(i(gate), p(x, 0));
    }
    b.w(p(&v, 0), p(&p1, 1));
    b.w(p(&p1, 2), p(&p2, 1));
    b.w(p(&p2, 2), o(0));
    b.w(o(0), p(&n1, 1));
    b.w(o(0), p(&n2, 1));
    b.w(p(&n1, 2), p(&g, 0));
    b.w(p(&n2, 2), p(&g, 0));
    check("nor", b);

    // Gates
    let mut b = B::new("and");
    let n = b.g2("nand", i(0), i(1));
    let y = b.g1("not", n);
    b.w(y, o(0));
    check("and", b);

    let mut b = B::new("or");
    let n = b.g2("nor", i(0), i(1));
    let y = b.g1("not", n);
    b.w(y, o(0));
    check("or", b);

    let mut b = B::new("xor");
    let x = b.g2("or", i(0), i(1));
    let y = b.g2("nand", i(0), i(1));
    let z = b.g2("and", x, y);
    b.w(z, o(0));
    check("xor", b);

    let mut b = B::new("mux");
    let ns = b.g1("not", i(2));
    let a = b.g2("nand", i(0), ns);
    let c = b.g2("nand", i(1), i(2));
    let y = b.g2("nand", a, c);
    b.w(y, o(0));
    check("mux", b);

    // Arithmetic
    let mut b = B::new("half_adder");
    let s = b.g2("xor", i(0), i(1));
    let c = b.g2("and", i(0), i(1));
    b.w(s, o(0));
    b.w(c, o(1));
    check("half_adder", b);

    let mut b = B::new("full_adder");
    let (h1, h2) = (b.chip("half_adder"), b.chip("half_adder"));
    b.w(i(0), p(&h1, 0));
    b.w(i(1), p(&h1, 1));
    b.w(p(&h1, 2), p(&h2, 0));
    b.w(i(2), p(&h2, 1));
    b.w(p(&h2, 2), o(0));
    let c = b.g2("or", p(&h1, 3), p(&h2, 3));
    b.w(c, o(1));
    check("full_adder", b);

    // Wide buses
    let mut b = B::new("add4");
    let mut carry = i(8);
    for k in 0..4 {
        let fa = b.chip("full_adder");
        b.w(i(k), p(&fa, 0));
        b.w(i(4 + k), p(&fa, 1));
        b.w(carry, p(&fa, 2));
        b.w(p(&fa, 3), o(k));
        carry = p(&fa, 4);
    }
    b.w(carry, o(4));
    check("add4", b);

    let mut b = B::new("add8");
    let (lo, hi) = (b.chip("add4"), b.chip("add4"));
    for k in 0..4 {
        b.w(i(k), p(&lo, k));
        b.w(i(8 + k), p(&lo, 4 + k));
        b.w(i(4 + k), p(&hi, k));
        b.w(i(12 + k), p(&hi, 4 + k));
        b.w(p(&lo, 9 + k), o(k));
        b.w(p(&hi, 9 + k), o(4 + k));
    }
    b.w(i(16), p(&lo, 8));
    b.w(p(&lo, 13), p(&hi, 8));
    b.w(p(&hi, 13), o(8));
    check("add8", b);

    let mut b = B::new("flip4");
    for k in 0..4 {
        let y = b.g2("xor", i(k), i(4));
        b.w(y, o(k));
    }
    check("flip4", b);

    let mut b = B::new("flip8");
    let (lo, hi) = (b.chip("flip4"), b.chip("flip4"));
    for k in 0..4 {
        b.w(i(k), p(&lo, k));
        b.w(i(4 + k), p(&hi, k));
        b.w(p(&lo, 5 + k), o(k));
        b.w(p(&hi, 5 + k), o(4 + k));
    }
    b.w(i(8), p(&lo, 4));
    b.w(i(8), p(&hi, 4));
    check("flip8", b);

    let mut b = B::new("zero8");
    let mut acc = i(0);
    for k in 1..8 {
        acc = b.g2("or", acc, i(k));
    }
    let z = b.g1("not", acc);
    b.w(z, o(0));
    check("zero8", b);

    let mut b = B::new("alu8");
    let (f, ad, z) = (b.chip("flip8"), b.chip("add8"), b.chip("zero8"));
    for k in 0..8 {
        b.w(i(8 + k), p(&f, k));
        b.w(p(&f, 9 + k), p(&ad, 8 + k));
        b.w(i(k), p(&ad, k));
        b.w(p(&ad, 17 + k), o(k));
        b.w(p(&ad, 17 + k), p(&z, k));
    }
    b.w(i(16), p(&f, 8));
    b.w(i(16), p(&ad, 16));
    b.w(p(&ad, 25), o(8));
    b.w(p(&z, 8), o(9));
    check("alu8", b);

    // Memory
    let mut b = B::new("sr_latch");
    let (n1, n2) = (b.chip("nor"), b.chip("nor"));
    b.w(i(1), p(&n1, 0));
    b.w(i(0), p(&n2, 0));
    b.w(p(&n2, 2), p(&n1, 1));
    b.w(p(&n1, 2), p(&n2, 1));
    b.w(p(&n1, 2), o(0));
    b.w(p(&n2, 2), o(1));
    check("sr_latch", b);

    let mut b = B::new("d_latch");
    let nd = b.g1("not", i(0));
    let s = b.g2("and", i(0), i(1));
    let r = b.g2("and", nd, i(1));
    let sr = b.chip("sr_latch");
    b.w(s, p(&sr, 0));
    b.w(r, p(&sr, 1));
    b.w(p(&sr, 2), o(0));
    check("d_latch", b);

    let mut b = B::new("dff");
    let nc = b.g1("not", i(1));
    let (m, s) = (b.chip("d_latch"), b.chip("d_latch"));
    b.w(i(0), p(&m, 0));
    b.w(nc, p(&m, 1));
    b.w(p(&m, 2), p(&s, 0));
    b.w(i(1), p(&s, 1));
    b.w(p(&s, 2), o(0));
    check("dff", b);

    let mut b = B::new("reg_bit");
    let (mx, d) = (b.chip("mux"), b.chip("dff"));
    b.w(p(&d, 2), p(&mx, 0));
    b.w(i(0), p(&mx, 1));
    b.w(i(1), p(&mx, 2));
    b.w(p(&mx, 3), p(&d, 0));
    b.w(i(2), p(&d, 1));
    b.w(p(&d, 2), o(0));
    check("reg_bit", b);

    let mut b = B::new("reg4");
    for k in 0..4 {
        let r = b.chip("reg_bit");
        b.w(i(k), p(&r, 0));
        b.w(i(4), p(&r, 1));
        b.w(i(5), p(&r, 2));
        b.w(p(&r, 3), o(k));
    }
    check("reg4", b);

    let mut b = B::new("reg8");
    for half in 0..2 {
        let r = b.chip("reg4");
        for k in 0..4 {
            b.w(i(half * 4 + k), p(&r, k));
            b.w(p(&r, 6 + k), o(half * 4 + k));
        }
        b.w(i(8), p(&r, 4));
        b.w(i(9), p(&r, 5));
    }
    check("reg8", b);

    // The machine
    let mut b = B::new("accum");
    let (reg, alu, z, v) = (
        b.chip("reg8"),
        b.chip("alu8"),
        b.chip("zero8"),
        b.add(PartKind::Vcc, None),
    );
    b.w(p(&v, 0), p(&reg, 8));
    b.w(i(9), p(&reg, 9));
    b.w(i(8), p(&alu, 16));
    for k in 0..8 {
        b.w(p(&reg, 10 + k), p(&alu, k));
        b.w(i(k), p(&alu, 8 + k));
        b.w(p(&alu, 17 + k), p(&reg, k));
        b.w(p(&reg, 10 + k), p(&z, k));
        b.w(p(&reg, 10 + k), o(k));
    }
    b.w(p(&z, 8), o(8));
    check("accum", b);
}

#[test]
fn half_built_inverter_floats() {
    let mut b = B::new("not");
    let v = b.add(PartKind::Vcc, None);
    let pm = b.add(PartKind::Pmos, None);
    b.w(i(0), p(&pm, 0));
    b.w(p(&v, 0), p(&pm, 1));
    b.w(p(&pm, 2), o(0));
    let r = run("not", &b);
    assert!(!r.pass);
    assert!(r.failure.unwrap().note.contains("floating"));
}

#[test]
fn vcc_to_gnd_is_a_short() {
    let mut b = B::new("power");
    let (v, g) = (b.add(PartKind::Vcc, None), b.add(PartKind::Gnd, None));
    b.w(p(&v, 0), o(0));
    b.w(p(&g, 0), o(0));
    b.w(p(&g, 0), o(1));
    let r = run("power", &b);
    assert!(!r.pass);
    assert!(r.failure.unwrap().note.contains("Short"));
}

#[test]
fn transistor_ring_oscillator_is_unstable() {
    // Three inverters in a loop, Y tapped off the ring.
    let mut b = B::new("not");
    let mut prev: Option<P> = None;
    let mut first_in: Option<P> = None;
    for _ in 0..3 {
        let (v, g) = (b.add(PartKind::Vcc, None), b.add(PartKind::Gnd, None));
        let (pm, nm) = (b.add(PartKind::Pmos, None), b.add(PartKind::Nmos, None));
        b.w(p(&pm, 0), p(&nm, 0));
        b.w(p(&v, 0), p(&pm, 1));
        b.w(p(&pm, 2), p(&nm, 1));
        b.w(p(&nm, 2), p(&g, 0));
        if let Some(pr) = prev.take() {
            b.w(pr, p(&pm, 0));
        } else {
            first_in = Some(p(&pm, 0));
        }
        prev = Some(p(&pm, 2));
    }
    let out = prev.unwrap();
    b.w(out.clone(), first_in.unwrap());
    b.w(out, o(0));
    let r = run("not", &b);
    assert!(!r.pass);
}

#[test]
fn locked_chips_are_rejected() {
    let mut b = B::new("and");
    let x = b.g2("xor", i(0), i(1));
    b.w(x, o(0));
    let done = BTreeMap::new();
    let ctx = Ctx {
        done: &done,
        unlock_all: true,
    };
    let r = game::verify(game::index_of("and").unwrap(), &b.c, &ctx);
    assert!(r.error.unwrap().contains("not available"));
}

fn check_cost(level: &str, b: &B) -> u32 {
    check_delay(level, b);
    let r = run(level, b);
    assert!(r.pass, "{level} failed: {r:?}");
    println!(
        "{level}: cost {} par {}",
        r.cost,
        game::level(level).unwrap().par
    );
    r.cost
}

/// Wire `n` consecutive pins: a[i] -> b[i].
fn bus(b: &mut B, a: &str, a0: usize, to: &str, b0: usize, n: usize) {
    for k in 0..n {
        b.w(p(a, a0 + k), p(to, b0 + k));
    }
}

/// RAM built from two halves of the size below.
fn ram_level(level: &str, sub: &str, abits: usize) -> B {
    let mut b = B::new(level);
    let (we, clk) = (abits + 8, abits + 9);
    let top = abits - 1;
    let sub_d = top; // data pins start after the sub-RAM's address pins
    let sub_q = top + 10;
    let n = b.g1("not", i(top));
    let w0 = b.g2("and", i(we), n);
    let w1 = b.g2("and", i(we), i(top));
    let mx = b.chip("mux8");
    for (h, wsel) in [(0, w0), (1, w1)] {
        let r = b.chip(sub);
        if sub == "reg8" {
            for k in 0..8 {
                b.w(i(abits + k), p(&r, k));
                b.w(p(&r, 10 + k), p(&mx, h * 8 + k));
            }
            b.w(wsel, p(&r, 8));
            b.w(i(clk), p(&r, 9));
        } else {
            for k in 0..top {
                b.w(i(k), p(&r, k));
            }
            for k in 0..8 {
                b.w(i(abits + k), p(&r, sub_d + k));
                b.w(p(&r, sub_q + k), p(&mx, h * 8 + k));
            }
            b.w(wsel, p(&r, top + 8));
            b.w(i(clk), p(&r, top + 9));
        }
    }
    b.w(i(top), p(&mx, 16));
    for k in 0..8 {
        b.w(p(&mx, 17 + k), o(k));
    }
    b
}

/// The reference CPU, also used inside the computer level.
fn build_cpu(b: &mut B) {
    let ctrl = b.chip("ctrl");
    let pc = b.chip("pc4");
    let acc = b.chip("reg8");
    let outr = b.chip("reg8");
    let ram = b.chip("ram16");
    let alu = b.chip("alu8");
    let z = b.chip("zero8");
    let mb = b.chip("mux8");
    let ma = b.chip("mux8");
    let gnd = b.add(PartKind::Gnd, None);
    // control: OP = I4..I7, Z from ZERO8
    for k in 0..4 {
        b.w(i(4 + k), p(&ctrl, k));
    }
    b.w(p(&z, 8), p(&ctrl, 4));
    let (bimm, pass, sub, accw, ramw, jump, outw, halt) = (5, 6, 7, 8, 9, 10, 11, 12);
    // PC
    for k in 0..4 {
        b.w(i(k), p(&pc, k));
        b.w(p(&pc, 7 + k), o(k));
    }
    b.w(p(&ctrl, jump), p(&pc, 4));
    let nh = b.g1("not", p(&ctrl, halt));
    b.w(nh, p(&pc, 5));
    b.w(i(8), p(&pc, 6));
    // RAM: address n, data ACC
    for k in 0..4 {
        b.w(i(k), p(&ram, k));
    }
    for k in 0..8 {
        b.w(p(&acc, 10 + k), p(&ram, 4 + k));
    }
    b.w(p(&ctrl, ramw), p(&ram, 12));
    b.w(i(8), p(&ram, 13));
    // B mux: RAM output or operand
    for k in 0..8 {
        b.w(p(&ram, 14 + k), p(&mb, k));
    }
    for k in 0..4 {
        b.w(i(k), p(&mb, 8 + k));
        b.w(p(&gnd, 0), p(&mb, 12 + k));
    }
    b.w(p(&ctrl, bimm), p(&mb, 16));
    // ALU
    for k in 0..8 {
        b.w(p(&acc, 10 + k), p(&alu, k));
        b.w(p(&mb, 17 + k), p(&alu, 8 + k));
    }
    b.w(p(&ctrl, sub), p(&alu, 16));
    // ACC input mux
    for k in 0..8 {
        b.w(p(&alu, 17 + k), p(&ma, k));
        b.w(p(&mb, 17 + k), p(&ma, 8 + k));
        b.w(p(&ma, 17 + k), p(&acc, k));
    }
    b.w(p(&ctrl, pass), p(&ma, 16));
    b.w(p(&ctrl, accw), p(&acc, 8));
    b.w(i(8), p(&acc, 9));
    // zero flag
    for k in 0..8 {
        b.w(p(&acc, 10 + k), p(&z, k));
    }
    // OUT register
    for k in 0..8 {
        b.w(p(&acc, 10 + k), p(&outr, k));
        b.w(p(&outr, 10 + k), o(4 + k));
    }
    b.w(p(&ctrl, outw), p(&outr, 8));
    b.w(i(8), p(&outr, 9));
    b.w(p(&ctrl, halt), o(12));
}

fn computer_board(b: &mut B, prog: &[u8]) {
    let cpu = b.chip("cpu");
    let rom = b.add(PartKind::Rom, None);
    b.c.parts.last_mut().unwrap().data = Some(prog.to_vec());
    bus(b, &rom, 4, &cpu, 0, 8);
    bus(b, &cpu, 9, &rom, 0, 4);
    b.w(i(0), p(&cpu, 8));
    for k in 0..9 {
        b.w(p(&cpu, 13 + k), o(k));
    }
}

#[test]
fn computer_marks_have_solutions_at_par() {
    let mut costs = BTreeMap::new();
    let mut put = |l: &str, b: B| {
        let c = check_cost(l, &b);
        costs.insert(l.to_string(), (c, game::level(l).unwrap().par));
    };

    let mut b = B::new("inc4");
    let v = b.add(PartKind::Vcc, None);
    let mut carry = p(&v, 0);
    for k in 0..4 {
        let h = b.chip("half_adder");
        b.w(i(k), p(&h, 0));
        b.w(carry, p(&h, 1));
        b.w(p(&h, 2), o(k));
        carry = p(&h, 3);
    }
    put("inc4", b);

    let mut b = B::new("mux4");
    for k in 0..4 {
        let m = b.chip("mux");
        b.w(i(k), p(&m, 0));
        b.w(i(4 + k), p(&m, 1));
        b.w(i(8), p(&m, 2));
        b.w(p(&m, 3), o(k));
    }
    put("mux4", b);

    let mut b = B::new("mux8");
    for h in 0..2 {
        let m = b.chip("mux4");
        for k in 0..4 {
            b.w(i(h * 4 + k), p(&m, k));
            b.w(i(8 + h * 4 + k), p(&m, 4 + k));
            b.w(p(&m, 9 + k), o(h * 4 + k));
        }
        b.w(i(16), p(&m, 8));
    }
    put("mux8", b);

    let mut b = B::new("pc4");
    let (inc, mx, r) = (b.chip("inc4"), b.chip("mux4"), b.chip("reg4"));
    for k in 0..4 {
        b.w(p(&r, 6 + k), p(&inc, k));
        b.w(p(&inc, 4 + k), p(&mx, k));
        b.w(i(k), p(&mx, 4 + k));
        b.w(p(&mx, 9 + k), p(&r, k));
        b.w(p(&r, 6 + k), o(k));
    }
    b.w(i(4), p(&mx, 8));
    b.w(i(5), p(&r, 4));
    b.w(i(6), p(&r, 5));
    put("pc4", b);

    let mut b = B::new("dec2");
    let na0 = b.g1("not", i(0));
    let na1 = b.g1("not", i(1));
    let e0 = b.g2("and", i(2), na1);
    let e1 = b.g2("and", i(2), i(1));
    for (k, (e, a)) in [
        (e0.clone(), na0.clone()),
        (e0, i(0)),
        (e1.clone(), na0),
        (e1, i(0)),
    ]
    .into_iter()
    .enumerate()
    {
        let y = b.g2("and", e, a);
        b.w(y, o(k));
    }
    put("dec2", b);

    let mut b = B::new("dec4");
    let top = b.chip("dec2");
    b.w(i(2), p(&top, 0));
    b.w(i(3), p(&top, 1));
    b.w(i(4), p(&top, 2));
    for g in 0..4 {
        let d = b.chip("dec2");
        b.w(i(0), p(&d, 0));
        b.w(i(1), p(&d, 1));
        b.w(p(&top, 3 + g), p(&d, 2));
        for k in 0..4 {
            b.w(p(&d, 3 + k), o(g * 4 + k));
        }
    }
    put("dec4", b);

    put("ram2", ram_level("ram2", "reg8", 1));
    put("ram4", ram_level("ram4", "ram2", 2));
    put("ram8", ram_level("ram8", "ram4", 3));
    put("ram16", ram_level("ram16", "ram8", 4));

    let mut b = B::new("ctrl");
    let (d, v) = (b.chip("dec4"), b.add(PartKind::Vcc, None));
    for k in 0..4 {
        b.w(i(k), p(&d, k));
    }
    b.w(p(&v, 0), p(&d, 4));
    let y = |k: usize| p(&d, 5 + k);
    let bimm = b.g2("or", y(1), y(2));
    let pass = b.g2("or", y(1), y(5));
    let a1 = b.g2("or", bimm.clone(), y(3));
    let a2 = b.g2("or", a1, y(4));
    let accw = b.g2("or", a2, y(5));
    let jz = b.g2("and", y(8), i(4));
    let jump = b.g2("or", y(7), jz);
    for (k, s) in [bimm, pass, y(4), accw, y(6), jump, y(9), y(15)]
        .into_iter()
        .enumerate()
    {
        b.w(s, o(k));
    }
    put("ctrl", b);

    let mut b = B::new("cpu");
    build_cpu(&mut b);
    put("cpu", b);

    let mut b = B::new("computer");
    computer_board(&mut b, &[0x11, 0x90, 0xF0]);
    put("computer", b);

    for (id, prog) in game::REF_PROGRAMS {
        let mut b = B::new(id);
        computer_board(&mut b, prog);
        put(id, b);
    }

    for (l, (c, par)) in &costs {
        assert_eq!(c, par, "{l} cost vs par");
    }
}

#[test]
fn computer_rejects_wrong_programs() {
    let mut b = B::new("prog_countdown");
    computer_board(&mut b, &[0x19, 0x90, 0xF0]);
    let r = run("prog_countdown", &b);
    assert!(!r.pass);
    assert_eq!(r.program.unwrap().outs, vec![9]);
}

#[test]
fn programs_must_run_on_the_board() {
    let mut b = B::new("prog_answer");
    // A correct program in a ROM with no CPU wired to it.
    b.add(PartKind::Rom, None);
    b.c.parts.last_mut().unwrap().data = Some(game::REF_PROGRAMS[0].1.to_vec());
    let r = run("prog_answer", &b);
    assert!(!r.pass);
    assert!(r.failure.is_some(), "{r:?}");
}

#[test]
fn computer_without_rom_is_rejected() {
    let mut b = B::new("computer");
    let cpu = b.chip("cpu");
    b.w(i(0), p(&cpu, 8));
    let r = run("computer", &b);
    assert!(r.error.unwrap().contains("ROM"));
}

#[test]
fn cpu_with_broken_jump_is_caught() {
    let mut b = B::new("cpu");
    build_cpu(&mut b);
    // Cut JUMP from the PC's LOAD pin and tie LOAD low instead.
    let pc =
        b.c.parts
            .iter()
            .find(|x| x.chip.as_deref() == Some("pc4"))
            .unwrap()
            .id
            .clone();
    b.c.wires.retain(|w| !(w.to.part == pc && w.to.pin == 4));
    let g = b.add(PartKind::Gnd, None);
    b.w(p(&g, 0), p(&pc, 4));
    let r = run("cpu", &b);
    assert!(!r.pass);
    let f = r.failure.unwrap();
    assert!(f.note.contains("Wrong value"), "{}", f.note);
}
