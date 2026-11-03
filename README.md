<p align="center">
  <img src="docs/images/banner.png" alt="Mark One" width="100%">
</p>

<p align="center">
  <img alt="Rust 1.85+" src="https://img.shields.io/badge/rust-1.85%2B-ffb84d?style=flat-square&logo=rust&logoColor=white&labelColor=071423">
  <img alt="No build step for the web view" src="https://img.shields.io/badge/web%20view-plain%20ES%20modules-5ad7ff?style=flat-square&labelColor=071423">
  <img alt="Single binary" src="https://img.shields.io/badge/ships%20as-one%20binary-63f5b0?style=flat-square&labelColor=071423">
</p>

# Mark One

A factory game about building a computer from charge up. You start with two lines, one charged (VCC = 1) and one drained (GND = 0). Every circuit you build and verify comes off the line as a chip, and that chip goes into the fabricator for the next Mark. Forty-two Marks later you have a working computer, built from your own transistors, running programs you wrote.

Underneath the game is a serious engine: a mixed-level simulator (switch-level MOSFETs up to an 8-bit CPU) with a waveform view, test benches, equivalence checking, a timing mode with gate delays, a program debugger, and export to Verilog and KiCad.

## A tour

<table>
  <tr>
    <td width="50%"><img src="docs/images/mission.png" alt="Mark 4: a CMOS NAND gate built from four transistors, with diagnostics passed"></td>
    <td width="50%"><img src="docs/images/timing-glitch.png" alt="Timing mode: a selector built from gates glitches when SEL falls, shown in the waveform"></td>
  </tr>
  <tr>
    <td><b>Missions.</b> Mk 4 asks for a NAND gate from NMOS and PMOS transistors. Diagnostics check every input combination, then NAND becomes a chip for the next Mark.</td>
    <td><b>Timing mode and the waveform.</b> A selector built from a NOT and three NANDs glitches low for one gate delay when SEL falls. Its test bench, which runs zero-delay, still passes.</td>
  </tr>
  <tr>
    <td><img src="docs/images/debugger.png" alt="The program debugger stopped at a breakpoint, showing PC, ACC and RAM"></td>
    <td><img src="docs/images/schematic.png" alt="A printed schematic of a full adder with standard gate symbols"></td>
  </tr>
  <tr>
    <td><b>Program debugger.</b> A CPU and ROM computer stopped at a breakpoint on address 4, with PC, ACC, OUT and all 16 bytes of RAM on view.</td>
    <td><b>Printed schematics.</b> Standard gate, MOSFET and IC symbols, junction dots, reference designators and a bill of materials.</td>
  </tr>
</table>

## Run it

Requires Rust 1.85+.

```
cargo run --release
```

Open http://127.0.0.1:7878. Set `PORT` to change the port, `MARK_ONE_SAVE` to choose the save file (default `mark_one_save.json`), and `UNLOCK_ALL=1` to open every Mark and chip.

## Progression

| Chapter | Marks | Fabricates |
|---|---|---|
| Charge | 1-2: Live wire, Relay | |
| Transistors | 3-5: Inverter, NAND, NOR (NMOS and PMOS only) | NOT, NAND, NOR |
| Gates | 6-9: AND, OR, XOR, Selector | AND, OR, XOR, MUX |
| Arithmetic | 10-11: Half adder, Full adder | HA, FA |
| Wide buses | 12-17: adders, flippers, zero detector, ALU | ADD4, ADD8, FLIP4, FLIP8, ZERO8, ALU8 |
| Memory | 18-23: latches, flip-flop, registers | SR, DLATCH, DFF, REG1, REG4, REG8 |
| The machine | 24: Accumulator | ACC8 |
| Counting | 25-28: incrementer, selectors, program counter | INC4, MUX4, MUX8, PC4 |
| Decoding | 29-30: 2-to-4 and 4-to-16 decoders | DEC2, DEC4 |
| Memory banks | 31-34: 2, 4, 8 and 16 bytes of RAM | RAM2, RAM4, RAM8, RAM16 |
| The CPU | 35-36: control unit, CPU | CTRL, CPU |
| The computer | 37: CPU + ROM + display | unlocks ROM and DISPLAY |
| Software | 38-41: The answer, Countdown, Powers of two, Times table | |
| Workshop | 42: free build, no diagnostics | |

Every Mark has a reference design built from the components before it, and the tests use them to prove each Mark can be solved.

## Modes

- **Missions:** the 42 Marks in order. Each opens with a one-card lesson that explains the component but never how to build it. Passing diagnostics unlocks the next Mark and adds the component to your fabricator.
- **Experiment (Bench):** build anything from the parts you have unlocked, with named INPUT and OUTPUT pins (name one `CLK` to clock it). Designs are saved by name, and **Save as component** turns a design into a reusable chip.
- **Parts:** the component library: what is unlocked, pins, transistor counts, and your own components.

## Engineering tools

- **Waveform:** select a wire or part and press **A** to probe it; **W** shows the traces. **VCD** saves them for GTKWave.
- **Test benches:** the Bench tab holds a design's expected behaviour as text and reruns it after every change. Rows run in order, so state carries over. `S[3:0]` is a bus; `P` pulses an input; `-` keeps or skips a value; `reset` powers up again.

  ```
  A[3:0] B[3:0] CIN | S[3:0] COUT
  3      4      0   | 7      0
  0xF    1      0   | 0      1
  ```

