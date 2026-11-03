//! Verilog export.
//!
//! A design becomes a structural top module: one wire per net, one instance
//! per chip. Each library component it uses is written as a behavioural
//! module (`m1_<id>`), checked against the engine's own model by the tests
//! below whenever Icarus Verilog is installed. Custom components become
//! modules of their own, so the hierarchy survives, and each distinct ROM
//! image becomes a small lookup module.
//!
//! Transistors are written as Verilog's bidirectional switches: an NMOS is
//! `tranif1`, a PMOS `tranif0`. That simulates in Icarus and other
//! simulators, but FPGA synthesis tools do not accept switch-level designs;
//! for an FPGA, build from chips.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use super::Nets;
use crate::circuit::{Circuit, PartKind};
use crate::components;
use crate::workbench::{self, Library};

const KEYWORDS: &[&str] = &[
    "always",
    "and",
    "assign",
    "begin",
    "buf",
    "case",
    "default",
    "else",
    "end",
    "endcase",
    "endmodule",
    "for",
    "function",
    "if",
    "initial",
    "inout",
    "input",
    "integer",
    "module",
    "nand",
    "nmos",
    "nor",
    "not",
    "or",
    "output",
    "pmos",
    "reg",
    "supply0",
    "supply1",
    "task",
    "tran",
    "tranif0",
    "tranif1",
    "tri",
    "wire",
    "xor",
    "xnor",
];

/// A legal Verilog identifier for `s`.
pub fn ident(s: &str) -> String {
    let mut o: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if o.is_empty() || o.starts_with(|c: char| c.is_ascii_digit()) {
        o.insert_str(0, "p_");
    }
    if KEYWORDS.contains(&o.to_ascii_lowercase().as_str()) {
        o.push('_');
    }
    o
}

/// Port identifiers for a pin list, unique within the module.
fn ports(ins: &[String], outs: &[String]) -> (Vec<String>, Vec<String>) {
    let mut seen = BTreeSet::new();
    let mut name = |n: &String| {
        let base = ident(n);
        let mut s = base.clone();
        let mut k = 2;
        while !seen.insert(s.to_ascii_lowercase()) {
            s = format!("{base}_{k}");
            k += 1;
        }
        s
    };
    let i = ins.iter().map(&mut name).collect();
    let o = outs.iter().map(&mut name).collect();
    (i, o)
}

/// `{P7, ..., P0}` for pins P0..P(n-1).
fn cat(p: &str, n: usize) -> String {
    let v: Vec<String> = (0..n).rev().map(|k| format!("{p}{k}")).collect();
    format!("{{{}}}", v.join(", "))
}

fn header(name: &str, ins: &[&str], outs: &[&str]) -> String {
    let mut s = format!("module {name} (\n");
    let all: Vec<String> = ins
        .iter()
        .map(|n| format!("  input  wire {n}"))
        .chain(outs.iter().map(|n| format!("  output wire {n}")))
        .collect();
    s += &all.join(",\n");
    s += "\n);\n";
    s
}

fn ram_body(abits: usize) -> String {
    let n = 1 << abits;
    format!(
        "  reg [7:0] mem [0:{last}];\n  integer k;\n  initial for (k = 0; k < {n}; k = k + 1) mem[k] = 8'd0;\n  wire [{am}:0] a = {a};\n  always @(posedge CLK) if (WE) mem[a] <= {d};\n  assign {q} = mem[a];\n",
        last = n - 1,
        am = abits - 1,
        a = cat("A", abits),
        d = cat("D", 8),
        q = cat("Q", 8)
    )
}

