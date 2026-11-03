/* Wire routing on the board grid.

   Each wire is an orthogonal path along grid lines. It leaves each pin in the
   direction the pin faces, runs straight when the two ends line up, and steps
   out and around when its target lies behind a pin. After routing, segments
   of different nets that would lie on top of each other are nudged apart, so
   a drawing never shows two separate nets as one. */

import {box, geom, GRID, partById, pinPos} from './geometry.js';
import {snap} from '../core/dom.js';
import {G} from '../core/state.js';

const S = GRID;

/* The grid line nearest `want` along which a segment from lo to hi touches
   no part body or outline. `horiz`: the segment is horizontal at that y. */
function freeLine(want, lo, hi, horiz, boxes) {
  if (lo > hi) [lo, hi] = [hi, lo];
  const blocked = c => boxes.some(b => horiz
    ? c >= b.y && c <= b.y + b.h && Math.max(lo, b.x) < Math.min(hi, b.x + b.w)
    : c >= b.x && c <= b.x + b.w && Math.max(lo, b.y) < Math.min(hi, b.y + b.h));
  for (let k = 0; k <= 12; k++) for (const c of k ? [want - k * S, want + k * S] : [want]) if (!blocked(c)) return c;
  return want;
}

/* Points of an orthogonal route from pin a to pin b (each {x, y, dx, dy}),
   keeping crossover segments off the part bodies in `boxes`. */
export function route(a, b, boxes = []) {
  const ah = a.dx !== 0, bh = b.dx !== 0;
  // An end may run straight on only if the other end lies ahead of it.
  const ahead = (p, q) => (q.x - p.x) * (p.dx || 0) + (q.y - p.y) * (p.dy || 0) > 0;
  const fwd = ahead(a, b) && ahead(b, a);
  const A = [a.x, a.y], B = [b.x, b.y];
  if ((a.x === b.x || a.y === b.y) && fwd) return [A, B];
  const ax = a.x + a.dx * S, ay = a.y + a.dy * S, bx = b.x + b.dx * S, by = b.y + b.dy * S;
  if (ah && bh) {
    if (fwd) { const mx = freeLine(snap((a.x + b.x) / 2), a.y, b.y, false, boxes); return [A, [mx, a.y], [mx, b.y], B]; }
    // Step out of each pin, cross between them, or below both on one row.
    const my = freeLine(Math.abs(a.y - b.y) < 2 * S ? Math.max(a.y, b.y) + 2 * S : snap((a.y + b.y) / 2), ax, bx, true, boxes);
    return [A, [ax, a.y], [ax, my], [bx, my], [bx, b.y], B];
  }
  if (!ah && !bh) {
    if (fwd) { const my = freeLine(snap((a.y + b.y) / 2), a.x, b.x, true, boxes); return [A, [a.x, my], [b.x, my], B]; }
    const mx = freeLine(Math.abs(a.x - b.x) < 2 * S ? Math.max(a.x, b.x) + 2 * S : snap((a.x + b.x) / 2), ay, by, false, boxes);
    return [A, [a.x, ay], [mx, ay], [mx, by], [b.x, by], B];
  }
  // One horizontal end, one vertical: an L when both point at the corner.
  const [h, v] = ah ? [a, b] : [b, a];
  const ok = (v.x - h.x) * h.dx > 0 && (h.y - v.y) * v.dy > 0;
  const pts = ok ? [[h.x, h.y], [v.x, h.y], [v.x, v.y]]
    : [[h.x, h.y], [h.x + h.dx * S, h.y], [h.x + h.dx * S, v.y + v.dy * S], [v.x, v.y + v.dy * S], [v.x, v.y]];
  return ah ? pts : pts.reverse();
}

/* Drop repeated and collinear points, so segments alternate direction. */
function simplify(pts) {
  const out = [];
  for (const p of pts) {
    if (out.length && out[out.length - 1][0] === p[0] && out[out.length - 1][1] === p[1]) continue;
    if (out.length >= 2) {
      const [q, r] = [out[out.length - 2], out[out.length - 1]];
      if ((q[0] === r[0] && r[0] === p[0]) || (q[1] === r[1] && r[1] === p[1])) { out[out.length - 1] = p; continue; }
    }
    out.push(p);
  }
  return out;
}

/* Net of each wire: wires that share a pin belong to the same net. */
function nets(wires) {
  const parent = new Map(), find = k => { while (parent.get(k) !== k) { parent.set(k, parent.get(parent.get(k))); k = parent.get(k); } return k; };
  const key = e => `${e.part}:${e.pin}`;
  for (const w of wires) for (const e of [w.from, w.to]) if (!parent.has(key(e))) parent.set(key(e), key(e));
  for (const w of wires) parent.set(find(key(w.from)), find(key(w.to)));
  return wires.map(w => find(key(w.from)));
}

const seg = (p, q) => p[0] === q[0]
  ? {key: 'v' + p[0], lo: Math.min(p[1], q[1]), hi: Math.max(p[1], q[1])}
  : {key: 'h' + p[1], lo: Math.min(p[0], q[0]), hi: Math.max(p[0], q[0])};

/* Routes for every wire: {i, pts, net}. `gf` gives part shapes (see box). */
export function routeAll(gf = geom) {
  const ws = G.circuit.wires, netOf = nets(ws), used = new Map(), out = [], boxes = G.circuit.parts.map(p => box(p, gf));
  // Segments of different nets on one line must not overlap, or even touch
  // end to end: either would read as one wire.
  const clash = (s, net) => (used.get(s.key) || []).some(o => o.net !== net && Math.max(o.lo, s.lo) <= Math.min(o.hi, s.hi));
  ws.forEach((w, i) => {
    const a = partById(w.from.part), b = partById(w.to.part);
    if (!a || !b) return;
    const pts = simplify(route(pinPos(a, w.from.pin, gf), pinPos(b, w.to.pin, gf), boxes)), net = netOf[i];
    // Segments not touching a pin can move sideways off another net.
    for (let k = 1; k + 2 < pts.length; k++) {
      const vert = pts[k][0] === pts[k + 1][0], ax = vert ? 0 : 1;
      for (const d of [0, S, -S, 2 * S, -2 * S, 3 * S, -3 * S]) {
        const p = pts[k].slice(), q = pts[k + 1].slice();
        p[ax] += d; q[ax] += d;
        // Moving this segment stretches its neighbours: check them too.
        const ok = !clash(seg(p, q), net) && !clash(seg(pts[k - 1], p), net) && !clash(seg(q, pts[k + 2]), net);
        if (ok || d === -3 * S) { pts[k] = p; pts[k + 1] = q; break; }
      }
    }
    for (let k = 0; k + 1 < pts.length; k++) {
      const s = seg(pts[k], pts[k + 1]);
      if (!used.has(s.key)) used.set(s.key, []);
      used.get(s.key).push({...s, net});
    }
    out.push({i, pts: simplify(pts), net});
  });
  return out;
}

export const pathD = pts => 'M' + pts.map(([x, y]) => `${x},${y}`).join(' L');
