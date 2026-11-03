/* Fabricator dock of placeable parts. */

import {$, esc} from '../core/dom.js';
import {lvl} from '../core/state.js';

export const ICON = {
  vcc: '<path d="M8,4 H26 L30,8 V16 L26,20 H8 L4,16 V8 Z" fill="none" stroke="#ffb84d"/><path d="M13,12 H21 M17,8 V16" stroke="#ffb84d" stroke-width="1.6"/>',
  gnd: '<path d="M17,2 V9 M8,9 H26 M11,13 H23 M14,17 H20" stroke="#8fd9f5" stroke-width="1.5" fill="none"/>',
  nmos: '<path d="M3,11 H12 M13,4 V18 M17,3 V19 M17,3 H24 V0 M17,19 H24 V22" stroke="#8fd9f5" stroke-width="1.5" fill="none"/>',
  pmos: '<path d="M3,11 H8 M13,4 V18 M17,3 V19 M17,3 H24 V0 M17,19 H24 V22" stroke="#8fd9f5" stroke-width="1.5" fill="none"/><circle cx="10.5" cy="11" r="2.2" fill="none" stroke="#8fd9f5" stroke-width="1.3"/>',
  chip: '<path d="M8,3 H26 L30,7 V15 L26,19 H8 L4,15 V7 Z" fill="rgba(255,184,77,.08)" stroke="#ffd18a"/><path d="M0,8 H4 M0,14 H4 M30,11 H34" stroke="#ffd18a"/>',
  in: '<path d="M4,5 H24 L30,11 L24,17 H4 Z" fill="rgba(255,184,77,.08)" stroke="#ffb84d"/>',
  out: '<path d="M10,5 H30 V17 H10 L4,11 Z" fill="rgba(90,215,255,.06)" stroke="#8fd9f5"/>',
  rom: '<path d="M6,2 H28 V20 H6 Z" fill="rgba(90,215,255,.06)" stroke="#8fd9f5"/><path d="M10,7 H24 M10,11 H20 M10,15 H24" stroke="#8fd9f5"/>',
  display: '<path d="M3,3 H31 V19 H3 Z" fill="rgba(255,184,77,.06)" stroke="#ffb84d"/><text x="17" y="15" text-anchor="middle" font-size="10" fill="#ffb84d" font-family="monospace">42</text>',
};

export function renderDock() {
  const l = lvl();
  $('#dock').innerHTML = `<div class="lead"><b>Fabricator</b>click to place</div>` + l.palette.map((e, i) =>
    `<button class="tile${e.kind === 'chip' ? ' chip' : ''}" data-pal="${i}">
      <svg viewBox="0 0 34 22">${ICON[e.kind] || ''}</svg><span class="tl">${esc(e.label)}</span>
      </button>`).join('');
}
