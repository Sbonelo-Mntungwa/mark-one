/* The lesson card. */

import {LESSONS} from '../content/lessons.js';
import {$, esc} from '../core/dom.js';
import {G, lvl} from '../core/state.js';
import {DEMOS} from './demos.js';

export const LS = {level: null};

export function lessonSeen(id) { try { return localStorage.getItem('m1-lesson-' + id) === '1'; } catch (e) { return true; } }

export function lessonOpen() { return !$('#lesson').classList.contains('hidden'); }

export function openLesson() {
  const l = lvl(); if (!l || !LESSONS[l.id]) return;
  LS.level = l;
  $('#lesson').classList.remove('hidden');
  renderLesson();
}

export function closeLesson() {
  $('#lesson').classList.add('hidden');
  try { localStorage.setItem('m1-lesson-' + LS.level.id, '1'); } catch (e) {}
}

export function lessonKey(e) {
  if (e.key !== 'Escape') return;
  e.preventDefault(); closeLesson();
}

export function renderLesson() {
  const l = LS.level, s = LESSONS[l.id];
  $('#lesson').innerHTML = `<div class="lcard panel" role="dialog" aria-label="Lesson">
    <div class="lhd"><span class="lk">Mk ${G.levels.indexOf(l) + 1}</span><span class="lt">Lesson: ${esc(l.name)}</span></div>
    <div class="ltext"><h2>${esc(s.t)}</h2><p>${esc(s.b)}</p><p class="note">The mission panel has your task. Working out how to build it is up to you.</p></div>
    <div class="ldemo" id="ldemo"></div>
    <div class="lnav"><span class="sp">Esc to close. Reopen anytime with Lesson.</span><button id="lNext" class="prime">Start building</button></div></div>`;
  $('#lNext').onclick = closeLesson;
  $('#lNext').focus();
  (DEMOS[s.d] || (() => {}))($('#ldemo'), l);
}
