/* Standard schematic symbols for the printed sheet.

   - Gates use the ANSI/IEEE Std 91 distinctive shapes: the D of AND, the
     shield of OR, the extra back curve of XOR, the triangle of a buffer, and
     an inversion bubble for NOT, NAND and NOR.
   - Transistors are enhancement-mode MOSFETs: insulated gate, broken
     channel, body tied to the source, with the arrow pointing in for NMOS
     and out for PMOS.
   - Every other chip is drawn the way IC datasheets and EDA tools draw them:
     a rectangle with pin names inside, pin numbers on the leads, a clock
     wedge on CLK, the reference designator above and the part below.
   - Supplies are the VCC bar and the GND symbol; design pins are port flags.

   Each symbol is drawn in its part's unrotated frame with the pins exactly
   where `printGeom` puts them, so the wires routed for the sheet meet them. */

import {geom, rotOf} from '../board/geometry.js';
import {esc} from '../core/dom.js';
import {chipDef, lvl} from '../core/state.js';

const GATES = {not: 'not', nand: 'and', and: 'and', nor: 'or', or: 'or', xor: 'xor'};
const INVERTING = new Set(['not', 'nand', 'nor']);

const isGate = p => p.kind === 'chip' && GATES[p.chip];

/* Part shapes on the sheet: as on the board, except that a gate's output
   sits level with the middle of its inputs, as on any schematic. */
export function printGeom(p) {
  const g = geom(p);
  if (!isGate(p)) return g;
  const ins = g.pins.filter(q => q.side === 'l'), mid = ins.reduce((a, q) => a + q.y, 0) / ins.length;
  return {...g, pins: g.pins.map(q => q.side === 'r' ? {...q, y: mid} : q)};
}

/* Reference designators, numbered in board order as in the KiCad export:
   Q for transistors, U for chips and ROMs, DS for displays. */
export function designators(parts) {
  const n = {Q: 0, U: 0, DS: 0}, out = {};
  for (const p of parts) {
    const k = p.kind === 'nmos' || p.kind === 'pmos' ? 'Q' : p.kind === 'chip' || p.kind === 'rom' ? 'U' : p.kind === 'display' ? 'DS' : null;
    if (k) out[p.id] = k + ++n[k];
  }
  return out;
}

/* Text that stays upright however the part is turned. */
function text(x, y, s, r, cls = '', anchor = 'middle') {
  const t = r ? ` transform="rotate(${-90 * r} ${x} ${y})"` : '';
  return `<text class="${cls}" x="${x}" y="${y}" text-anchor="${anchor}" dominant-baseline="central"${t}>${esc(s)}</text>`;
}

/* x of the OR shield's back curve at height y: a quadratic from (x0, top)
   through control (x0 + bulge, mid) to (x0, bot). */
function backX(x0, top, bot, bulge, y) {
  const t = (y - top) / (bot - top);
  return x0 + 2 * t * (1 - t) * bulge;
}

function gate(p, g, r, ref) {
  const kind = GATES[p.chip], inv = INVERTING.has(p.chip);
  const ins = g.pins.filter(q => q.side === 'l'), out = g.pins.find(q => q.side === 'r');
  const cy = out.y, half = Math.max(20, (ins[ins.length - 1].y - ins[0].y) / 2 + 12);
  const top = cy - half, bot = cy + half, L = 30;
  let s = '', tip;
  if (kind === 'not') {
    tip = L + 2 * half * 0.87;
    s += `<path class="sb" d="M${L},${top} L${tip},${cy} L${L},${bot} Z"/>`;
    for (const q of ins) s += `<path class="sl" d="M0,${q.y} H${L}"/>`;
  } else if (kind === 'and') {
    const flat = L + half * 0.6;
    tip = flat + half;
    s += `<path class="sb" d="M${L},${top} H${flat} A${half},${half} 0 0 1 ${flat},${bot} H${L} Z"/>`;
    for (const q of ins) s += `<path class="sl" d="M0,${q.y} H${L}"/>`;
  } else {
    const bulge = half * 0.5, len = half * 2.1;
    tip = L + len;
    s += `<path class="sb" d="M${L},${top} Q${L + len * 0.62},${top} ${tip},${cy} Q${L + len * 0.62},${bot} ${L},${bot} Q${L + bulge},${cy} ${L},${top} Z"/>`;
    const back = kind === 'xor' ? L - 7 : L;
    if (kind === 'xor') s += `<path class="sb open" d="M${back},${top} Q${back + bulge},${cy} ${back},${bot}"/>`;
    for (const q of ins) s += `<path class="sl" d="M0,${q.y} H${backX(back, top, bot, bulge, q.y)}"/>`;
  }
  if (inv) { s += `<circle class="sb" cx="${tip + 4}" cy="${cy}" r="4"/>`; tip += 8; }
  s += `<path class="sl" d="M${tip},${cy} H${g.w}"/>`;
  s += text(L + 4, top - 10, ref, r, 'ref', 'start');
  return s;
}

