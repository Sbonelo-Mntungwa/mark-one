/* The progress reactor drawn in the brand mark. */

import {$} from '../core/dom.js';
import {doneCount, G} from '../core/state.js';

export function reactorSVG(big) {
  const n = G.levels.length || 24, R1 = 34, R2 = 50, gap = 3.2;
  let s = `<defs><filter id="rglow"><feGaussianBlur stdDeviation="2"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter>
    <radialGradient id="coreg"><stop offset="0" stop-color="#ffffff"/><stop offset=".45" stop-color="#bfefff"/><stop offset="1" stop-color="#5ad7ff" stop-opacity="0"/></radialGradient></defs>`;
  s += `<circle class="ring" r="57"/><circle class="ring" r="28"/>`;
  for (let i = 0; i < n; i++) {
    const a0 = (i / n * 360 + gap / 2 - 90) * Math.PI / 180, a1 = ((i + 1) / n * 360 - gap / 2 - 90) * Math.PI / 180;
    const p = (r, a) => `${(r * Math.cos(a)).toFixed(2)},${(r * Math.sin(a)).toFixed(2)}`;
    const st = G.levels[i] ? G.levels[i].status : 'locked';
    s += `<path class="seg ${big ? '' : st}" data-seg="${i}" d="M${p(R1, a0)} L${p(R2, a0)} A${R2},${R2} 0 0 1 ${p(R2, a1)} L${p(R1, a1)} A${R1},${R1} 0 0 0 ${p(R1, a0)} Z"/>`;
  }
  const lit = doneCount() / (G.levels.length || 1);
  s += `<g class="spin${big ? '' : ' rev'}"><circle r="22" fill="none" stroke="#5ad7ff" stroke-opacity=".5" stroke-dasharray="4 6"/></g>`;
  s += `<circle r="${12 + lit * 8}" fill="url(#coreg)" opacity="${.45 + lit * .55}"/><circle class="core" r="5"/>`;
  return s;
}

export function renderReactor() {
  $('#reactor').innerHTML = reactorSVG(false);
  const d = doneCount();
  $('#progress').textContent = `${d} of ${G.levels.length} Marks fabricated`;
}
