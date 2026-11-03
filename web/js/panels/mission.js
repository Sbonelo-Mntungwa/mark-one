/* Mission panel for a Mark. */

import {LORE} from '../content/lore.js';
import {$, esc} from '../core/dom.js';
import {G, lvl} from '../core/state.js';
import {renderProgram} from '../isa/editor.js';
import {renderFreeMission} from '../modes/free.js';
import {renderSpec} from './spec.js';
import {renderKeys} from '../ui/keys.js';

export function renderMission() {
  const l = lvl();
  if (G.free) { renderFreeMission(); return; }
  $('#mk').textContent = `Mk ${G.idx + 1}`;
  $('#mname').textContent = l.name;
  $('#mprod').textContent = l.chip ? `fabricates ${l.label}` : l.chapter;
  $('#mission').innerHTML = `<div class="intro">${esc(l.intro)}</div>
    <div class="pinrow">${l.inputs.map(n => `<span class="pin-chip">${esc(n)}</span>`).join('')}${l.outputs.map(n => `<span class="pin-chip o">${esc(n)}</span>`).join('')}</div>`;
  const lore = LORE[l.id];
  $('#databank').innerHTML = lore ? `<div class="lore"><b>${esc(lore[0])}</b>${esc(lore[1])}</div>` : '';
  renderKeys();
  renderSpec();
  renderProgram(true);
}
