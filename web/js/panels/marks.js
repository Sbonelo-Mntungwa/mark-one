/* Left panel: Missions, Experiment, and the component library. */

import {$, esc} from '../core/dom.js';
import {G} from '../core/state.js';

export function renderMarks() {
  let s = '', chap = null;
  G.levels.forEach((l, i) => {
    if (l.chapter !== chap) { chap = l.chapter; s += `<div class="chap">${esc(chap)}</div>`; }
    s += `<button class="mrow ${l.status}${i === G.idx && !G.free ? ' cur' : ''}" data-lv="${i}" ${l.status === 'locked' ? 'disabled' : ''}>
      <span class="node"></span><span class="n">Mk ${i + 1}</span><span class="t">${esc(l.name)}</span></button>`;
  });
  const lib = G.components || [], have = lib.filter(c => c.unlocked).length, mine = (G.custom || []).length;
  $('#marks').innerHTML = `<div class="ptitle">Experiment</div>
    <button class="mrow open${G.free ? ' cur' : ''}" data-free="1"><span class="node"></span><span class="n">Bench</span><span class="t">Build with what you unlocked</span></button>
    <button class="mrow open" data-library="1"><span class="node"></span><span class="n">Parts</span><span class="t">Components</span><span class="c">${have}/${lib.length}${mine ? ` +${mine}` : ''}</span></button>
    <div class="ptitle" style="margin-top:10px">Missions</div>${s}`;
}