/// Behavioural Verilog for library component `id`, or None if the library
/// has no such component.
pub fn library_module(id: &str) -> Option<String> {
    let spec = &components::find(id)?.spec;
    let body = match id {
        "not" => "  assign Y = ~A;\n".into(),
        "nand" => "  assign Y = ~(A & B);\n".into(),
        "nor" => "  assign Y = ~(A | B);\n".into(),
        "and" => "  assign Y = A & B;\n".into(),
        "or" => "  assign Y = A | B;\n".into(),
        "xor" => "  assign Y = A ^ B;\n".into(),
        "mux" => "  assign Y = SEL ? B : A;\n".into(),
        "half_adder" => "  assign S = A ^ B;\n  assign C = A & B;\n".into(),
        "full_adder" => "  assign {COUT, S} = A + B + CIN;\n".into(),
        "add4" | "add8" => {
            let n = if id == "add4" { 4 } else { 8 };
            let s = cat("S", n);
            format!(
                "  assign {{COUT, {}}} = {} + {} + CIN;\n",
                &s[1..s.len() - 1],
                cat("A", n),
                cat("B", n)
            )
        }
        "flip4" | "flip8" => {
            let n = if id == "flip4" { 4 } else { 8 };
            format!("  assign {} = {} ^ {{{n}{{INV}}}};\n", cat("Y", n), cat("X", n))
        }
        "zero8" => format!("  assign Z = ~|{};\n", cat("X", 8)),
        "alu8" => format!(
            "  wire [8:0] r = {a} + ({b} ^ {{8{{SUB}}}}) + SUB;\n  assign {s} = r[7:0];\n  assign C = r[8];\n  assign Z = (r[7:0] == 8'd0);\n",
            a = cat("A", 8),
            b = cat("B", 8),
            s = cat("S", 8)
        ),
        "sr_latch" => "  // Level-sensitive. S = R = 1 drives both outputs low; releasing\n  // both at once leaves the state unknown, as in the engine.\n  reg q = 1'b0;\n  reg forbidden = 1'b0;\n  always @(S or R) begin\n    if (S && R) forbidden = 1'b1;\n    else begin\n      if (S) q = 1'b1;\n      else if (R) q = 1'b0;\n      else if (forbidden) q = 1'bx;\n      forbidden = 1'b0;\n    end\n  end\n  assign Q = (S && R) ? 1'b0 : q;\n  assign QN = (S && R) ? 1'b0 : ~q;\n".into(),
        "d_latch" => "  // Level-sensitive: transparent while E is 1.\n  reg q = 1'b0;\n  always @(D or E) if (E) q = D;\n  assign Q = q;\n".into(),
        "dff" => "  reg q = 1'b0;\n  always @(posedge CLK) q <= D;\n  assign Q = q;\n".into(),
        "reg_bit" => "  reg q = 1'b0;\n  always @(posedge CLK) if (LOAD) q <= D;\n  assign Q = q;\n".into(),
        "reg4" | "reg8" => {
            let n = if id == "reg4" { 4 } else { 8 };
            format!(
                "  reg [{m}:0] q = {n}'d0;\n  always @(posedge CLK) if (LOAD) q <= {d};\n  assign {qq} = q;\n",
                m = n - 1,
                d = cat("D", n),
                qq = cat("Q", n)
            )
        }
        "accum" => format!(
            "  reg [7:0] acc = 8'd0;\n  always @(posedge CLK) acc <= SUB ? acc - {b} : acc + {b};\n  assign {o} = acc;\n  assign Z = (acc == 8'd0);\n",
            b = cat("B", 8),
            o = cat("ACC", 8)
        ),
        "inc4" => format!("  assign {} = {} + 4'd1;\n", cat("Y", 4), cat("A", 4)),
        "mux4" | "mux8" => {
            let n = if id == "mux4" { 4 } else { 8 };
            format!(
                "  assign {} = SEL ? {} : {};\n",
                cat("Y", n),
                cat("B", n),
                cat("A", n)
            )
        }
        "pc4" => format!(
            "  reg [3:0] q = 4'd0;\n  always @(posedge CLK) if (EN) q <= LOAD ? {d} : q + 4'd1;\n  assign {o} = q;\n",
            d = cat("D", 4),
            o = cat("Q", 4)
        ),
        "dec2" | "dec4" => {
            let (a, n) = if id == "dec2" { (2, 4) } else { (4, 16) };
            format!(
                "  assign {} = EN ? ({n}'d1 << {}) : {n}'d0;\n",
                cat("Y", n),
                cat("A", a)
            )
        }
        "ram2" => ram_body(1),
        "ram4" => ram_body(2),
        "ram8" => ram_body(3),
        "ram16" => ram_body(4),
        "ctrl" => format!(
            "  wire [3:0] o = {};\n  assign BIMM = (o == 4'd1) || (o == 4'd2);\n  assign PASS = (o == 4'd1) || (o == 4'd5);\n  assign SUB  = (o == 4'd4);\n  assign ACCW = (o >= 4'd1) && (o <= 4'd5);\n  assign RAMW = (o == 4'd6);\n  assign JUMP = (o == 4'd7) || ((o == 4'd8) && Z);\n  assign OUTW = (o == 4'd9);\n  assign HALT = (o == 4'd15);\n",
            cat("OP", 4)
        ),
        "cpu" => format!(
            "  // Single-cycle accumulator machine; see the instruction set in the\n  // Mark One README. Executes the byte on I at each rising edge of CLK.\n  wire [7:0] i = {i};\n  wire [3:0] n = i[3:0];\n  reg [3:0] pc = 4'd0;\n  reg [7:0] acc = 8'd0;\n  reg [7:0] out = 8'd0;\n  reg [7:0] ram [0:15];\n  integer k;\n  initial for (k = 0; k < 16; k = k + 1) ram[k] = 8'd0;\n  always @(posedge CLK) begin\n    case (i[7:4])\n      4'd1: acc <= {{4'd0, n}};\n      4'd2: acc <= acc + {{4'd0, n}};\n      4'd3: acc <= acc + ram[n];\n      4'd4: acc <= acc - ram[n];\n      4'd5: acc <= ram[n];\n      4'd6: ram[n] <= acc;\n      4'd9: out <= acc;\n      default: ;\n    endcase\n    case (i[7:4])\n      4'd7: pc <= n;\n      4'd8: pc <= (acc == 8'd0) ? n : pc + 4'd1;\n      4'd15: pc <= pc;\n      default: pc <= pc + 4'd1;\n    endcase\n  end\n  assign {p} = pc;\n  assign {o} = out;\n  assign HALT = (i[7:4] == 4'd15);\n",
            i = cat("I", 8),
            p = cat("PC", 4),
            o = cat("OUT", 8)
        ),
        _ => return None,
    };
    let mut s = format!("// {}: {}\n", spec.name, spec.summary);
    s += &header(&format!("m1_{id}"), spec.inputs, spec.outputs);
    s += &body;
    s += "endmodule\n";
    Some(s)
}

