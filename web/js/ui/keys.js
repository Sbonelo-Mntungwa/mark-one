/* Keyboard shortcuts. One table drives both the keys and the help bar, so
   the bar always shows exactly what works on the current board. */

import {clearBoard, deleteSel, rotateSel} from '../board/editing.js';
import {render} from '../board/render.js';
import {fitView} from '../board/view.js';
import {$, esc} from '../core/dom.js';
import {runSim} from '../core/session.js';
import {clkIdx, G, lvl} from '../core/state.js';
import {LESSONS} from '../content/lessons.js';
import {lessonKey, lessonOpen, openLesson} from '../lesson/player.js';
import {clockLabel, clockRunning, cycleClock, pulse, runToHalt} from '../machine/clock.js';
import {runDiagnostics} from '../panels/diagnostics.js';
import {closeLibrary, libraryOpen} from '../panels/library.js';
import {dialogOpen} from './dialog.js';
import {say} from './log.js';
import {probeSelection, toggleTimed, toggleWave, waveOpen} from '../panels/wave.js';
import {printSchematic} from './print.js';

const paneHidden = () => document.body.classList.contains('nopane');

export function togglePane(show = paneHidden()) {
  document.body.classList.toggle('nopane', !show);
  try { localStorage.setItem('m1-pane', show ? '1' : '0'); } catch (e) {}
  const tab = $('#paneTab');
  tab.textContent = show ? '‹' : '›';
  tab.title = show ? 'Hide the panel ([)' : 'Show the panel ([)';
  tab.setAttribute('aria-label', tab.title);
  renderKeys();
}

const hasClk = () => clkIdx() >= 0;
const hasHalt = () => lvl().outputs.includes('HALT');
const graded = () => !G.free && lvl().mode !== 'sandbox';

/* key: KeyboardEvent.key values (lowercase for letters). */
const KEYS = [
  {key: ['d'], show: 'D', label: 'Run diagnostics', when: graded, run: runDiagnostics, prime: true},
  {key: ['r'], show: 'R / ⇧R', label: 'Rotate', run: e => rotateSel(e.shiftKey)},
  {key: ['Delete', 'Backspace'], show: 'Del', label: 'Delete', run: deleteSel},
  {key: ['f'], show: 'F', label: 'Recenter', run: fitView},
  {key: [' '], show: 'Space', label: 'Pulse', when: hasClk, run: pulse},
  {key: ['k'], show: 'K', label: clockLabel, when: hasClk, run: cycleClock, on: clockRunning},
  {key: ['h'], show: 'H', label: 'Run to halt', when: hasHalt, run: runToHalt},
  {key: ['a'], show: 'A', label: 'Probe', run: probeSelection},
  {key: ['w'], show: 'W', label: () => waveOpen() ? 'Hide waveform' : 'Waveform', run: () => toggleWave()},
  {key: ['t'], show: 'T', label: () => G.timed ? 'Timed' : 'Zero-delay', run: toggleTimed, on: () => G.timed},
  {key: ['p'], show: 'P', label: 'Print', run: printSchematic},
  {key: ['l'], show: 'L', label: 'Lesson', when: () => !G.free && !!LESSONS[lvl().id], run: openLesson},
  {key: ['x'], show: 'X', label: 'Reset simulation', run: () => { runSim(true); say('Simulation rebuilt. Stored state cleared.'); }},
  {key: ['c'], show: 'C', label: 'Clear board', run: clearBoard},
  {key: ['['], show: '[', label: () => paneHidden() ? 'Show panel' : 'Hide panel', run: () => togglePane()},
  {key: ['Escape'], show: 'Esc', label: 'Cancel', run: () => { G.pending = null; G.sel = null; render(); }},
];

const text = v => typeof v === 'function' ? v() : v;

export function renderKeys() {
  const bar = $('#helpbar'); if (!bar || !G.levels.length) return;
  bar.innerHTML = KEYS.filter(k => !k.when || k.when()).map(k =>
    `<span class="hk${k.prime ? ' prime' : ''}${k.on && k.on() ? ' on' : ''}"><kbd>${esc(k.show)}</kbd>${esc(text(k.label))}</span>`).join('');
}

addEventListener('keydown', e => {
  if (['INPUT', 'TEXTAREA', 'SELECT'].includes(e.target.tagName) || !G.levels.length || dialogOpen()) return;
  if (lessonOpen()) { lessonKey(e); return; }
  if (libraryOpen()) { if (e.key === 'Escape') closeLibrary(); return; }
  // Leave browser and system shortcuts (Cmd+P, Ctrl+R ...) alone.
  if (e.ctrlKey || e.metaKey || e.altKey) return;
  const key = e.key.length === 1 ? e.key.toLowerCase() : e.key;
  const k = KEYS.find(x => x.key.includes(key) && (!x.when || x.when()));
  if (!k) return;
  e.preventDefault();
  k.run(e);
});

$('#paneTab').onclick = () => togglePane();

let stored = null;
try { stored = localStorage.getItem('m1-pane'); } catch (e) {}
togglePane(stored !== '0');
