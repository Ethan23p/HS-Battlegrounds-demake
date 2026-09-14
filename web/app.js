// Plays back a bg-sim `Resolution` (produced live by bg-wasm; see boot()
// below).
//
// bg-sim's `boards` field is the fix for a debt this file used to carry: it
// used to reconstruct board state itself, replaying `log` against a
// hand-written copy of bg-sim's own rules (when a shield breaks, what Reborn
// revives with, how compaction packs). That required a producer (bg-sim) and
// a consumer (this file) to agree on logic neither could check the other
// against -- exactly the kind of coupling a fact log is supposed to avoid.
// `boards[i]` now tells the resulting state directly: the Board exactly as it
// stood once Beat `i + 1` finished, nothing inferred. `log` is read here only
// for narration and animation timing -- which slot to flash, what number to
// pop -- never for what a Unit's resulting stats or keywords are.

const KEYWORD_BADGE = {
  Taunt: "T",
  DivineShield: "D",
  Windfury: "W",
  Poisonous: "P",
  Reborn: "R",
  Rally: "L",
};

const SLOTS = 8;
const SIDES = ["Player", "Opposing"];

function otherSide(side) {
  return side === "Player" ? "Opposing" : "Player";
}

// One entry per Struck/Died/etc. in a Beat, carrying only what a cue needs to
// animate -- never enough to derive a rule from.
function narrate(cue) {
  switch (cue.kind) {
    case "hit": {
      const verb = cue.answering ? "strikes back at" : "strikes";
      return `  ${cue.attackerSide.toLowerCase()} slot ${cue.attackerSlot + 1} ${verb} ${cue.targetSide.toLowerCase()} slot ${cue.targetSlot + 1} for ${cue.damage}`;
    }
    case "absorb": {
      // The absorb cue stands in for the hit line entirely -- bg-sim merges
      // the blow and its absorption into one Event pair, so this is the only
      // narration this blow gets. It needs the same who-did-what a hit line
      // carries, or a reader can't tell what was absorbed.
      const verb = cue.answering ? "strikes back at" : "strikes";
      return `  ${cue.attackerSide.toLowerCase()} slot ${cue.attackerSlot + 1} ${verb} ${cue.targetSide.toLowerCase()} slot ${cue.targetSlot + 1}, absorbed by its shield`;
    }
    case "died":
      return `  ${cue.side.toLowerCase()} slot ${cue.slot + 1} (${cue.name}) dies`;
    case "reborn":
      return `  ${cue.side.toLowerCase()} slot ${cue.slot + 1} returns as ${cue.name} with 1 health`;
    case "compact":
      return `  the ${cue.side.toLowerCase()} party closes ranks`;
    default:
      return "";
  }
}

// Turn the flat Event log into one step per Beat (a list of narration cues,
// plus that Beat's already-resolved Board) and a final "ended" step. A
// Struck/StruckBack is immediately followed by its ShieldAbsorbed when one
// happens -- bg-sim's `hit()` pushes them as one atomic pair and nothing else
// can land between them -- so this is the one place the log is read with a
// lookahead of one.
function buildBeatSteps(log, boards) {
  const steps = [];
  let current = null;

  for (let i = 0; i < log.length; i++) {
    const [type, data] = Object.entries(log[i])[0];

    if (type === "BeatBegan") {
      current = { kind: "beat", beat: data.beat, cues: [], board: boards[data.beat - 1] };
      steps.push(current);
      continue;
    }
    if (type === "Ended") {
      steps.push({ kind: "ended", outcome: data.outcome, beats: data.beats });
      continue;
    }
    if (type === "Struck" || type === "StruckBack") {
      // A StruckBack is the defensive answer a struck Unit makes in the same
      // motion, not a second, independent attack -- distinguished in the
      // narration ("strikes back at") so two Units trading blows in one Beat
      // doesn't read as the same line printed twice.
      const answering = type === "StruckBack";
      const attackerSide = data.by;
      const targetSide = otherSide(data.by);
      const attackerSlot = answering ? data.slot - 1 : data.attacker_slot - 1;
      const targetSlot = data.target_slot - 1;
      const next = log[i + 1] && Object.entries(log[i + 1])[0];
      if (next && next[0] === "ShieldAbsorbed" && next[1].side === targetSide && next[1].slot - 1 === targetSlot) {
        current.cues.push({ kind: "absorb", answering, attackerSide, attackerSlot, targetSide, targetSlot });
        i++; // consume the paired ShieldAbsorbed
      } else {
        current.cues.push({ kind: "hit", answering, attackerSide, attackerSlot, targetSide, targetSlot, damage: data.damage });
      }
      continue;
    }
    if (type === "Died") current.cues.push({ kind: "died", side: data.side, slot: data.slot - 1, name: data.name });
    else if (type === "Reborn") current.cues.push({ kind: "reborn", side: data.side, slot: data.slot - 1, name: data.name });
    else if (type === "Compacted") current.cues.push({ kind: "compact", side: data.side });
    else if (type === "ShieldAbsorbed") {
      // Only reachable if one appears without a preceding Struck/StruckBack,
      // which bg-sim never emits. A visible failure beats a silently dropped
      // event.
      throw new Error("unpaired ShieldAbsorbed in log");
    }
  }
  return steps;
}