fn rom_name(data: &[u8]) -> String {
    let mut b = [0u8; 16];
    for (i, x) in data.iter().take(16).enumerate() {
        b[i] = *x;
    }
    format!(
        "m1_rom_{}",
        b.iter().map(|x| format!("{x:02x}")).collect::<String>()
    )
}

fn rom_module(data: &[u8]) -> String {
    let mut s = "// 16-byte mask ROM.\n".to_string();
    let ins: Vec<String> = (0..4).map(|k| format!("A{k}")).collect();
    let outs: Vec<String> = (0..8).map(|k| format!("D{k}")).collect();
    let (i, o): (Vec<&str>, Vec<&str>) = (
        ins.iter().map(|s| s.as_str()).collect(),
        outs.iter().map(|s| s.as_str()).collect(),
    );
    s += &header(&rom_name(data), &i, &o);
    s += &format!("  reg [7:0] d;\n  always @* case ({})\n", cat("A", 4));
    for a in 0..16 {
        let v = data.get(a).copied().unwrap_or(0);
        s += &format!("    4'd{a}: d = 8'h{v:02x};\n");
    }
    s += &format!("  endcase\n  assign {} = d;\nendmodule\n", cat("D", 8));
    s
}

/// Everything one export needs, gathered before writing.
struct Out<'a> {
    lib: &'a Library<'a>,
    library: BTreeSet<String>,
    roms: BTreeMap<String, Vec<u8>>,
    /// Custom component modules, dependencies first.
    customs: Vec<(String, String)>,
    done: BTreeSet<String>,
}

