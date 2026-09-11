// Plays back a bg-sim `Resolution` (the global FIGHT, loaded from fight.js).
//
// A Resolution is a replay: `initial_board` plus `log` (see
// crates/bg-sim/src/action_phase.rs). Every Event states a change, not a
// state, so this file reconstructs board state by replaying the log against
// a client-side Board -- and the rules it uses to do that (when a shield
// breaks, what Reborn revives with, how compaction packs) are read directly
// off bg-sim's source, not invented. If bg-sim's rules change, this drifts;
// there is no way around that for a log-based replay short of shipping the
// engine itself (see 0.4 in docs/scratchpad/roadmap.md).

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

function cloneUnit(u) {
  return u ? { ...u, keywords: [...u.keywords] } : null;
}

// Mirrors bg-sim's Board/Party: two eight-slot arrays, left-packed, indexed
// from 0 -- the log's slot numbers count from 1, so callers subtract 1.
class ClientBoard {
  constructor(initialBoard) {
    this.player = initialBoard.player.slots.map(cloneUnit);
    this.opposing = initialBoard.opposing.slots.map(cloneUnit);
    // Corpses kept only long enough for a possible Reborn to consume them.
    this._corpses = new Map();
    // Slots whose shield absorbed something this Beat. Broken at the top of
    // the next -- see bg-sim's `break_spent_shields`.
    this._shieldsToBreak = new Set();
  }

  side(name) {
    return name === "Player" ? this.player : this.opposing;
  }

  startBeat() {
    for (const key of this._shieldsToBreak) {
      const [side, slot] = key.split(":");
      const unit = this.side(side)[Number(slot)];
      if (unit) unit.keywords = unit.keywords.filter((k) => k !== "DivineShield");
    }
    this._shieldsToBreak.clear();
  }

  hit(side, slot, damage) {
    const unit = this.side(side)[slot];
    if (unit) unit.health -= damage;
  }

  absorb(side, slot) {
    this._shieldsToBreak.add(`${side}:${slot}`);
  }

  die(side, slot) {
    const arr = this.side(side);
    this._corpses.set(`${side}:${slot}`, arr[slot]);
    arr[slot] = null;
  }

  revive(side, slot, name) {
    const key = `${side}:${slot}`;
    const corpse = this._corpses.get(key);
    this._corpses.delete(key);
    this.side(side)[slot] = {
      ...corpse,
      name,
      health: 1,
      poisoned: false,
      reborn_spent: true,
      keywords: corpse.keywords.filter((k) => k !== "Reborn"),
    };
  }

  compact(side) {
    const arr = this.side(side);
    const packed = arr.filter((u) => u !== null);
    while (packed.length < SLOTS) packed.push(null);
    arr.splice(0, SLOTS, ...packed);
  }
}

// Turn the flat Event log into animation steps. A Struck/StruckBack is
// immediately followed by its ShieldAbsorbed when one happens -- bg-sim's
// `hit()` pushes them as one atomic pair and nothing else can land between
// them -- so this is the one place the log is read with a lookahead of one.
function buildSteps(log) {
  const steps = [];
  for (let i = 0; i < log.length; i++) {
    const [type, data] = Object.entries(log[i])[0];
    if (type === "Struck" || type === "StruckBack") {
      const attackerSide = data.by;
      const targetSide = otherSide(data.by);
      const attackerSlot =
        type === "Struck" ? data.attacker_slot - 1 : data.slot - 1;
      const targetSlot = data.target_slot - 1;
      const next = log[i + 1] && Object.entries(log[i + 1])[0];
      if (
        next &&
        next[0] === "ShieldAbsorbed" &&
        next[1].side === targetSide &&
        next[1].slot - 1 === targetSlot
      ) {
        steps.push({
          kind: "absorb",
          attackerSide,
          attackerSlot,
          targetSide,
          targetSlot,
        });
        i++; // consume the paired ShieldAbsorbed
      } else {
        steps.push({
          kind: "hit",
          attackerSide,
          attackerSlot,
          targetSide,
          targetSlot,
          damage: data.damage,
        });
      }
      continue;
    }
    if (type === "BeatBegan") steps.push({ kind: "beat", beat: data.beat });
    else if (type === "Compacted") steps.push({ kind: "compact", side: data.side });
    else if (type === "Died")
      steps.push({ kind: "died", side: data.side, slot: data.slot - 1, name: data.name });
    else if (type === "Reborn")
      steps.push({ kind: "reborn", side: data.side, slot: data.slot - 1, name: data.name });
    else if (type === "Ended")
      steps.push({ kind: "ended", outcome: data.outcome, beats: data.beats });
    else if (type === "ShieldAbsorbed") {
      // Only reachable if a ShieldAbsorbed appears without a preceding
      // Struck/StruckBack, which bg-sim never emits. Kept as a visible
      // failure rather than a silently dropped event.
      throw new Error("unpaired ShieldAbsorbed in log");
    }
  }
  return steps;
}

function otherSide(side) {
  return side === "Player" ? "Opposing" : "Player";
}