function mosfet(p, g, r, ref) {
  const P = p.kind === 'pmos';
  // Pins: G (0,40), top (40,0), bottom (40,80). The source is the supply
  // side: the top of a PMOS, the bottom of an NMOS.
  const yd = P ? 56 : 24, ys = P ? 24 : 56;
  let s = `<path class="sl" d="M0,40 H11 M11,22 V58"/>`;
  s += `<path class="sl thick" d="M17,18 V30 M17,34 V46 M17,50 V62"/>`;
  s += `<path class="sl" d="M17,${yd} H40 M17,${ys} H40 M17,40 H40 V${ys} M40,0 V24 M40,56 V80"/>`;
  // Body arrow on the middle segment: into the channel for N, out for P.
  s += `<path class="sf" d="${P ? 'M34,40 L26,36 V44 Z' : 'M18,40 L26,36 V44 Z'}"/>`;
  s += text(48, 32, ref, r, 'ref', 'start') + text(48, 48, P ? 'PMOS' : 'NMOS', r, 'val', 'start');
  s += text(6, 31, 'G', r, 'pnum') + text(34, yd < 40 ? 9 : 71, 'D', r, 'pnum') + text(34, ys < 40 ? 9 : 71, 'S', r, 'pnum');
  return s;
}

/* A datasheet-style block: rectangle, named pins, numbered leads. */
function block(p, g, r, ref, value) {
  const x0 = 20, x1 = g.w - 20, y0 = 24, y1 = g.h - 12;
  let s = `<rect class="sb" x="${x0}" y="${y0}" width="${x1 - x0}" height="${y1 - y0}"/>`;
  g.pins.forEach((q, k) => {
    const left = q.side !== 'r';
    s += `<path class="sl" d="M${left ? 0 : x1},${q.y} H${left ? x0 : g.w}"/>`;
    s += text(left ? 10 : g.w - 10, q.y - 6, String(k + 1), r, 'pnum');
    if (q.n === 'CLK' && left) {
      s += `<path class="sl" d="M${x0},${q.y - 5} L${x0 + 8},${q.y} L${x0},${q.y + 5}"/>`;
      s += text(x0 + 12, q.y, 'CLK', r, 'pin', 'start');
    } else if (q.n) s += text(left ? x0 + 5 : x1 - 5, q.y, q.n, r, 'pin', left ? 'start' : 'end');
  });
  s += text(x0, y0 - 9, ref, r, 'ref', 'start');
  s += text((x0 + x1) / 2, y1 + 11, value, r, 'val');
  return s;
}

function port(p, g, r) {
  const l = lvl(), name = (p.kind === 'in' ? l.inputs : l.outputs)[p.pin] || '?';
  // An input flag points into the circuit (towards its pin on the right);
  // an output flag points away from the circuit, its pin on the left.
  const flag = p.kind === 'in' ? 'M8,10 H58 L68,20 L58,30 H8 Z' : 'M22,10 H72 L82,20 L72,30 H22 Z';
  const lead = p.kind === 'in' ? 'M68,20 H80' : 'M0,20 H22';
  return `<path class="sb" d="${flag}"/><path class="sl" d="${lead}"/>` + text(p.kind === 'in' ? 36 : 50, 20, name, r, 'port');
}

/* One part's symbol, placed and turned as on the board. */
export function symbol(p, refs) {
  const g = printGeom(p), r = rotOf(p), ref = refs[p.id] || '';
  let s;
  if (p.kind === 'vcc') s = `<path class="sl" d="M20,40 V16 M8,16 H32"/>` + text(20, 7, 'VCC', r, 'val');
  else if (p.kind === 'gnd') s = `<path class="sl" d="M20,0 V18 M8,18 H32 M12,24 H28 M16,30 H24"/>`;
  else if (p.kind === 'nmos' || p.kind === 'pmos') s = mosfet(p, g, r, ref);
  else if (p.kind === 'in' || p.kind === 'out') s = port(p, g, r);
  else if (isGate(p)) s = gate(p, g, r, ref);
  else if (p.kind === 'rom') s = block(p, g, r, ref, 'ROM 16x8');
  else if (p.kind === 'display') s = block(p, g, r, ref, 'DISPLAY');
  else { const d = chipDef(p.chip); s = block(p, g, r, ref, d ? d.label : p.chip); }
  return `<g transform="translate(${p.x},${p.y})${r ? ` rotate(${90 * r} ${g.w / 2} ${g.h / 2})` : ''}">${s}</g>`;
}

/* Junction dots: wherever three or more branches of one net meet. A pin
   counts as a branch into its part. */
export function junctions(routes, pinEnds) {
  const byNet = new Map();
  for (const {pts, net} of routes) {
    if (!byNet.has(net)) byNet.set(net, []);
    for (let k = 0; k + 1 < pts.length; k++) byNet.get(net).push([pts[k], pts[k + 1]]);
  }
  const dots = [];
  for (const [net, segs] of byNet) {
    const pts = new Map();
    for (const [a, b] of segs) for (const q of [a, b]) pts.set(q.join(), q);
    for (const q of pts.values()) {
      const dirs = new Set();
      for (const [a, b] of segs) {
        const vert = a[0] === b[0], lo = vert ? Math.min(a[1], b[1]) : Math.min(a[0], b[0]), hi = vert ? Math.max(a[1], b[1]) : Math.max(a[0], b[0]);
        const on = vert ? q[0] === a[0] && q[1] >= lo && q[1] <= hi : q[1] === a[1] && q[0] >= lo && q[0] <= hi;
        if (!on) continue;
        const c = vert ? q[1] : q[0];
        if (c > lo) dirs.add(vert ? 'u' : 'l');
        if (c < hi) dirs.add(vert ? 'd' : 'r');
      }
      for (const e of pinEnds) if (e.net === net && e.x === q[0] && e.y === q[1]) dirs.add('pin:' + e.dir);
      if (dirs.size >= 3) dots.push(q);
    }
  }
  return dots;
}