impl Out<'_> {
    fn custom_module_name(id: &str) -> String {
        ident(&id.replace(crate::custom::PREFIX, "custom_"))
    }

    /// Write the module for circuit `c`, collecting what it instantiates.
    fn design(&mut self, name: &str, c: &Circuit) -> Result<String, String> {
        let nets = Nets::new(c, self.lib)?;
        let (ins, outs) = (workbench::inputs(c), workbench::outputs(c));
        let (pi, po) = ports(&ins, &outs);
        let mut body = String::new();
        let mut inst = String::new();
        for (k, p) in c.parts.iter().enumerate() {
            let n = nets.pins(k);
            let w = |i: usize| format!("n{}", n[i]);
            let pid = ident(&p.id);
            match p.kind {
                PartKind::Vcc => writeln!(body, "  assign {} = 1'b1; // VCC {}", w(0), p.id).ok(),
                PartKind::Gnd => writeln!(body, "  assign {} = 1'b0; // GND {}", w(0), p.id).ok(),
                PartKind::In => {
                    let k = p.pin.unwrap_or(0);
                    writeln!(body, "  assign {} = {};", w(0), pi[k]).ok()
                }
                PartKind::Out => None,
                PartKind::Nmos => writeln!(
                    inst,
                    "  tranif1 t_{pid} ({}, {}, {}); // NMOS",
                    w(1),
                    w(2),
                    w(0)
                )
                .ok(),
                PartKind::Pmos => writeln!(
                    inst,
                    "  tranif0 t_{pid} ({}, {}, {}); // PMOS",
                    w(1),
                    w(2),
                    w(0)
                )
                .ok(),
                PartKind::Display => {
                    let v: Vec<String> = (0..8).rev().map(w).collect();
                    writeln!(inst, "  // DISPLAY {} reads {{{}}}", p.id, v.join(", ")).ok()
                }
                PartKind::Rom => {
                    let data = p.data.clone().unwrap_or_default();
                    let m = rom_name(&data);
                    self.roms.insert(m.clone(), data);
                    let conns: Vec<String> = (0..4)
                        .map(|k| format!(".A{k}({})", w(k)))
                        .chain((0..8).map(|k| format!(".D{k}({})", w(4 + k))))
                        .collect();
                    writeln!(inst, "  {m} u_{pid} ({});", conns.join(", ")).ok()
                }
                PartKind::Chip => {
                    let id = p.chip.as_deref().unwrap_or("");
                    let (ci, co) = self
                        .lib
                        .pins(id)
                        .ok_or_else(|| format!("Unknown component '{id}'."))?;
                    let module = if components::find(id).is_some() {
                        self.library.insert(id.to_string());
                        format!("m1_{id}")
                    } else {
                        let m = Self::custom_module_name(id);
                        if self.done.insert(id.to_string()) {
                            let def = self.lib.custom(id).expect("checked").clone();
                            let text = self.design(&m, &def.circuit)?;
                            let text = format!("// Custom component: {}\n{text}", def.name);
                            self.customs.push((id.to_string(), text));
                        }
                        m
                    };
                    let (qi, qo) = ports(&ci, &co);
                    let conns: Vec<String> = qi
                        .iter()
                        .chain(&qo)
                        .enumerate()
                        .map(|(k, port)| format!(".{port}({})", w(k)))
                        .collect();
                    writeln!(inst, "  {module} u_{pid} ({});", conns.join(", ")).ok()
                }
            };
        }
        let mut s = header(
            name,
            &pi.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            &po.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        );
        if nets.count > 0 {
            let all: Vec<String> = (0..nets.count).map(|k| format!("n{k}")).collect();
            for chunk in all.chunks(16) {
                writeln!(s, "  wire {};", chunk.join(", ")).ok();
            }
        }
        s += &body;
        s += &inst;
        for (k, port) in po.iter().enumerate() {
            match c
                .parts
                .iter()
                .position(|p| p.kind == PartKind::Out && p.pin == Some(k))
            {
                Some(i) => writeln!(s, "  assign {port} = n{};", nets.of(i, 0)).ok(),
                None => writeln!(s, "  assign {port} = 1'bz;").ok(),
            };
        }
        s += "endmodule\n";
        Ok(s)
    }
}

