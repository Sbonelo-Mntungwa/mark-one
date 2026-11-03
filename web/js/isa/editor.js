/* ROM program editor in the side panel. */

import {partById} from '../board/geometry.js';
import {$, esc} from '../core/dom.js';
import {runSim, saveDraft} from '../core/session.js';
import {G, lvl} from '../core/state.js';
import {assemble, disasm, hex2, ISA, romAddr} from './assembler.js';
import {toggleBreakpoint} from '../panels/debugger.js';
import {say} from '../ui/log.js';

export function targetRom() {
  const s = G.sel && G.sel.type === 'part' ? partById(G.sel.id) : null;
  if (s && s.kind === 'rom') return s;
  return G.circuit.parts.find(p => p.kind === 'rom') || null;
}

export function renderProgram(force) {
  const l = lvl(), wrap = $('#progWrap');
  if (!l || !l.rom) { wrap.classList.add('hidden'); return; }
  wrap.classList.remove('hidden');
  const rom = targetRom();
  const id = rom ? rom.id : null;
  if (!force && id === G.romTarget) return;
  G.romTarget = id;
  if (!rom) { $('#prog').innerHTML = `<div class="note">Place a ROM from the fabricator, then write its program here.</div>`; return; }
  $('#prog').innerHTML = `<textarea id="asm" spellcheck="false" aria-label="Assembly program">${esc(rom.src ?? disasm(rom.data))}</textarea>
    <div class="row"><button id="asmLoad" class="prime">Load into ROM</button><span class="note" id="asmInfo"></span></div>
    <div id="asmErr"></div><div class="listing" id="listing"></div>`;
  $('#asmLoad').onclick = () => {
    const r = partById(G.romTarget); if (!r) return;
    const text = $('#asm').value, a = assemble(text);
    $('#asmErr').innerHTML = a.errors.map(e => `<div class="asmerr">${esc(e)}</div>`).join('');
    if (a.errors.length) { say('Assembler found errors. Nothing was loaded.', 'bad'); return; }
    r.src = text; r.data = a.bytes;
    say(`Program loaded: ${a.count} instruction${a.count === 1 ? '' : 's'}. The computer restarts from address 0.`, 'good');
    saveDraft();
    runSim(true);
  };
  $('#asm').addEventListener('keydown', e => { if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') { e.preventDefault(); $('#asmLoad').click(); } });
  renderListing();
}

export function renderListing() {
  const el = $('#listing'); if (!el) return;
  const rom = partById(G.romTarget); if (!rom) return;
  const data = rom.data || [], a = romAddr(rom);
  $('#asmInfo').textContent = 'Ctrl+Enter also loads';
  el.innerHTML = Array.from({length: 16}, (_, i) => {
    const x = data[i] || 0, op = ISA.find(o => o[1] === x >> 4);
    const txt = op ? (op[2] ? `${op[0]} ${x & 15}` : op[0]) : '?';
    const bp = G.breakpoints.has(i);
    return `<div class="${i === a ? 'cur' : ''}${bp ? ' bp' : ''}" data-addr="${i}" title="${bp ? 'Remove the breakpoint' : 'Stop here when running'}"><span class="a">${String(i).padStart(2, '0')}</span> ${hex2(x)} ${txt}</div>`;
  }).join('');
  el.onclick = e => {
    const d = e.target.closest('[data-addr]'); if (!d) return;
    toggleBreakpoint(+d.dataset.addr); renderListing();
  };
}