function narrate(step) {
  switch (step.kind) {
    case "beat":
      return { text: `-- beat ${step.beat} --`, cls: "beat" };
    case "hit":
      return {
        text: `  ${step.attackerSide.toLowerCase()} slot ${step.attackerSlot + 1} strikes ${step.targetSide.toLowerCase()} slot ${step.targetSlot + 1} for ${step.damage}`,
      };
    case "absorb":
      return {
        text: `  ${step.targetSide.toLowerCase()} slot ${step.targetSlot + 1} absorbs it on its shield`,
      };
    case "died":
      return { text: `  ${step.side.toLowerCase()} slot ${step.slot + 1} (${step.name}) dies` };
    case "reborn":
      return {
        text: `  ${step.side.toLowerCase()} slot ${step.slot + 1} returns as ${step.name} with 1 health`,
      };
    case "compact":
      return { text: `  the ${step.side.toLowerCase()} party closes ranks` };
    case "ended":
      return { text: `== ${step.outcome} after ${step.beats} beats ==`, cls: "beat" };
    default:
      return { text: "" };
  }
}

// ---------------------------------------------------------------------------
// Rendering + playback
// ---------------------------------------------------------------------------

const rowEl = { Player: document.getElementById("player-row"), Opposing: document.getElementById("opposing-row") };
const logEl = document.getElementById("log");
const outcomeEl = document.getElementById("outcome");
const playBtn = document.getElementById("play");
const skipBtn = document.getElementById("skip");
const speedSelect = document.getElementById("speed");

for (const side of SIDES) {
  for (let i = 0; i < SLOTS; i++) {
    const el = document.createElement("div");
    el.className = "unit empty";
    el.dataset.slot = String(i);
    rowEl[side].appendChild(el);
  }
}

function slotEl(side, slot) {
  return rowEl[side].children[slot];
}

function renderUnit(side, slot, unit) {
  const el = slotEl(side, slot);
  el.className = "unit" + (unit ? "" : " empty");
  if (!unit) {
    el.innerHTML = "";
    return;
  }
  const badges = unit.keywords
    .map((k) => `<span class="badge ${k}" title="${k}">${KEYWORD_BADGE[k] ?? "?"}</span>`)
    .join("");
  el.innerHTML = `
    <div class="badges">${badges}</div>
    <div class="name">${unit.name}</div>
    <div class="stats"><span class="atk">${unit.attack}</span><span class="sep">/</span><span class="hp">${Math.max(unit.health, 0)}</span></div>
  `;
}

function renderAll(board) {
  for (const side of SIDES) {
    for (let i = 0; i < SLOTS; i++) renderUnit(side, i, board.side(side)[i]);
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

class Player {
  constructor(board, steps) {
    this.board = board;
    this.steps = steps;
    this.index = 0;
    this.timer = null;
    renderAll(board);
  }

  get done() {
    return this.index >= this.steps.length;
  }

  // Apply one step's board mutation, without animation. Used by both the
  // animated path and "skip to end".
  apply(step) {
    switch (step.kind) {
      case "beat":
        this.board.startBeat();
        break;
      case "hit":
        this.board.hit(step.targetSide, step.targetSlot, step.damage);
        break;
      case "absorb":
        this.board.absorb(step.targetSide, step.targetSlot);
        break;
      case "died":
        this.board.die(step.side, step.slot);
        break;
      case "reborn":
        this.board.revive(step.side, step.slot, step.name);
        break;
      case "compact":
        this.board.compact(step.side);
        break;
      case "ended":
        break;
    }
  }

  logLine(step, current) {
    const { text, cls } = narrate(step);
    const div = document.createElement("div");
    div.className = "line" + (cls ? ` ${cls}` : "") + (current ? " current" : "");
    div.textContent = text;
    logEl.appendChild(div);
    logEl.scrollTop = logEl.scrollHeight;
  }

  // Advance one step with animation, returning the delay before the next.
  step(speed) {
    const step = this.steps[this.index++];
    this.logLine(step, true);

    if (step.kind === "hit") {
      pulse(step.attackerSide, step.attackerSlot, "attacking", speed);
      popDamage(step.targetSide, step.targetSlot, step.damage);
      setTimeout(() => {
        this.apply(step);
        renderUnit(step.targetSide, step.targetSlot, this.board.side(step.targetSide)[step.targetSlot]);
      }, speed / 2);
    } else if (step.kind === "absorb") {
      pulse(step.attackerSide, step.attackerSlot, "attacking", speed);
      pulse(step.targetSide, step.targetSlot, "shield-flash", speed);
      this.apply(step);
    } else if (step.kind === "died") {
      pulse(step.side, step.slot, "dying", speed);
      setTimeout(() => {
        this.apply(step);
        renderUnit(step.side, step.slot, null);
      }, speed);
    } else if (step.kind === "reborn") {
      this.apply(step);
      renderUnit(step.side, step.slot, this.board.side(step.side)[step.slot]);
      pulse(step.side, step.slot, "revive-flash", speed);
    } else if (step.kind === "compact") {
      this.apply(step);
      renderAll(this.board);
    } else if (step.kind === "beat") {
      this.apply(step);
      renderAll(this.board);
    } else if (step.kind === "ended") {
      outcomeEl.textContent = `${step.outcome} — ${step.beats} beats`;
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
    while (!this.done) {
      const step = this.steps[this.index++];
      this.logLine(step, false);
      this.apply(step);
      if (step.kind === "ended") {
        outcomeEl.textContent = `${step.outcome} — ${step.beats} beats`;
      }
    }
    renderAll(this.board);
    playBtn.disabled = true;
    skipBtn.disabled = true;
  }
}

const board = new ClientBoard(FIGHT.initial_board);
const steps = buildSteps(FIGHT.log);
const player = new Player(board, steps);

playBtn.addEventListener("click", () => player.play());
skipBtn.addEventListener("click", () => player.skipToEnd());
