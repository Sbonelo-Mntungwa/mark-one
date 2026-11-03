//! Static timing: the critical path of a circuit, in gate delays (see
//! [`crate::timing`]).
//!
//! Arrival times start at 0 on every INPUT pin and grow along each timing
//! arc: one unit through a transistor (gate to channel, or along a channel
//! from a driven line), a chip's delay through a chip. Supply rails never
//! change, so paths never start from them. Nets joined by conducting
//! channels form one node. Edge-triggered chips start new paths at CLK and
//! end the paths into their data inputs, so a register breaks a loop. A
//! loop with no register in it (a latch built from gates) is cut where the
//! search first meets it, and reported.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::circuit::{Circuit, PartKind};
use crate::components;
use crate::export::Nets;
use crate::sim::union_find::Uf;
use crate::timing;
use crate::workbench::{self, Library};

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct PathStep {
    /// Part id on the board, and what it is.
    pub part: String,
    pub what: String,
    /// Arrival time at this part's output.
    pub at: u32,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct CriticalPath {
    /// Longest input-to-output delay; None if no output depends on an input.
    pub depth: Option<u32>,
    /// Arrival time at each output pin, in pin order.
    pub outputs: Vec<(String, Option<u32>)>,
    /// The critical path, input first.
    pub path: Vec<PathStep>,
    /// A loop without a register was cut to measure the rest.
    pub loops: bool,
}

struct Edge {
    from: usize,
    to: usize,
    w: u32,
    part: usize,
}

/// Arcs of a chip as (input, output, delay).
fn chip_arcs(id: &str, lib: &Library, depth: usize) -> Result<Vec<(usize, usize, u32)>, String> {
    if let Some(k) = components::find(id) {
        let ins: Vec<String> = k.spec.inputs.iter().map(|s| s.to_string()).collect();
        let outs: Vec<String> = k.spec.outputs.iter().map(|s| s.to_string()).collect();
        let d = timing::delay(id).unwrap_or(1);
        return Ok(timing::arcs(id, &ins, &outs)
            .into_iter()
            .enumerate()
            .flat_map(|(o, is)| is.into_iter().map(move |i| (i, o, d)))
            .collect());
    }
    let def = lib
        .custom(id)
        .ok_or_else(|| format!("Unknown component '{id}'."))?;
    if depth > 32 {
        return Err("Components are nested too deeply.".into());
    }
    let n_in = def.inputs.len();
    let mut arcs = Vec::new();
    for i in 0..n_in {
        let r = analyse(&def.circuit, lib, Some(i), depth + 1)?;
        for (o, (_, at)) in r.outputs.iter().enumerate() {
            if let Some(at) = at {
                arcs.push((i, o, (*at).max(1)));
            }
        }
    }
    Ok(arcs)
}

/// The critical path of circuit `c`, built with `lib`.
pub fn critical_path(c: &Circuit, lib: &Library) -> Result<CriticalPath, String> {
    lib.check(c)?;
    analyse(c, lib, None, 0)
}

/// The longest arc through custom component `id`: its delay as one chip.
pub fn component_delay(id: &str, lib: &Library) -> Result<u32, String> {
    Ok(chip_arcs(id, lib, 0)?
        .iter()
        .map(|a| a.2)
        .max()
        .unwrap_or(1))
}

/// Analyse, starting from every input or only input pin `only`.
fn analyse(
    c: &Circuit,
    lib: &Library,
    only: Option<usize>,
    depth: usize,
) -> Result<CriticalPath, String> {
    let nets = Nets::new(c, lib)?;
    let n = nets.count;
    let mut source = vec![false; n];
    let chip_out = |k: usize, pins: std::ops::Range<usize>, src: &mut Vec<bool>| {
        for pin in pins {
            src[nets.of(k, pin)] = true;
        }
    };
    let mut pin_counts = Vec::new();
    for (k, p) in c.parts.iter().enumerate() {
        match p.kind {
            PartKind::Vcc | PartKind::Gnd | PartKind::In => chip_out(k, 0..1, &mut source),
            PartKind::Rom => chip_out(k, 4..12, &mut source),
            PartKind::Chip => {
                let id = p.chip.as_deref().unwrap_or("");
                let (i, o) = lib
                    .pins(id)
                    .ok_or_else(|| format!("Unknown component '{id}'."))?;
                chip_out(k, i.len()..i.len() + o.len(), &mut source);
                pin_counts.push((k, i.len()));
            }
            _ => {}
        }
    }
    // Channel-connected nets that nothing drives directly form one node.
    let mut uf = Uf::new(n);
    for (k, p) in c.parts.iter().enumerate() {
        if matches!(p.kind, PartKind::Nmos | PartKind::Pmos) {
            let (a, b) = (nets.of(k, 1), nets.of(k, 2));
            if !source[a] && !source[b] {
                uf.union(a, b);
            }
        }
    }
    let node: Vec<usize> = (0..n).map(|i| uf.find(i)).collect();

    let mut edges = Vec::new();
    let mut add = |from: usize, to: usize, w: u32, part: usize| {
        if node[from] != node[to] {
            edges.push(Edge {
                from: node[from],
                to: node[to],
                w,
                part,
            });
        }
    };
    let mut arc_cache: BTreeMap<String, Vec<(usize, usize, u32)>> = BTreeMap::new();
    for (k, p) in c.parts.iter().enumerate() {
        match p.kind {
            PartKind::Nmos | PartKind::Pmos => {
                let (g, a, b) = (nets.of(k, 0), nets.of(k, 1), nets.of(k, 2));
                // A driven line is an ideal source: only the undriven side of
                // a channel responds to its gate.
                if !source[a] {
                    add(g, a, 1, k);
                }
                if !source[b] {
                    add(g, b, 1, k);
                }
                if source[a] {
                    add(a, b, 1, k);
                }
                if source[b] {
                    add(b, a, 1, k);
                }
            }
            PartKind::Rom => {
                let d = timing::delay("rom").unwrap_or(1);
                for i in 0..4 {
                    for o in 4..12 {
                        add(nets.of(k, i), nets.of(k, o), d, k);
                    }
                }
            }
            PartKind::Chip => {
                let id = p.chip.as_deref().unwrap_or("");
                if !arc_cache.contains_key(id) {
                    arc_cache.insert(id.to_string(), chip_arcs(id, lib, depth)?);
                }
                let n_in = pin_counts.iter().find(|x| x.0 == k).map_or(0, |x| x.1);
                for &(i, o, d) in &arc_cache[id] {
                    add(nets.of(k, i), nets.of(k, n_in + o), d, k);
                }
            }
            _ => {}
        }
    }

    // Start nodes: input pins.
    let mut arrival: Vec<Option<u32>> = vec![None; n];
    let mut starts = Vec::new();
    for (k, p) in c.parts.iter().enumerate() {
        if p.kind == PartKind::In && only.is_none_or(|o| p.pin == Some(o)) {
            let v = node[nets.of(k, 0)];
            arrival[v] = Some(0);
            starts.push(v);
        }
    }

    // Cut back edges found by depth-first search from the starts, then
    // relax in topological order.
    let mut out_edges: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (e, x) in edges.iter().enumerate() {
        out_edges[x.from].push(e);
    }
    let mut cut = vec![false; edges.len()];
    let mut state = vec![0u8; n]; // 0 new, 1 on stack, 2 done
    let mut loops = false;
    for &s in &starts {
        if state[s] != 0 {
            continue;
        }
        let mut stack = vec![(s, 0usize)];
        state[s] = 1;
        while let Some(&mut (v, ref mut i)) = stack.last_mut() {
            if let Some(&e) = out_edges[v].get(*i) {
                *i += 1;
                let t = edges[e].to;
                match state[t] {
                    0 => {
                        state[t] = 1;
                        stack.push((t, 0));
                    }
                    1 => {
                        cut[e] = true;
                        loops = true;
                    }
                    _ => {}
                }
            } else {
                state[v] = 2;
                stack.pop();
            }
        }
    }
    let mut indeg = vec![0usize; n];
    for (e, x) in edges.iter().enumerate() {
        if !cut[e] && state[x.from] == 2 {
            indeg[x.to] += 1;
        }
    }
    let mut pred: Vec<Option<usize>> = vec![None; n];
    let mut queue: Vec<usize> = (0..n).filter(|&v| state[v] == 2 && indeg[v] == 0).collect();
    while let Some(v) = queue.pop() {
        for &e in &out_edges[v] {
            if cut[e] {
                continue;
            }
            let x = &edges[e];
            if let Some(a) = arrival[v] {
                if arrival[x.to].is_none_or(|b| a + x.w > b) {
                    arrival[x.to] = Some(a + x.w);
                    pred[x.to] = Some(e);
                }
            }
            indeg[x.to] -= 1;
            if indeg[x.to] == 0 {
                queue.push(x.to);
            }
        }
    }

    let names = workbench::outputs(c);
    let out_node = |k: usize| {
        c.parts
            .iter()
            .position(|p| p.kind == PartKind::Out && p.pin == Some(k))
            .map(|i| node[nets.of(i, 0)])
    };
    let outputs: Vec<(String, Option<u32>)> = names
        .iter()
        .enumerate()
        .map(|(k, nm)| (nm.clone(), out_node(k).and_then(|v| arrival[v])))
        .collect();
    let worst = (0..names.len())
        .filter_map(|k| Some((out_node(k)?, outputs[k].1?)))
        .max_by_key(|&(_, a)| a);
    let mut path = Vec::new();
    if let Some((mut v, _)) = worst {
        while let Some(e) = pred[v] {
            let p = &c.parts[edges[e].part];
            let what = match p.kind {
                PartKind::Chip => {
                    let id = p.chip.as_deref().unwrap_or("");
                    components::find(id)
                        .map(|k| k.spec.label.to_string())
                        .or_else(|| lib.custom(id).map(|d| d.label.clone()))
                        .unwrap_or_default()
                }
                PartKind::Nmos => "NMOS".into(),
                PartKind::Pmos => "PMOS".into(),
                _ => "ROM".into(),
            };
            path.push(PathStep {
                part: p.id.clone(),
                what,
                at: arrival[v].unwrap_or(0),
            });
            v = edges[e].from;
        }
        path.reverse();
    }
    Ok(CriticalPath {
        depth: worst.map(|w| w.1),
        outputs,
        path,
        loops,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::component_circuit;
    use crate::circuit::Part;

    fn chain(chips: &[&str]) -> Circuit {
        let mut c = Circuit::default();
        c.parts.push(Part::pin("a", PartKind::In, 0, Some("A")));
        c.parts.push(Part::pin("y", PartKind::Out, 0, Some("Y")));
        let mut prev = ("a".to_string(), 0);
        for (k, id) in chips.iter().enumerate() {
            let pid = format!("g{k}");
            c.parts.push(Part::chip(&pid, id));
            c.connect((&prev.0, prev.1), (&pid, 0));
            prev = (pid, 1);
        }
        c.connect((&prev.0, prev.1), ("y", 0));
        c
    }

    #[test]
    fn delays_add_along_a_chain() {
        let r = critical_path(&chain(&["not", "not", "not"]), &Library::open()).unwrap();
        assert_eq!(r.depth, Some(3));
        assert_eq!(r.path.iter().map(|s| s.at).collect::<Vec<_>>(), [1, 2, 3]);
        assert!(!r.loops);
    }

    #[test]
    fn a_cmos_inverter_is_one_delay() {
        let mut c = Circuit::default();
        c.parts.push(Part::pin("a", PartKind::In, 0, Some("A")));
        c.parts.push(Part::pin("y", PartKind::Out, 0, Some("Y")));
        c.parts.push(Part::new("v", PartKind::Vcc));
        c.parts.push(Part::new("g", PartKind::Gnd));
        c.parts.push(Part::new("p", PartKind::Pmos));
        c.parts.push(Part::new("n", PartKind::Nmos));
        c.connect(("a", 0), ("p", 0));
        c.connect(("a", 0), ("n", 0));
        c.connect(("v", 0), ("p", 1));
        c.connect(("p", 2), ("y", 0));
        c.connect(("y", 0), ("n", 1));
        c.connect(("n", 2), ("g", 0));
        let r = critical_path(&c, &Library::open()).unwrap();
        assert_eq!(r.depth, Some(1));
    }

    #[test]
    fn a_register_starts_paths_at_its_clock() {
        // DFF: only CLK reaches Q; D is a path endpoint.
        let lib = Library::open();
        let r = critical_path(&component_circuit("dff", &lib).unwrap(), &lib).unwrap();
        assert_eq!(r.depth, timing::delay("dff"));
        let mut c = component_circuit("dff", &lib).unwrap();
        c.parts.retain(|p| p.label.as_deref() != Some("CLK"));
        c.wires.retain(|w| w.from.part != "i1");
        let r = critical_path(&c, &lib).unwrap();
        assert_eq!(r.depth, None, "D alone never reaches Q");
    }

    #[test]
    fn a_gate_loop_is_cut_and_reported() {
        // Cross-coupled NORs.
        let mut c = Circuit::default();
        c.parts.push(Part::pin("s", PartKind::In, 0, Some("S")));
        c.parts.push(Part::pin("r", PartKind::In, 1, Some("R")));
        c.parts.push(Part::pin("q", PartKind::Out, 0, Some("Q")));
        c.parts.push(Part::chip("n1", "nor"));
        c.parts.push(Part::chip("n2", "nor"));
        c.connect(("r", 0), ("n1", 0));
        c.connect(("n2", 2), ("n1", 1));
        c.connect(("s", 0), ("n2", 0));
        c.connect(("n1", 2), ("n2", 1));
        c.connect(("n1", 2), ("q", 0));
        let r = critical_path(&c, &Library::open()).unwrap();
        assert!(r.loops);
        assert_eq!(r.depth, Some(2));
    }

    #[test]
    fn every_library_component_has_a_delay() {
        for k in components::catalog() {
            assert!(timing::delay(k.spec.id).is_some(), "{}", k.spec.id);
        }
    }
}