// ---------------------------------------------------------------------------
// Rendering + playback
// ---------------------------------------------------------------------------

const rowEl = { Player: document.getElementById("player-row"), Opposing: document.getElementById("opposing-row") };
const logEl = document.getElementById("log");
const outcomeEl = document.getElementById("outcome");
const beatValueEl = document.getElementById("beat-value");
const resultValueEl = document.getElementById("result-value");
const playBtn = document.getElementById("play");
const skipBtn = document.getElementById("skip");
const speedSelect = document.getElementById("speed");

// Each slot keeps a permanent slot-number label (real engine vocabulary --
// the log itself says "slot 3" -- so a viewer's cell should say so too) plus
// a content wrapper that renderUnit replaces freely.
for (const side of SIDES) {
  for (let i = 0; i < SLOTS; i++) {
    const el = document.createElement("div");
    el.className = "unit empty";
    el.dataset.slot = String(i);
    const label = document.createElement("span");
    label.className = "slot-no";
    label.textContent = String(i + 1);
    const content = document.createElement("div");
    content.className = "content";
    el.append(content, label);
    rowEl[side].appendChild(el);
  }
}

function slotEl(side, slot) {
  return rowEl[side].children[slot];
}

function renderUnit(side, slot, unit) {
  const el = slotEl(side, slot);
  const content = el.querySelector(".content");
  el.className = "unit" + (unit ? "" : " empty");
  if (!unit) {
    content.innerHTML = "";
    return;
  }
  const badges = unit.keywords
    .map((k) => `<span class="badge ${k}" title="${k}">${KEYWORD_BADGE[k] ?? "?"}</span>`)
    .join("");
  content.innerHTML = `
    <div class="badges">${badges}</div>
    <div class="name">${unit.name}</div>
    <div class="stats"><span class="atk">${unit.attack}</span><span class="sep">/</span><span class="hp">${Math.max(unit.health, 0)}</span></div>
  `;
}

// `board` is a plain bg-sim Board (`{player: {slots}, opposing: {slots}}`) --
// told directly from `initial_board` or a `boards[i]` entry, never mutated.
function renderBoard(board) {
  for (const side of SIDES) {
    const slots = side === "Player" ? board.player.slots : board.opposing.slots;
    for (let i = 0; i < SLOTS; i++) renderUnit(side, i, slots[i]);
  }
}

function popDamage(side, slot, amount) {
  const el = slotEl(side, slot);
  const pop = document.createElement("div");
  pop.className = "dmg-pop";
  pop.textContent = `-${amount}`;
  el.appendChild(pop);
  setTimeout(() => pop.remove(), 700);
}

function pulse(side, slot, cls, duration) {
  const el = slotEl(side, slot);
  el.classList.add(cls);
  setTimeout(() => el.classList.remove(cls), duration);
}

