/* Copying the Mk 37 computer onto software Marks. */

import {api} from '../core/api.js';
import {ensureIO, saveDraft} from '../core/session.js';
import {G} from '../core/state.js';
import {assemble, STARTER} from './assembler.js';
import {say} from '../ui/log.js';

export async function buildComputerTemplate() {
  let src = null;
  try { src = await api('GET', '/api/levels/computer/circuit'); } catch (e) {}
  const ok = src && src.parts && src.parts.some(p => p.kind === 'chip' && p.chip === 'cpu') && src.parts.some(p => p.kind === 'rom');
  if (!ok) { say('Build your computer on this board, or finish Mk 37 first and it will be copied here.', 'warn'); return; }
  G.circuit = JSON.parse(JSON.stringify(src));
  ensureIO();
  say('Loaded your computer from Mk 37.');
  G.circuit.parts.filter(p => p.kind === 'rom').forEach(p => { p.src = STARTER; p.data = assemble(STARTER).bytes; });
  saveDraft();
}
