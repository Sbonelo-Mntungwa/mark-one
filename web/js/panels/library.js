/* The component library: what Missions have unlocked, what is still
   locked, and the components you made yourself. */

import {api} from '../core/api.js';
import {$, esc} from '../core/dom.js';
import {refreshState} from '../core/session.js';
import {G} from '../core/state.js';
import {dialog} from '../ui/dialog.js';
import {say} from '../ui/log.js';

const pins = (ins, outs) => `<div class="pinrow">${ins.map(n => `<span class="pin-chip">${esc(n)}</span>`).join('')}${outs.map(n => `<span class="pin-chip o">${esc(n)}</span>`).join('')}</div>`;

function card({label, name, summary, ins, outs, cost, locked, note, del}) {
  return `<div class="lcomp${locked ? ' locked' : ''}">
    <div class="lchip">${esc(label)}</div>
    <div class="lmeta"><b>${esc(name)}</b>${summary ? `<span>${esc(summary)}</span>` : ''}${ins ? pins(ins, outs) : ''}
      <small>${cost != null ? `${cost} transistor${cost === 1 ? '' : 's'}` : ''}${note ? `${cost != null ? ' · ' : ''}${note}` : ''}</small></div>
    ${del ? `<button class="ldel" data-del="${esc(del)}" title="Delete this component">Delete</button>` : ''}</div>`;
}

export function libraryOpen() { return !$('#library').classList.contains('hidden'); }

export function closeLibrary() { $('#library').classList.add('hidden'); }

export function openLibrary() {
  const u = G.unlocks || {}, lib = G.components || [], mine = G.custom || [];
  const lockNote = mk => `<span class="lk">Locked: pass Mk ${mk}</span>`;
  const basics = [
    card({label: 'VCC / GND', name: 'Supply rails', summary: 'Constant 1 and constant 0.', cost: 0, note: 'Always available'}),
    card({label: 'IN / OUT', name: 'Pins', summary: 'Named inputs and outputs of an Experiment design.', cost: 0, note: 'Always available'}),
    card({label: 'NMOS / PMOS', name: 'Transistors', summary: 'Switch-level MOSFETs.', cost: 1, locked: !u.transistors, note: u.transistors ? 'Unlocked' : lockNote(u.transistors_mark)}),
    card({label: 'ROM / DISP', name: 'Program memory and display', summary: '16-byte ROM with an assembler, and a decimal readout.', locked: !u.rom, note: u.rom ? 'Unlocked' : lockNote(u.rom_mark)}),
  ].join('');
  const comps = lib.map(c => card({label: c.label, name: c.name, summary: c.summary, ins: c.inputs, outs: c.outputs, cost: c.cost,
    locked: !c.unlocked, note: c.unlocked ? `Unlocked by Mk ${c.mark}` : lockNote(c.mark)})).join('');
  const own = mine.length ? mine.map(c => card({label: c.label, name: c.name, ins: c.inputs, outs: c.outputs, cost: c.cost, note: 'Your component', del: c.id})).join('')
    : `<div class="note">None yet. In Experiment, build a circuit with INPUT and OUTPUT pins, then press Save as component.</div>`;
  $('#library').innerHTML = `<div class="libcard panel" role="dialog" aria-label="Components">
    <div class="lhd"><h2>Components</h2><span class="lt">${lib.filter(c => c.unlocked).length} of ${lib.length} unlocked through Missions</span><button id="libClose" class="lclose">Close</button></div>
    <div class="lbody">
      <div class="ptitle">Your components</div><div class="lgrid">${own}</div>
      <div class="ptitle">Basic parts</div><div class="lgrid">${basics}</div>
      <div class="ptitle">Library</div><div class="lgrid">${comps}</div>
    </div></div>`;
  $('#library').classList.remove('hidden');
  $('#libClose').onclick = closeLibrary;
  $('#library').onclick = e => { if (e.target.id === 'library') closeLibrary(); };
  $('#library').querySelectorAll('[data-del]').forEach(b => b.onclick = async () => {
    const c = mine.find(x => x.id === b.dataset.del);
    const ok = await dialog({title: `Delete ${c.name}?`, danger: true, ok: 'Delete component',
      body: 'Designs that use this component will no longer build. This cannot be undone.'});
    if (!ok) return;
    try {
      await api('DELETE', `/api/custom/${encodeURIComponent(c.id)}`);
      say(`Deleted component ${c.name}.`, 'warn');
      await refreshState(); openLibrary();
    } catch (e) { dialog({title: 'Cannot delete', body: esc(e.message), ok: 'OK'}); }
  });
}
