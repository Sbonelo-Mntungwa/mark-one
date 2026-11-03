<p align="center">
  <img src="docs/images/banner.png" alt="Mark One" width="100%">
</p>

# Mark One

A game about building a computer from transistors up, on top of a real circuit engine written in Rust.

Forty-two Marks take you from a live wire to NAND gates, adders, memory, a CPU and programs running on the computer you built.

<table>
  <tr>
    <td width="50%"><img src="docs/images/mission.png" alt="Mark 4: a CMOS NAND gate built from four transistors, with diagnostics passed"></td>
    <td width="50%"><img src="docs/images/timing-glitch.png" alt="Timing mode: a selector built from gates glitches when SEL falls, shown in the waveform"></td>
  </tr>
  <tr>
    <td><img src="docs/images/debugger.png" alt="The program debugger stopped at a breakpoint, showing PC, ACC and RAM"></td>
    <td><img src="docs/images/schematic.png" alt="A printed schematic of a full adder with standard gate symbols"></td>
  </tr>
</table>

## Quick Start

```bash
# 1. Build and run (Rust 1.85+)
cargo run --release

# 2. Open the app
open http://127.0.0.1:7878

# 3. Run the tests
cargo test --release --workspace
```

## What's Included

| Feature | What it does |
|---------|--------------|
| Missions | 42 Marks, each checked by diagnostics before it unlocks the next |
| Experiment | Build anything from unlocked parts and save designs as components |
| Waveform | Probe wires and watch them as timing traces, export to VCD |
| Test benches | Expected input/output rows, rerun after every change |
| Equivalence | Prove two designs behave the same |
| Timing mode | Gate delays, glitches, setup problems and critical paths |
| Debugger | Step, breakpoints, live registers and RAM |
| Export | Verilog and KiCad netlists |
| Print | Schematic sheets with standard symbols |

## Architecture

- **Switch-level transistors**: NMOS and PMOS simulated as switches, in the style of IRSIM
- **Behavioural chips**: every verified component runs as a tested model
- **One binary**: the web view is embedded at build time, no build step for the front end
- **Tested from MOSFETs up**: every Mark has a reference solution that must pass

## Project Structure

```
mark-one/
├── crates/
│   ├── engine/      # simulator, components, analysis, export
│   ├── course/      # the 42 Marks and diagnostics
│   └── server/      # the mark-one binary and JSON API
├── web/             # the view: plain ES modules and CSS
└── docs/images/     # README pictures
```

## License

[MIT](LICENSE) © 2026 Sbonelo Mntungwa

## Contributing

This repository doesn't take pull requests. To report a bug or suggest a change, please [open an issue](https://github.com/Sbonelo-Mntungwa/mark-one/issues/new). See [CONTRIBUTING.md](CONTRIBUTING.md).
