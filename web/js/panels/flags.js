/* Warning flags for shorts, oscillation and floating outputs. */

import {$, esc} from '../core/dom.js';
import {G} from '../core/state.js';
import {say} from '../ui/log.js';

export function renderFlags() {
  let s = '';
  if (G.simError) s += `<div class="flag">${esc(G.simError)}</div>`;
  if (G.short) s += `<div class="flag">Short circuit: a line is driven to 1 and 0 at once</div>`;
  if (G.unstable) s += `<div class="flag warn">Feedback loop oscillating</div>`;
  if (G.degraded) s += `<div class="flag warn">Output degraded: driven only through a pass transistor</div>`;
  if (!G.simError && G.outputs.includes('Z')) s += `<div class="flag warn">Output floating: nothing drives it</div>`;
  $('#flags').innerHTML = s;
  const now = {short: G.short, unstable: G.unstable, err: G.simError, degraded: G.degraded};
  if (now.short && !G.last.short) say('Short circuit. Current would flow straight from supply to ground and cook the chip. Follow the red line.', 'bad');
  if (now.unstable && !G.last.unstable) say('Oscillation detected. A loop with an odd number of inversions never settles. That is how a ring oscillator works.', 'warn');
  if (now.err && now.err !== G.last.err) say(now.err, 'bad');
  if (G.degraded && !G.last.degraded) say('Degraded output. An NMOS passes 1 only up to about VDD minus its threshold voltage, and a PMOS passes 0 only down to about its threshold. Real logic restores levels with a complementary stage.', 'warn');
  G.last = now;
}
