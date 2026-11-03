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

Open http://127.0.0.1:7878.

| Env var | Default | Effect |
|---|---|---|
| `PORT` | `7878` | HTTP port |
| `MARK_ONE_SAVE` | `mark_one_save.json` | Progress and drafts |
| `UNLOCK_ALL` | off | `1` opens every Mark and chip (sandbox mode) |

## Project layout

Mark One is a Cargo workspace with three crates and a web view. Each layer depends only on the ones below it.

```
crates/
  engine/          mark-one-engine: the simulator and component library. No I/O, no game rules.
    src/
      signal.rs        the four values a net carries: 0, 1, Z, X
      circuit.rs       circuits as data: parts and wires
      sim/             switch-level transistors plus behavioural chips
        netlist.rs       circuit -> nets, drivers, transistors, chip instances
        switch.rs        resolving nets through conducting transistors
        union_find.rs
      components/      one file per component: spec, model, tests
        clocked.rs       shared master-slave edge logic
        testkit.rs       test helpers, including transistor-level proofs
        timed.rs         timing mode: event-driven simulation with gate delays
      workbench.rs     open circuits and the library of parts they may use (Experiment)
      custom.rs        designs saved and reused as components
      timing.rs        gate delays and timing arcs of every component
      analysis/        truth tables, test benches, equivalence checks, critical paths
      export/          Verilog and KiCad netlists
  course/          mark-one-course: the learning path on top of the engine
    src/
      level.rs         what a Mark is
      catalog/         every Mark, one file per chapter
      bench.rs         input sequences diagnostics drive
      programs.rs      reference programs and diagnostic program sets
      verify/          diagnostics: steps, CPU, computer, software
      progress.rs      unlocks, costs, the save file
      state.rs         the JSON the client draws from
    tests/solutions.rs reference solutions for all 42 Marks at par
  server/          mark-one-server: the `mark-one` binary
    build.rs           embeds web/ into the binary
    src/
      config.rs, app.rs
      routes/          one module per API resource
      assets.rs        serves the embedded web view
web/               the view: plain ES modules and CSS, no build step
  index.html
  styles/            one stylesheet per area of the screen
  js/
    main.js          entry point
    core/            state, API calls, the simulation session
    board/           geometry, wire routing, rendering, view, editing, input
    panels/          marks, mission, spec, dock, flags, diagnostics,
                     waveform (wave.js), program debugger (debugger.js)
    modes/free.js    Experiment mode
    modes/tools.js   Experiment tools: bench, equivalence, timing, export
    panels/library.js the component library
    machine/clock.js clock, pulse, run to halt
    isa/             assembler, ROM editor, computer template
    lesson/          lesson player and demos
    content/         lessons, lore and part descriptions, as data
    ui/              log, reactor, scanner, boot, toolbar, dialogs, printing
```

The engine is a library, so other front ends can drive it directly: a CLI, a native app, or a WebAssembly build for the browser. The web files are embedded at build time, so `cargo build --release` produces one self-contained executable.

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

Each Mark has a reference design built from the components before it. The reference designs are what the tests use to prove every Mark can be solved. Their transistor counts appear in the component library and on printed schematics as plain information. They are not a score, and hand-optimised static CMOS would be roughly 1.5 to 2 times smaller (XOR in 12 transistors, a mirror full adder in 28).

## Modes

The left panel has three sections.

### Missions

The 42 Marks, in order. Each asks for one component. Passing diagnostics unlocks the next Mark in place, and adds that component to your fabricator for later Marks. A Mission only accepts the parts and chips it has unlocked, never custom components.

### Experiment

Choose **Bench** to build anything, from the parts Missions have unlocked:

| Part | Unlocks when you pass |
|---|---|
| VCC, GND, INPUT, OUTPUT | Always available |
| NMOS, PMOS | Mk 2 (end of Charge) |
| Each library chip | Its own Mark |
| ROM, DISPLAY | Mk 36 (the CPU) |

The server enforces this, including for parts inside custom components.

- Add INPUT and OUTPUT pins as you need them. Double-click a pin to rename it, and name an input `CLK` to drive it with the clock buttons.
- **Generate truth table** applies every input combination (up to 12 inputs) to a freshly powered copy of the circuit. It flags:
  - shorts and oscillation;
  - races, where the result depends on evaluation order;
  - degraded outputs;
  - floating or unknown outputs;
  - memory, where the outputs depend on what came before.
