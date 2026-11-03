//! Switch-level behaviour that the component proofs do not cover.

use super::{Sim, CHARGE_SETTLES};
use crate::circuit::{Circuit, Part, PartKind};
use crate::signal::Sig;

fn fet(c: &mut Circuit, id: &str, p: bool, g: (&str, usize), s: (&str, usize), d: (&str, usize)) {
    c.parts.push(Part::new(
        id,
        if p { PartKind::Pmos } else { PartKind::Nmos },
    ));
    c.connect(g, (id, 0));
    c.connect(s, (id, 1));
    c.connect(d, (id, 2));
}

fn rails() -> Circuit {
    let mut c = Circuit::default();
    c.parts.push(Part::new("vcc", PartKind::Vcc));
    c.parts.push(Part::new("gnd", PartKind::Gnd));
    c
}

/// CMOS inverter `id` from net (a, k) to its own output (`id`p, 2).
fn inverter(c: &mut Circuit, id: &str, a: (&str, usize)) -> (String, usize) {
    let (p, n) = (format!("{id}p"), format!("{id}n"));
    fet(c, &p, true, a, ("vcc", 0), (&p, 2));
    fet(c, &n, false, a, (&p, 2), ("gnd", 0));
    (p, 2)
}

fn build(c: &Circuit, n_in: usize, n_out: usize) -> Sim {
    Sim::build(c, n_in, n_out, &|_| None, &|_| Ok(())).unwrap()
}

#[test]
fn an_inverter_fed_its_own_output_is_unknown() {
    let mut c = rails();
    c.parts.push(Part::pin("y", PartKind::Out, 0, None));
    fet(&mut c, "p", true, ("y", 0), ("vcc", 0), ("y", 0));
    fet(&mut c, "n", false, ("y", 0), ("y", 0), ("gnd", 0));
    let mut s = build(&c, 0, 1);
    s.settle();
    assert_eq!(
        s.outputs(),
        [Sig::X],
        "a self-fed inverter sits near mid-rail"
    );
}

#[test]
fn an_unknown_gate_makes_its_output_unknown_not_floating() {
    let mut c = rails();
    c.parts.push(Part::pin("y", PartKind::Out, 0, None));
    // Gate left unconnected: its value is unknown.
    let out = inverter(&mut c, "i", ("nowhere", 0));
    c.parts.push(Part::new("nowhere", PartKind::Display));
    c.connect((&out.0, out.1), ("y", 0));
    let mut s = build(&c, 0, 1);
    s.settle();
    assert_eq!(s.outputs(), [Sig::X]);
}

#[test]
fn a_ring_of_three_inverters_never_reads_as_a_valid_level() {
    let mut c = rails();
    c.parts.push(Part::pin("y", PartKind::Out, 0, None));
    c.parts.push(Part::new("start", PartKind::Display));
    let a = inverter(&mut c, "a", ("start", 0));
    let b = inverter(&mut c, "b", (&a.0, a.1));
    let d = inverter(&mut c, "c", (&b.0, b.1));
    c.connect((&d.0, d.1), ("start", 0));
    c.connect((&d.0, d.1), ("y", 0));
    let mut s = build(&c, 0, 1);
    s.settle();
    assert!(s.outputs() == [Sig::X] || s.unstable);
}

#[test]
fn nmos_passes_a_weak_one_and_a_strong_zero() {
    let mut c = rails();
    c.parts.push(Part::pin("a", PartKind::In, 0, None));
    c.parts.push(Part::pin("y", PartKind::Out, 0, None));
    fet(&mut c, "n", false, ("vcc", 0), ("a", 0), ("y", 0));
    let mut s = build(&c, 1, 1);
    s.set_inputs(&[false]);
    s.settle();
    assert_eq!(s.outputs(), [Sig::L]);
    s.set_inputs(&[true]);
    s.settle();
    assert_eq!(s.degraded_outputs(), [true]);
    assert_eq!(s.outputs(), [Sig::X]);
}

#[test]
fn stored_charge_leaks_away() {
    // A pass transistor charges a node, then opens; an inverter reads it.
    let mut c = rails();
    c.parts.push(Part::pin("d", PartKind::In, 0, None));
    c.parts.push(Part::pin("e", PartKind::In, 1, None));
    c.parts.push(Part::pin("y", PartKind::Out, 0, None));
    fet(&mut c, "t", false, ("e", 0), ("d", 0), ("t", 2));
    let out = inverter(&mut c, "i", ("t", 2));
    c.connect((&out.0, out.1), ("y", 0));
    let mut s = build(&c, 2, 1);
    s.set_inputs(&[false, true]);
    s.settle();
    s.set_inputs(&[false, false]);
    s.settle();
    assert_eq!(s.outputs(), [Sig::H], "held while the charge lasts");
    for _ in 0..CHARGE_SETTLES {
        s.settle();
    }
    assert_eq!(s.outputs(), [Sig::X], "then leaks to unknown");
}

#[test]
fn rails_are_not_corrupted_by_a_possible_short() {
    // An inverter with an unknown gate might short; a separate inverter on a
    // known input must still read correctly.
    let mut c = rails();
    c.parts.push(Part::pin("a", PartKind::In, 0, None));
    c.parts.push(Part::pin("y", PartKind::Out, 0, None));
    c.parts.push(Part::new("nowhere", PartKind::Display));
    inverter(&mut c, "u", ("nowhere", 0));
    let out = inverter(&mut c, "k", ("a", 0));
    c.connect((&out.0, out.1), ("y", 0));
    let mut s = build(&c, 1, 1);
    s.set_inputs(&[true]);
    s.settle();
    assert_eq!(s.outputs(), [Sig::L]);
}