function logLine(text, cls, current) {
  const div = document.createElement("div");
  div.className = "line" + (cls ? ` ${cls}` : "") + (current ? " current" : "");
  div.textContent = text;
  logEl.appendChild(div);
  logEl.scrollTop = logEl.scrollHeight;
}

// Fire every cue's flavor animation (pulse, damage pop, fade, flash) at once.
// None of it mutates anything: the numbers on screen don't move until
// `renderBoard` commits the Beat's already-resolved result at the end of the
// step. A Beat is one slice of time -- its cues are concurrent, so nothing
// here is staggered to imply an order between them.
function animateCues(cues, speed) {
  for (const cue of cues) {
    if (cue.kind === "hit") {
      pulse(cue.attackerSide, cue.attackerSlot, "attacking", speed);
      popDamage(cue.targetSide, cue.targetSlot, cue.damage);
    } else if (cue.kind === "absorb") {
      pulse(cue.attackerSide, cue.attackerSlot, "attacking", speed);
      pulse(cue.targetSide, cue.targetSlot, "shield-flash", speed);
    } else if (cue.kind === "died") {
      pulse(cue.side, cue.slot, "dying", speed);
    } else if (cue.kind === "reborn") {
      pulse(cue.side, cue.slot, "revive-flash", speed);
    }
  }
}

class Player {
  constructor(initialBoard, steps) {
    this.steps = steps;
    this.index = 0;
    this.timer = null;
    renderBoard(initialBoard);
  }

  get done() {
    return this.index >= this.steps.length;
  }

  // Commit one Beat (or the closing summary): log its lines, animate its
  // cues, and reveal its already-resolved board. No board mutation anywhere
  // in here -- everything shown is told, not derived.
  commit(step, { current }) {
    if (step.kind === "beat") {
      logLine(`-- beat ${step.beat} --`, "beat", current);
      for (const cue of step.cues) logLine(narrate(cue), null, current);
      beatValueEl.textContent = String(step.beat);
    } else {
      logLine(`== ${step.outcome} after ${step.beats} beats ==`, "beat", current);
      outcomeEl.textContent = `${step.outcome} — ${step.beats} beats`;
      resultValueEl.textContent = step.outcome;
    }
  }

  step(speed) {
    const step = this.steps[this.index++];
    this.commit(step, { current: true });
    if (step.kind === "beat") {
      animateCues(step.cues, speed);
      setTimeout(() => renderBoard(step.board), speed);
    }
  }

  play() {
    if (this.done) return;
    playBtn.disabled = true;
    skipBtn.disabled = false;
    const tick = () => {
      if (this.done) {
        playBtn.disabled = false;
        return;
      }
      const speed = Number(speedSelect.value);
      this.step(speed);
      this.timer = setTimeout(tick, speed);
    };
    tick();
  }

  skipToEnd() {
    clearTimeout(this.timer);
    let lastBoard = null;
    while (!this.done) {
      const step = this.steps[this.index++];
      this.commit(step, { current: false });
      if (step.kind === "beat") lastBoard = step.board;
    }
    if (lastBoard) renderBoard(lastBoard);
    playBtn.disabled = true;
    skipBtn.disabled = true;
  }
}

// bg-wasm resolves the fight -- the same engine `bg-cli` calls, running in
// the browser instead of shelled out to. `resolve` takes/returns the exact
// JSON `bg-cli` would produce, so there is one wire format either way.
import init, { showcase_board_json, resolve as resolveWasm } from "./pkg/bg_wasm.js";

async function boot() {
  await init();
  // The seed is a Rust u64, which wasm-bindgen maps to a JS BigInt because a
  // JS number can't hold the full range losslessly.
  const fight = JSON.parse(resolveWasm(showcase_board_json(), 1n));

  const steps = buildBeatSteps(fight.log, fight.boards);
  const player = new Player(fight.initial_board, steps);

  playBtn.addEventListener("click", () => player.play());
  skipBtn.addEventListener("click", () => player.skipToEnd());
}

boot();
