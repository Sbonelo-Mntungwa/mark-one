/* Pan and zoom of the board viewport. */

import {box} from './geometry.js';
import {G, svg} from '../core/state.js';

export function safe() {
  const W = innerWidth, H = innerHeight;
  if (W <= 820) return {l: 12, r: 12, t: 90, b: H * .36 + 154};
  const pane = document.body.classList.contains('nopane') ? 12 : W <= 1100 ? 212 : 256;
  const wave = document.body.classList.contains('wave') ? 236 : 0;
  return {l: pane, r: W <= 1100 ? 324 : 380, t: 92, b: 154 + wave};
}

export function setVB() { svg.setAttribute('viewBox', `${G.vb.x} ${G.vb.y} ${G.vb.w} ${G.vb.h}`); }

export function fitView() {
  const ps = G.circuit.parts, W = innerWidth, H = innerHeight, s = safe();
  let x0 = 0, y0 = 0, x1 = 700, y1 = 400;
  if (ps.length) {
    const bs = ps.map(p => box(p));
    x0 = Math.min(...bs.map(b => b.x)); y0 = Math.min(...bs.map(b => b.y));
    x1 = Math.max(...bs.map(b => b.x + b.w)); y1 = Math.max(...bs.map(b => b.y + b.h));
  }
  const pad = 50, sw = Math.max(W - s.l - s.r, 200), sh = Math.max(H - s.t - s.b, 160);
  const k = Math.max((x1 - x0 + pad * 2) / sw, (y1 - y0 + pad * 2) / sh, 0.9);
  G.vb = {w: W * k, h: H * k, x: (x0 + x1) / 2 - (s.l + sw / 2) * k, y: (y0 + y1) / 2 - (s.t + sh / 2) * k};
  setVB();
}

export function svgPt(e) { return {x: G.vb.x + e.clientX / innerWidth * G.vb.w, y: G.vb.y + e.clientY / innerHeight * G.vb.h}; }

addEventListener('resize', () => { G.vb.h = G.vb.w * innerHeight / innerWidth; setVB(); });