- Designs are saved by name. Use the selector, New and Delete to manage them.
- The **Analysis** tabs in the right panel hold the engineering tools: Table, Bench, Equiv, Timing and Export (below).
- **Save as component** turns the current design into a reusable chip. Its INPUT and OUTPUT pins become the chip's pins, and its cost is whatever is inside it. Custom components can contain other custom components, but never themselves. Saving again under the same name replaces the component everywhere it is used.

### Components

**Parts** opens the component library. It shows every component and whether it is unlocked (and by which Mark), its pins and transistor count, the basic parts, and your own components, which you can delete here. A component that another custom component uses cannot be deleted until that one is.

## Engineering tools

### Waveform

Select a wire or a part and press **A** to probe it: a wire probes the pin it starts from, a chip probes its outputs. **W** shows or hides the waveform panel above the fabricator. Every simulation step adds a sample, so toggling inputs or pulsing the clock draws timing traces like a logic analyzer: gold is 1, blue is 0, a dashed line is floating (Z) and red is unknown (X). Faint vertical lines mark rising clock edges. Probed pins have a gold ring on the board. **VCD** saves the traces as a Value Change Dump for GTKWave or any waveform viewer. Probes are kept per board.

### Test benches (Experiment)

The **Bench** tab holds the expected behaviour of a design as text, and reruns it after every change. The mission panel shows whether it passes from any tab.

```
# a 4-bit adder
A[3:0] B[3:0] CIN | S[3:0] COUT
3      4      0   | 7      0
0xF    1      0   | 0      1
```

- The header names input pins, `|`, then output pins. `S[3:0]` is a bus of pins S3 down to S0 and takes a number (decimal, `0x` hex, `0b` binary).
- Rows run in order on one powered-up circuit, so state carries over and a bench can check counters, registers and whole computers.
- Inputs: `0`, `1`, a number, `-` to keep the last value, `P` to pulse a one-bit input (0, 1, 0). Outputs: `0`, `1`, a number, `X`, `Z`, or `-` to skip.
- A line `reset` powers the circuit up again.
- Every bench runs in both chip evaluation orders, like diagnostics; a row that passes only one way is a race.
- **Record outputs** replaces each expected output with what the circuit does now; **Full table** writes a row for every input combination (up to 12 inputs). Both are only as right as the circuit is, so check what they record.

### Equivalence (Experiment)

The **Equiv** tab compares the design with another design or with a component (library or your own), matching pins by name. Designs with no memory and up to 16 inputs are **proved** equivalent over every input combination, in both evaluation orders. Otherwise the check is **bounded**: 200 seeded random sequences of 64 steps from power-up, changing data while CLK is low and while it is high (which tells a latch from a flip-flop). Bounded agreement is evidence, not proof, and the result says so. A difference comes with the shortest sequence from power-up that shows it.

### Program debugger

