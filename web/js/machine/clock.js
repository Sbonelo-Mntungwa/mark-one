/* Clock, pulse and run-to-halt. */

import {sleep} from '../core/dom.js';
import {runSim} from '../core/session.js';
import {clkIdx, G, lvl} from '../core/state.js';
import {renderKeys} from '../ui/keys.js';
import {say} from '../ui/log.js';

export const SPEEDS = [0, 1, 4, 12];

export let speedIdx = 0, clockTok = 0;

export function stopClock() {
  speedIdx = 0; clockTok++;
  renderKeys();
}

/* What the help bar shows for the clock key. These are function declarations
   so the key table can refer to them while modules are still loading. */
export function clockLabel() { return speedIdx ? `Clock ${SPEEDS[speedIdx]} Hz` : 'Clock off'; }
export function clockRunning() { return speedIdx > 0; }

export async function clockLoop(tok) {
  while (tok === clockTok) {
    const k = clkIdx(); if (k < 0) return;
    G.inputs[k] = !G.inputs[k];
    await runSim();
    if (G.outputs[lvl().outputs.indexOf('HALT')] === '1' && !G.inputs[k]) { stopClock(); say('Computer halted. Clock stopped.', 'good'); return; }
    await sleep(500 / SPEEDS[speedIdx]);
  }
}

/* Step the clock through off, 1, 4 and 12 Hz. */
export function cycleClock() {
  if (clkIdx() < 0) return;
  speedIdx = (speedIdx + 1) % SPEEDS.length;
  if (!speedIdx) { stopClock(); return; }
  clockTok++;
  renderKeys();
  if (speedIdx === 1) say('Clock running. State only changes on the rising edge, when CLK goes from 0 to 1. Click again to speed up.');
  clockLoop(clockTok);
};

export async function pulse() {
  const k = clkIdx(); if (k < 0) return;
  stopClock();
  // Start low so there is always a rising edge, even if the clock stopped high.
  if (G.inputs[k]) { G.inputs[k] = false; await runSim(); }
  G.inputs[k] = true; await runSim();
  await sleep(180);
  G.inputs[k] = false; await runSim();
}

/* Clock the computer until HALT goes high. */
export async function runToHalt() {
  const k = clkIdx(); if (k < 0) return;
  stopClock();
  G.inputs[k] = false;
  const r = await runSim(false, 1000);
  if (!r || r.error) return;
  const halted = G.outputs[lvl().outputs.indexOf('HALT')] === '1';
  if (r.stopped && r.stopped.breakpoint != null) { say(`Ran ${r.cycles_run} cycle${r.cycles_run === 1 ? '' : 's'}. Stopped at the breakpoint on address ${r.stopped.breakpoint}.`, 'warn'); return; }
  say(halted ? `Ran ${r.cycles_run} cycles and halted.` : `Ran ${r.cycles_run} cycles without halting. Is there an HLT the program can reach?`, halted ? 'good' : 'warn');
}