- **Equivalence:** compare a design with another design or a component, pins matched by name. Combinational designs with up to 16 inputs are proved over every input; anything else gets a bounded random check, labelled as such.
- **Program debugger:** on any board with a ROM: Step, Run to HLT or a breakpoint (click an address in the listing), and live PC, ACC, OUT and RAM.
- **Timing mode (T):** simulates with gate delays, so glitches, oscillation and setup problems show in the waveform. The Timing tab finds the critical path. Each library chip's delay is the longest path through its reference design.
- **Export:** Verilog (checked against the engine in Icarus Verilog) and KiCad netlists. Transistors export as switch primitives, which simulate but do not synthesise for an FPGA.
- **Print (P):** a landscape schematic sheet with standard gate, MOSFET and IC symbols, a bill of materials and the truth table.

## The instruction set

One byte per instruction: opcode in the high nibble, operand `n` in the low nibble. 16 bytes of ROM, 16 bytes of RAM, one 8-bit accumulator. Program and data memory are separate (Harvard).

| Op | Hex | Does |
|---|---|---|
| NOP | 00 | Nothing |
| LDI n | 1n | ACC = n |
| ADI n | 2n | ACC = ACC + n |
| ADD n | 3n | ACC = ACC + RAM[n] |
| SUB n | 4n | ACC = ACC - RAM[n] |
| LDA n | 5n | ACC = RAM[n] |
| STA n | 6n | RAM[n] = ACC |
| JMP n | 7n | PC = n |
| JZ n | 8n | If ACC is 0, PC = n |
| OUT | 90 | OUT = ACC |
| HLT | F0 | Stop |

Opcodes A to E execute as NOP. Select a ROM to write assembly: labels end with a colon, comments start with `;`, and Ctrl+Enter loads it.

## Controls

The help bar at the bottom lists the keys that work on the current board.

| Key | Does |
|---|---|
| D | Run diagnostics |
| R, Shift+R | Rotate the selected part |
| Del | Delete the selected part or wire |
| F | Recenter the view |
| Space, K | Pulse the clock; cycle the clock speed |
| H | Run to halt or a breakpoint |
| A, W, T | Probe; show the waveform; timing mode |
| P, L | Print; open the lesson |
| X, C | Reset the simulation; clear the board |
| [, Esc | Hide the left panel; cancel |

Click a pin, then another, to wire them. Drag parts to move them, click an input to toggle it, right-click to delete, scroll to zoom.

## How it works

Mark One is a Cargo workspace. Each layer depends only on the ones below it:

```
crates/engine   the simulator, the component library (one file per component), analysis and export
crates/course   the 42 Marks, diagnostics and progress
crates/server   the mark-one binary: a JSON API, with the web view embedded at build time
web/            the view: plain ES modules and CSS, no build step
```

- **Transistors are switch-level**, in the style of IRSIM, with a degraded strength for a 1 passed by an NMOS or a 0 passed by a PMOS, so pass-transistor tricks fail diagnostics. Undriven charge leaks away after a few settles.
- **Verified chips run as behavioural models** with three-valued logic, so the simulation stays small even though the reference CPU is about 12,000 transistors underneath.
- **Diagnostics** test combinational Marks exhaustively (or with 3,000 random vectors past 12 inputs). They give sequential Marks long random sequences, and run every bench in both chip evaluation orders to catch races.

## Limitations

- Timing is in gate delays, not nanoseconds. Diagnostics are zero-delay, and timing mode is not timing closure for a real process.
- Two strengths only, so ratioed logic reads as a short. Leakage is a settle count, not a time.
- Library memories power up at 0; real ones power up unknown.

## API

| Method | Path | Body |
|---|---|---|
| GET | `/api/state` | Marks, status, parts bin, truth tables, goals |
| GET, PUT | `/api/levels/{id}/circuit` | Draft circuit |
| POST | `/api/levels/{id}/verify` | Circuit; runs diagnostics |
| POST | `/api/sim` | `{level, circuit, inputs, reset, cycles?, probes?, timed?, inspect?, breakpoints?}` |
| POST | `/api/demo` | `{level, inputs, reset}` for lesson benches |
| GET, PUT, DELETE | `/api/designs/{name}` | An Experiment design |
| GET, PUT | `/api/designs/{name}/bench` | A design's test bench |
| POST | `/api/table` | Circuit; truth table |
| POST | `/api/bench`, `/api/bench/record` | `{circuit, bench}`; run or record a bench |
| POST | `/api/equiv` | `{circuit, against: {design} \| {component}}` |
| POST | `/api/timing` | Circuit; critical path |
| POST | `/api/export` | `{circuit, name, format: "verilog" \| "kicad"}` |
| POST, DELETE | `/api/custom`, `/api/custom/{id}` | Save or delete a custom component |

## Tests

```
cargo test --release --workspace
```

Each component is tested against an independent statement of its spec. NOT, NAND, NOR and the SR latch are also proved from transistors. `crates/course/tests/solutions.rs` builds a reference solution for every Mark at par, up to the CPU and its programs, and checks the timing table against those designs. The Verilog tests need [Icarus Verilog](https://steveicarus.github.io/iverilog/) (`brew install icarus-verilog`) and skip themselves without it.