Any board with a ROM shows a **Debugger** under the program editor:
- **Step** (Space) executes one instruction; **Run** (H) runs to HLT or a breakpoint; **Reset** (X) restarts from address 0.
- Click an address in the ROM listing to set a breakpoint (red dot). Run stops before the instruction at that address executes.
- It shows the next instruction, and the registers and memory inside every chip that stores state (the CPU's PC, ACC, OUT and RAM; a register's or RAM chip's contents; inside your own components, named `part/REG`). Values that changed on the last step are gold.

### Timing mode

**T** switches between zero-delay and timed simulation, for any board. In timing mode every gate takes time:
- The unit is one gate delay: one CMOS stage (an inverter, NAND or NOR). A transistor's channel responds one unit after its gate. A library chip takes the longest path through the reference design that fabricated it (an 8-bit adder is 80; the CPU is 44, because its slow ALU path ends at a register rather than an output); the course tests recompute every value from the reference designs. Your own components take the critical path of their circuit.
- Delays are inertial, as in Verilog: a pulse shorter than a chip's delay does not get through it; a longer one does. So static hazards show up: a selector built from a NOT and three NANDs glitches low for one delay when SEL falls with both inputs at 1.
- The waveform then has a time axis in gate delays and shows every change inside a step. Signals that changed more than once in one step are marked **glitch**. Loops without registers really oscillate (a three-stage ring toggles every three delays) until a 5,000-delay horizon.
- Edge-triggered chips report **setup** problems: a data input changing within one delay of its clock edge, including at the same moment. Red dashed lines mark them.
- Each new set of inputs arrives two delays after the circuit last moved, as from a test bench that waits for settling.

The **Timing** tab (Experiment) finds the static critical path: arrival time at every output and the chain of parts on the longest path. Registers start new paths at CLK and end paths at their data inputs; a loop with no register (a latch built from gates) is cut and reported.

Diagnostics, truth tables, benches and equivalence checks always run zero-delay.

### Export (Experiment)

The **Export** tab writes the design out, and shows whether it passes its bench first:
- **Verilog (.v)**: a structural top module, one wire per net and one instance per chip. Library chips become behavioural modules (`m1_add4` and so on) that the engine tests run in Icarus Verilog against the engine's own models, vector by vector. Custom components become modules of their own; each ROM image becomes a lookup module. Transistors become `tranif1` (NMOS) and `tranif0` (PMOS) switches: they simulate, but FPGA synthesis does not accept switch-level logic, so build from chips for an FPGA.
- **KiCad netlist (.net)**: the s-expression netlist pcbnew imports. Transistors are Q1, Q2 ... (pins 1 G, 2 and 3 the channel), chips and ROMs U1, U2 ... (pins numbered from 1 in pin order, inputs first), displays DS1 .... VCC, GND and your INPUT and OUTPUT pins name their nets. Parts have no footprints: a library chip is a function, so choose a real part for each (a 4-bit adder might become a 74HC283).

## Printing

**Print** (P) produces a schematic sheet in landscape:
- a title block with the design or Mark, mode, transistor count and date;
- the circuit drawn as a standard schematic, in black on white and fitted to the page:
  - gates in the ANSI/IEEE Std 91 distinctive shapes (AND, OR, XOR, NOT, NAND, NOR, with inversion bubbles), output level with the middle of the inputs;
  - transistors as enhancement-mode MOSFET symbols (insulated gate, broken channel, body arrow in for NMOS and out for PMOS), labelled G, D and S;
  - every other chip, ROM and display as a datasheet-style rectangle with pin names inside, pin numbers on the leads and a clock wedge on CLK;
  - VCC and GND symbols, port flags for the design's pins, and junction dots wherever three or more branches of a net meet;
  - reference designators (Q1, U1, DS1 ...) numbered as in the KiCad export;
- a bill of materials with the reference designators;
- the pins;
- the truth table (Experiment) or the spec (Missions).

Use your browser's print dialog to print it or save it as PDF.

**Clear** (C) asks first, and says how many parts and wires it will remove.

## Lessons

Each Mark opens with a one-card lesson the first time you reach it. Reopen it with the Lesson button or the L key. A lesson explains what the component is and lets you try the finished component on a live bench. It never says how to build it: working that out is the exercise. Passing diagnostics unlocks the next Mark in place, without leaving the page.

## The instruction set

One byte per instruction: opcode in the high nibble, operand `n` in the low nibble. 16 bytes of ROM, 16 bytes of RAM, one 8-bit accumulator.

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

Opcodes A to E are reserved and execute as NOP. The operand bits of NOP, OUT and HLT are ignored.

- **HALT:** the CPU's HALT output is decoded straight from the opcode, so it goes high as soon as the PC points at an HLT. The clock may keep running: the PC holds and HLT re-executes harmlessly.
- **Harvard layout:** the program sits in its own ROM and data in RAM, so a program cannot read or modify its own code. Machines like EDSAC and the Manchester Baby kept both in one memory, a von Neumann layout.

Select a ROM on the board to write assembly. Labels end with a colon, and comments start with `;` or `#`. Numbers can be decimal, `0x` hex or `0b` binary. A software Mark takes one ROM. Load with the Load button or Ctrl+Enter. H (run to halt) clocks the computer until HLT.

## Controls

Everything on the board is driven from the keyboard. The help bar along the bottom lists the keys that apply to the current board.

| Key | Does |
|---|---|
| D | Run diagnostics (Missions) |
| R, Shift+R | Rotate the selected part clockwise, anticlockwise |
| Del, Backspace | Delete the selected part or wire |
| F | Recenter the view on the circuit |
| Space | Pulse: one rising clock edge (boards with CLK) |
| K | Clock speed: off, 1, 4, 12 Hz (boards with CLK) |
| H | Run to halt or a breakpoint (boards with HALT) |
| A | Probe the selected wire or part in the waveform |
| W | Show or hide the waveform |
| T | Timing mode on or off |
| P | Print the schematic |
| L | Open the lesson (Missions) |
| X | Reset the simulation and its stored state |
| C | Clear the board, after a warning |
| [ | Hide or show the left panel (also the tab on its edge) |
| Esc | Cancel the wire being drawn, deselect |

With the mouse:
- **Wiring:** click a pin, then another pin, to wire them.
- **Moving and inputs:** drag parts. They snap to the 20-unit grid, so every pin sits on a grid point and wires run along grid lines. Click an input pin part to toggle it.
- **Removing:** right-click a part or wire to remove it.
- **Viewing:** drag the floor to pan, scroll to zoom.

Wire colours: gold with moving dashes is 1, steel is 0, dashed is floating (Z), red is unknown or shorted (X).

## Component library

`crates/engine/src/components/` holds one file per component. Each file contains:

- the pin specification,
- the behavioural model,
- tests that check the model against an independent statement of the spec, such as a literal truth table, column-by-column addition, or wrapping `u8` arithmetic.

NOT, NAND, NOR and the SR latch are also built from transistors in the switch-level simulator (`testkit::Fab`). They are checked on every input against their models: correct values, no shorts, no floating outputs. `crates/course/tests/solutions.rs` then builds every higher component from lower ones and checks it against its model. Together these tests trace the chain from MOSFETs up to the CPU.

The learning path (`crates/course`) is one way through the library. A test checks that each Mark's pins and par match its component.

## How the simulator works

`crates/engine/src/sim/` is a mixed-level simulator.

- **Transistors are switch-level**, in the style of IRSIM:
  - A conducting channel (NMOS gate = 1, PMOS gate = 0) merges two nets, and a merged group takes the value of its drivers. Supply rails, input pins and chip outputs are ideal sources.
  - A transistor whose gate is X or Z might conduct or not. The network is solved both ways, and nets that differ become X.
  - If both 1 and 0 drive a group, that is a short.
  - **Strength:** a 1 that reaches a net only through an NMOS, or a 0 only through a PMOS, is degraded (about one threshold voltage short of the rail). A degraded value can drive a gate, but a level output must be fully driven, so pass-transistor tricks fail diagnostics.
  - If nothing drives a net, it keeps its stored charge but counts as floating, so a half-built gate fails instead of passing on leftover charge. Stored charge leaks to X after a few settles, so dynamic storage must be refreshed.
- **Chips are behavioural models** (`crates/engine/src/components/`) with three-valued logic. They are evaluated in Gauss-Seidel order until no output changes, which lets cross-coupled latches settle.
- **Clocked chips use master-slave semantics:** inputs are sampled while CLK is 0 and committed on the rising edge.

Because a verified chip runs as its model, the simulation stays small at every level, even though the course's reference CPU is about 12,000 transistors underneath (hand-optimised CMOS would be fewer).

**Diagnostics** (`crates/course/src/verify/`):
- **Combinational Marks:** tested exhaustively up to 12 inputs, otherwise with 3,000 random vectors plus edge cases.
- **Sequential Marks:** get long random sequences, then long holds in each control state, so a design that only stores charge dynamically fails.
- **Every Mark** runs its bench twice, with the chips evaluated first to last and last to first. A design that passes only one way depends on a race or a glitch.
- **The CPU:** diagnostics act as program memory, feeding the instruction at its PC for 16 programs of 48 cycles each.
- **The computer:** diagnostics load test programs into its ROM.
- **Software Marks:** run the ROM program on the board's own computer, cycle by cycle beside the reference CPU. The OUT sequence must match the goal, and the board must agree with the reference on every cycle.

## Limitations

Mark One checks what a circuit computes, not when. Before taking a design to hardware, know what it does not model:

- **Timing is in gate delays, not nanoseconds.** Diagnostics are zero-delay. Timing mode adds unit gate delays, one delay per chip for all its outputs (pessimistic for its fast outputs), a one-delay setup window and no hold time, metastability or wire delay. It finds hazards, races, oscillation and critical-path depth; it is not timing closure for a real process.
- **Two strengths only.** Driven, degraded and stored charge. Ratioed logic (pseudo-NMOS, keepers, level restorers) is reported as a short, and charge sharing between different stored values gives X rather than a voltage.
- **Leakage is a settle count.** Stored charge lasts a fixed number of settles, not a time.
- **Power-up.** Library flip-flops, registers, the PC, the accumulator and RAM power up at 0. Real ones power up unknown, and PC4 and ACC8 have no reset pin. Circuits you build from transistors or gates power up X.
- **Chips resolve unknowns exactly.** A chip with an X input gives a known output whenever every value of that input agrees (like Verilog's `?:`), which can be more optimistic than the same circuit built from gates. Past six unknown inputs a chip reports X.
- **An X clock is ignored** by library flip-flops rather than corrupting their state.
- **Program ROM is separate** from data RAM (Harvard), and RAM writes are synchronous: on real asynchronous SRAM you would gate WE with the clock.

## API

| Method | Path | Body |
|---|---|---|
| GET | `/api/state` | Marks, status, parts bin, truth tables, goals |
| GET, PUT | `/api/levels/{id}/circuit` | Draft circuit |
| POST | `/api/sim` | `{level, circuit, inputs, reset, cycles?}` returns pin values and outputs. `cycles` runs that many clock cycles, stopping at HALT. |
| POST | `/api/demo` | `{level, inputs, reset}` evaluates the finished component for lessons |
| POST | `/api/levels/{id}/verify` | Circuit; runs diagnostics and saves progress on a pass |
| GET | `/api/designs` | Names of saved free-build designs |
| GET, PUT, DELETE | `/api/designs/{name}` | One free-build design |
| POST | `/api/table` | Circuit; truth table over every input combination, with Experiment unlocks and custom components |
| POST | `/api/custom` | `{name, label, circuit}`; saves a design as a custom component |
| POST | `/api/bench` | `{circuit, bench}`; runs a test bench |
| POST | `/api/bench/record` | `{circuit, bench}`; fills in expected outputs (empty bench: every input combination) |
| GET, PUT | `/api/designs/{name}/bench` | A design's bench text |
| POST | `/api/equiv` | `{circuit, against: {design: name} \| {component: id}}` |
| POST | `/api/timing` | Circuit; static critical path |
| POST | `/api/export` | `{circuit, name, format: "verilog" \| "kicad"}`; returns `{text, filename}` |
| DELETE | `/api/custom/{id}` | Deletes a custom component that nothing else uses |

`/api/sim` also takes `probes` (`[{part, pin}]`, sampled after every settle and returned as `trace`), `timed` (timing mode; `trace` then carries event times and `timing` the setup violations and glitches), `inspect` (returns `internals`: registers and memories per chip) and `breakpoints` (ROM addresses; a `cycles` run stops before fetching one and reports `stopped`).

`/api/sim` with `level: "free"` simulates an Experiment circuit. Its pins come from its INPUT and OUTPUT parts, and it may use what the player has unlocked plus their custom components.

## Tests

```
cargo test --release --workspace
```

Each module carries its own unit tests, and each component's tests live in its own file. `crates/course/tests/solutions.rs` builds a reference solution for all 42 Marks, up to the full CPU and all four programs, and asserts each passes at exactly par. It also checks:
- floating outputs,
- shorts,
- oscillation,
- locked chips,
- a computer without a ROM,
- wrong programs,
- a CPU with a broken jump.

It also recomputes each library component's timing delay from the critical path of its reference design and checks it against `engine/src/timing.rs`. After changing a reference design, run the solutions test with `M1_DELAY_SURVEY=1` and `--nocapture` to print the new values.

The Verilog export tests compile every library module, a CPU and ROM computer, a transistor-level NAND and a nested custom component with [Icarus Verilog](https://steveicarus.github.io/iverilog/) and check them against the engine's models. They skip themselves when `iverilog` is not installed (`brew install icarus-verilog` on macOS).