/// Verilog for design `c`, top module named after `top`, built with `lib`.
pub fn verilog(c: &Circuit, lib: &Library, top: &str) -> Result<String, String> {
    lib.check(c)?;
    let mut o = Out {
        lib,
        library: BTreeSet::new(),
        roms: BTreeMap::new(),
        customs: Vec::new(),
        done: BTreeSet::new(),
    };
    let top_name = ident(top);
    let top_text = o.design(&top_name, c)?;
    let fets = c
        .parts
        .iter()
        .any(|p| matches!(p.kind, PartKind::Nmos | PartKind::Pmos))
        || o.customs.iter().any(|(_, t)| t.contains("tranif"));
    let mut s = format!(
        "// {top}: exported from Mark One.\n//\n// Top module: {top_name}. Library chips are behavioural models that match\n// the Mark One engine; memories power up as 0 (real ones power up unknown).\n// Mark One is zero-delay, so this file has no timing; add constraints in\n// your own flow.\n"
    );
    if fets {
        s += "//\n// This design contains transistors, written as tranif1 (NMOS) and\n// tranif0 (PMOS) switches. Simulators such as Icarus Verilog run them;\n// FPGA synthesis does not accept switch-level logic.\n";
    }
    s += "\n`timescale 1ns / 1ps\n\n";
    s += &top_text;
    for (_, t) in o.customs.iter().rev() {
        s += "\n";
        s += t;
    }
    for id in &o.library {
        s += "\n";
        s += &library_module(id).expect("library component");
    }
    for data in o.roms.values() {
        s += "\n";
        s += &rom_module(data);
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    //! The exported Verilog is checked against the engine itself: Icarus
    //! Verilog runs a generated test bench whose expected values come from
    //! the engine's models. These tests skip (with a note) when `iverilog`
    //! is not installed.

    use super::*;
    use crate::circuit::Part;
    use crate::components::{catalog, from_num, Model};
    use crate::signal::Sig;

    fn have_iverilog() -> bool {
        let ok = std::process::Command::new("iverilog")
            .arg("-V")
            .output()
            .is_ok();
        if !ok {
            eprintln!("iverilog not found: skipping the Verilog simulation check");
        }
        ok
    }

    /// Compile and run `src` in Icarus; return what it printed.
    fn run(src: &str, tag: &str) -> String {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let k = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("m1-verilog-{}-{k}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let v = dir.join("t.v");
        let out = dir.join("t.vvp");
        std::fs::write(&v, src).unwrap();
        let c = std::process::Command::new("iverilog")
            .args(["-g2005", "-o"])
            .arg(&out)
            .arg(&v)
            .output()
            .unwrap();
        assert!(
            c.status.success(),
            "{tag}: iverilog rejected the export:\n{}\n{src}",
            String::from_utf8_lossy(&c.stderr)
        );
        let r = std::process::Command::new("vvp")
            .arg(&out)
            .output()
            .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        String::from_utf8_lossy(&r.stdout).into_owned()
    }

    fn lit(s: &[Sig]) -> String {
        s.iter()
            .rev()
            .map(|x| match x {
                Sig::L => '0',
                Sig::H => '1',
                Sig::Z => 'z',
                Sig::X => 'x',
            })
            .collect()
    }

    struct Rng(u64);
    impl Rng {
        fn bit(&mut self) -> bool {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0 >> 40 & 1 == 1
        }
    }

    /// Input vectors for a component: every combination up to 10 inputs,
    /// otherwise random ones. Clocked parts get data with CLK low, then the
    /// rising edge, then a falling edge with data held.
    fn vectors(ins: &[&str], seed: u64) -> Vec<Vec<bool>> {
        let n = ins.len();
        let clk = ins.iter().position(|s| *s == "CLK");
        let mut v = Vec::new();
        if clk.is_none() && n <= 10 {
            return (0..1u32 << n).map(|k| from_num(k, n)).collect();
        }
        let mut r = Rng(seed | 1);
        let mut cur = vec![false; n];
        for _ in 0..400 {
            for (k, b) in cur.iter_mut().enumerate() {
                if Some(k) != clk {
                    *b = r.bit();
                }
            }
            v.push(cur.clone());
            if let Some(c) = clk {
                cur[c] = true;
                v.push(cur.clone());
                cur[c] = false;
                v.push(cur.clone());
            }
        }
        v
    }

    /// A Verilog test bench that drives `vecs` into `module` and prints
    /// FAIL for any output that differs from the engine's model.
    fn bench(
        module: &str,
        ins: &[&str],
        outs: &[&str],
        m: &mut dyn Model,
        vecs: &[Vec<bool>],
    ) -> String {
        let mut s = String::from("module tb;\n");
        let (ri, wo) = (ins.len(), outs.len());
        writeln!(
            s,
            "  reg [{}:0] i;\n  wire [{}:0] o;",
            ri.max(1) - 1,
            wo - 1
        )
        .ok();
        let conns: Vec<String> = ins
            .iter()
            .enumerate()
            .map(|(k, n)| format!(".{n}(i[{k}])"))
            .chain(
                outs.iter()
                    .enumerate()
                    .map(|(k, n)| format!(".{n}(o[{k}])")),
            )
            .collect();
        writeln!(s, "  {module} dut ({});\n  initial begin", conns.join(", ")).ok();
        for (t, v) in vecs.iter().enumerate() {
            let want = m.eval(&crate::components::sigs(v));
            let iv: String = v.iter().rev().map(|&b| if b { '1' } else { '0' }).collect();
            writeln!(
                s,
                "    i = {ri}'b{iv}; #1 if (o !== {wo}'b{w}) $display(\"FAIL step {t}: in {iv} want {w} got %b\", o);",
                w = lit(&want)
            )
            .ok();
        }
        s += "    $display(\"DONE\");\n  end\nendmodule\n";
        s
    }

    #[test]
    fn every_library_module_matches_its_engine_model() {
        if !have_iverilog() {
            return;
        }
        for (n, c) in catalog().iter().enumerate() {
            let id = c.spec.id;
            let module = library_module(id).unwrap_or_else(|| panic!("no Verilog for {id}"));
            let vecs = vectors(c.spec.inputs, 0x1234_5678 + n as u64);
            let tb = bench(
                &format!("m1_{id}"),
                c.spec.inputs,
                c.spec.outputs,
                &mut *(c.make)(),
                &vecs,
            );
            let out = run(&format!("{module}\n{tb}"), id);
            assert!(
                out.contains("DONE") && !out.contains("FAIL"),
                "{id}:\n{out}"
            );
        }
    }

    #[test]
    fn the_cpu_runs_a_program_from_rom() {
        if !have_iverilog() {
            return;
        }
        // CPU + ROM computer: LDI 5, ADI 7, STA 2, ADD 2, OUT, HLT => 24.
        let mut c = Circuit::default();
        let mut rom = Part::new("rom", PartKind::Rom);
        rom.data = Some(vec![0x15, 0x27, 0x62, 0x32, 0x90, 0xF0]);
        c.parts.push(rom);
        c.parts.push(Part::chip("cpu", "cpu"));
        c.parts.push(Part::pin("clk", PartKind::In, 0, Some("CLK")));
        c.parts
            .push(Part::pin("halt", PartKind::Out, 8, Some("HALT")));
        for k in 0..4 {
            c.connect(("cpu", 9 + k), ("rom", k)); // PC -> A
        }
        for k in 0..8 {
            c.connect(("rom", 4 + k), ("cpu", k)); // D -> I
            let id = format!("o{k}");
            c.parts
                .push(Part::pin(&id, PartKind::Out, k, Some(&format!("OUT{k}"))));
            c.connect(("cpu", 13 + k), (&id, 0));
        }
        c.connect(("clk", 0), ("cpu", 8));
        c.connect(("cpu", 21), ("halt", 0));
        let v = verilog(&c, &Library::open(), "computer").unwrap();
        let tb = "module tb;\n  reg clk = 0;\n  wire [7:0] o;\n  wire h;\n  computer dut (.CLK(clk), .OUT0(o[0]), .OUT1(o[1]), .OUT2(o[2]), .OUT3(o[3]), .OUT4(o[4]), .OUT5(o[5]), .OUT6(o[6]), .OUT7(o[7]), .HALT(h));\n  integer k;\n  initial begin\n    for (k = 0; k < 20; k = k + 1) begin #1 clk = 1; #1 clk = 0; end\n    $display(\"OUT=%0d HALT=%b\", o, h);\n  end\nendmodule\n";
        let out = run(&format!("{v}\n{tb}"), "computer");
        assert!(out.contains("OUT=24 HALT=1"), "{out}");
    }

    #[test]
    fn a_hand_built_cmos_nand_simulates_as_switches() {
        if !have_iverilog() {
            return;
        }
        let mut c = Circuit::default();
        for (id, k) in [("a", 0), ("b", 1)] {
            c.parts
                .push(Part::pin(id, PartKind::In, k, Some(&id.to_uppercase())));
        }
        c.parts.push(Part::pin("y", PartKind::Out, 0, Some("Y")));
        c.parts.push(Part::new("v", PartKind::Vcc));
        c.parts.push(Part::new("g", PartKind::Gnd));
        for t in ["p1", "p2"] {
            c.parts.push(Part::new(t, PartKind::Pmos));
        }
        for t in ["n1", "n2"] {
            c.parts.push(Part::new(t, PartKind::Nmos));
        }
        for (t, i) in [("p1", "a"), ("n1", "a"), ("p2", "b"), ("n2", "b")] {
            c.connect((i, 0), (t, 0));
        }
        c.connect(("v", 0), ("p1", 1));
        c.connect(("v", 0), ("p2", 1));
        c.connect(("p1", 2), ("y", 0));
        c.connect(("p2", 2), ("y", 0));
        c.connect(("y", 0), ("n1", 1));
        c.connect(("n1", 2), ("n2", 1));
        c.connect(("n2", 2), ("g", 0));
        let v = verilog(&c, &Library::open(), "cmos nand").unwrap();
        assert!(v.contains("tranif1") && v.contains("tranif0") && v.contains("FPGA"));
        let mut nand = components::model("nand").unwrap();
        let tb = bench(
            "cmos_nand",
            &["A", "B"],
            &["Y"],
            &mut *nand,
            &vectors(&["A", "B"], 1),
        );
        let out = run(&format!("{v}\n{tb}"), "nand");
        assert!(out.contains("DONE") && !out.contains("FAIL"), "{out}");
    }

    #[test]
    fn custom_components_become_modules_and_names_are_made_legal() {
        use crate::custom::CustomDef;
        let mut inner = Circuit::default();
        inner
            .parts
            .push(Part::pin("a", PartKind::In, 0, Some("input")));
        inner
            .parts
            .push(Part::pin("y", PartKind::Out, 0, Some("2out")));
        inner.parts.push(Part::chip("n", "not"));
        inner.connect(("a", 0), ("n", 0));
        inner.connect(("n", 1), ("y", 0));
        let mut customs = std::collections::BTreeMap::new();
        let def =
            CustomDef::from_design("custom:inv", "inv", "INV", inner, &Library::open()).unwrap();
        customs.insert(def.id.clone(), def);
        let lib = Library::new(&customs, &|_| Ok(()));
        let mut top = Circuit::default();
        top.parts.push(Part::pin("a", PartKind::In, 0, Some("A")));
        top.parts.push(Part::pin("y", PartKind::Out, 0, Some("Y")));
        top.parts.push(Part::chip("u1", "custom:inv"));
        top.parts.push(Part::chip("u2", "custom:inv"));
        top.connect(("a", 0), ("u1", 0));
        top.connect(("u1", 1), ("u2", 0));
        top.connect(("u2", 1), ("y", 0));
        let v = verilog(&top, &lib, "buffer").unwrap();
        assert_eq!(
            v.matches("module custom_inv").count(),
            1,
            "one module per component"
        );
        assert!(
            v.contains("input  wire input_") && v.contains("output wire p_2out"),
            "{v}"
        );
        assert!(v.contains("custom_inv u_u1 (.input_(n"), "{v}");
        if have_iverilog() {
            let mut buf = components::Comb {
                n_in: 1,
                n_out: 1,
                f: |b| vec![b[0]],
            };
            let tb = bench("buffer", &["A"], &["Y"], &mut buf, &vectors(&["A"], 1));
            let out = run(&format!("{v}\n{tb}"), "custom");
            assert!(out.contains("DONE") && !out.contains("FAIL"), "{out}");
        }
    }
}
