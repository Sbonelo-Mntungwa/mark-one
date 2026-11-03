/* Part shapes and pin positions.

   The board grid is GRID units. Every part's width and height are multiples
   of 2 * GRID and every pin sits on a multiple of GRID, so pins land on grid
   points in every rotation. A part's rotation (p.rot, quarter turns
   clockwise) turns it about the centre of its unrotated box. */

import {chipDef, G} from '../core/state.js';

export const GRID = 20;

/* Unrotated shape: size and pins, each pin with an optional name and side. */
export function geom(p) {
  switch (p.kind) {
    case 'vcc': return {w: 40, h: 40, pins: [{x: 20, y: 40}]};
    case 'gnd': return {w: 40, h: 40, pins: [{x: 20, y: 0}]};
    // Pin order is G, top, bottom for both. The labels follow convention: a
    // PMOS has its source at the top (supply side), an NMOS at the bottom
    // (ground side). The simulator treats source and drain alike.
    case 'pmos': return {w: 40, h: 80, pins: [{x: 0, y: 40, n: 'G'}, {x: 40, y: 0, n: 'S'}, {x: 40, y: 80, n: 'D'}]};
    case 'nmos': return {w: 40, h: 80, pins: [{x: 0, y: 40, n: 'G'}, {x: 40, y: 0, n: 'D'}, {x: 40, y: 80, n: 'S'}]};
    case 'in': return {w: 80, h: 40, pins: [{x: 80, y: 20}]};
    case 'out': return {w: 80, h: 40, pins: [{x: 0, y: 20}]};
    case 'rom': {
      const pins = [];
      for (let i = 0; i < 4; i++) pins.push({x: 0, y: 80 + i * 20, n: 'A' + i, side: 'l'});
      for (let i = 0; i < 8; i++) pins.push({x: 120, y: 40 + i * 20, n: 'D' + i, side: 'r'});
      return {w: 120, h: 200, pins};
    }
    case 'display': {
      const pins = [];
      for (let i = 0; i < 8; i++) pins.push({x: 0, y: 40 + i * 20, n: 'B' + i, side: 'l'});
      return {w: 160, h: 200, pins};
    }
  }
  const d = chipDef(p.chip);
  if (!d) return {w: 80, h: 40, pins: []};
  const ni = d.inputs.length, no = d.outputs.length, n = Math.max(ni, no, 1);
  const up = (v, m) => Math.ceil(v / m) * m;
  // A 40-unit header for the label, a row per pin, a row of margin.
  const h = up(40 + n * 20, 40), w = up(Math.max(120, d.label.length * 11 + 66), 40);
  const row = k => 40 + Math.floor((n - k) / 2) * 20;
  const pins = [];
  d.inputs.forEach((nm, i) => pins.push({x: 0, y: row(ni) + i * 20, n: nm, side: 'l'}));
  d.outputs.forEach((nm, i) => pins.push({x: w, y: row(no) + i * 20, n: nm, side: 'r'}));
  return {w, h, pins};
}

export const rotOf = p => ((p.rot || 0) % 4 + 4) % 4;

/* Rotate a vector a quarter turn clockwise, r times (SVG y points down). */
function turn(x, y, r) {
  for (let i = 0; i < r; i++) [x, y] = [-y, x];
  return {x, y};
}

/* Which way a pin faces before rotation: out of the edge it sits on. */
function facing(g, q) {
  if (q.x <= 0) return {x: -1, y: 0};
  if (q.x >= g.w) return {x: 1, y: 0};
  if (q.y <= 0) return {x: 0, y: -1};
  return {x: 0, y: 1};
}

/* The part's footprint on the board, after rotation. `gf` gives the
   unrotated shape; the printed schematic passes its own. */
export function box(p, gf = geom) {
  const g = gf(p), odd = rotOf(p) % 2 === 1;
  const cx = p.x + g.w / 2, cy = p.y + g.h / 2, w = odd ? g.h : g.w, h = odd ? g.w : g.h;
  return {x: cx - w / 2, y: cy - h / 2, w, h};
}

/* Board position of pin k and the direction it faces. */
export function pinPos(p, k, gf = geom) {
  const g = gf(p), q = g.pins[k] || {x: 0, y: 0}, r = rotOf(p);
  const v = turn(q.x - g.w / 2, q.y - g.h / 2, r), d = turn(facing(g, q).x, facing(g, q).y, r);
  return {x: p.x + g.w / 2 + v.x, y: p.y + g.h / 2 + v.y, dx: d.x, dy: d.y};
}

export const partById = id => G.circuit.parts.find(p => p.id === id);

export const pv = (id, k) => (G.pins[id] && G.pins[id][k]) || 'Z';

export const cham = (w, h, c = 8) => `M${c},0 H${w - c} L${w},${c} V${h - c} L${w - c},${h} H${c} L0,${h - c} V${c} Z`;

export const ticks = (w, h) => `<path class="tick" d="M0,9 V0 H9 M${w - 9},${h} H${w} V${h - 9}"/>`;
