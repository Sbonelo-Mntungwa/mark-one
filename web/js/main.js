/* Entry point: load the course and open the last board. */

import {api} from './core/api.js';
import {esc} from './core/dom.js';
import {openLevel} from './core/session.js';
import {applyState, G} from './core/state.js';
import {openFree} from './modes/free.js';
import {boot} from './ui/boot.js';
import {renderReactor} from './ui/reactor.js';
// Modules that bind event handlers when loaded.
import './board/input.js';
import './board/view.js';
import './machine/clock.js';
import './panels/diagnostics.js';
import './ui/scanner.js';
import './ui/keys.js';

(async () => {
  try { applyState(await api('GET', '/api/state')); }
  catch (e) { document.body.insertAdjacentHTML('beforeend', `<div class="flag" style="position:fixed;top:100px;left:50%;transform:translateX(-50%);z-index:70">Cannot reach the Mark One server: ${esc(e.message)}</div>`); return; }
  renderReactor();
  await boot();
  let want = null;
  try { want = localStorage.getItem('lf-level'); } catch (e) {}
  if (want && want.startsWith('free:')) { await openFree(want.slice(5)); return; }
  let i = G.levels.findIndex(l => l.id === want && l.status !== 'locked');
  if (i < 0) { i = G.levels.findIndex(l => l.status === 'open'); if (i < 0) i = 0; }
  await openLevel(i);
})();
