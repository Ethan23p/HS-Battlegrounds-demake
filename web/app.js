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
//
// Playback is a position in `steps` (one entry per Beat, plus a closing
// "ended" entry), not a one-shot animation: `Player.revealStep(index)` is
// the single place that renders a position, and everything -- board, log,
// readouts, the arrows/numbers overlay -- is rebuilt fresh from `steps` each
// time it runs. Stepping forward, back, replaying from the start, or
// skipping to the end are all just calls to it with a different index.

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
const SIDE_COLOR = { Player: "indigo", Opposing: "rust" };

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

const battlefieldEl = document.getElementById("battlefield");
const rowEl = { Player: document.getElementById("player-row"), Opposing: document.getElementById("opposing-row") };
const opposingRankLabelEl = document.getElementById("opposing-rank-label");
const arrowSvgEl = document.querySelector(".overlay.arrows");
const arrowLinesEl = document.getElementById("arrow-lines");
const logEl = document.getElementById("log");
const logToggleEl = document.getElementById("log-toggle");
const outcomeEl = document.getElementById("outcome");
const beatValueEl = document.getElementById("beat-value");
const resultValueEl = document.getElementById("result-value");
const goldValueEl = document.getElementById("gold-value");
const tierValueEl = document.getElementById("tier-value");
const recordValueEl = document.getElementById("record-value");
const shopControlsEl = document.getElementById("shop-controls");
const playbackControlsEl = document.getElementById("playback-controls");
const runOverControlsEl = document.getElementById("run-over-controls");
const runOverTextEl = document.getElementById("run-over-text");
const shopMessageEl = document.getElementById("shop-message");
const fightBtn = document.getElementById("fight");
const rerollBtn = document.getElementById("reroll");
const freezeBtn = document.getElementById("freeze");
const upgradeTavernBtn = document.getElementById("upgrade-tavern");
const upgradeCostEl = document.getElementById("upgrade-cost");
const continueBtn = document.getElementById("continue");
const newRunBtn = document.getElementById("new-run");
const prevBtn = document.getElementById("prev");
const nextBtn = document.getElementById("next");
const playBtn = document.getElementById("play");
const resetBtn = document.getElementById("reset");
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

// `opts.offer` (Prep, opposing row): a shop offer rather than a Unit -- adds
// the freeze toggle and, when frozen, a highlight. `opts.sellable` (Prep,
// player row): the board a run is built on -- adds the sell button. Neither
// applies during a fight, so plain `renderUnit(side, slot, unit)` (opts
// defaulting to {}) is what the fight viewer still calls.
function renderUnit(side, slot, unit, opts = {}) {
  const el = slotEl(side, slot);
  const content = el.querySelector(".content");
  // A Unit at 0 health hasn't died yet -- bg-sim buries it at the top of the
  // *next* Beat (Departure 2). "critical" is what makes that rule visible
  // instead of the card just silently reading 0 until it vanishes.
  const critical = unit && unit.health <= 0;
  el.className =
    "unit" +
    (unit ? "" : " empty") +
    (critical ? " critical" : "") +
    (opts.offer ? " offer" : "") +
    (opts.offer?.frozen ? " frozen" : "") +
    (opts.offer?.affordable === false ? " unaffordable" : "");
  if (!unit) {
    content.innerHTML = "";
    return;
  }
  const badges = unit.keywords
    .map((k) => `<span class="badge ${k}" title="${k}">${KEYWORD_BADGE[k] ?? "?"}</span>`)
    .join("");
  const corner = opts.sellable ? `<button class="sell-btn" data-action="sell" title="Sell">×</button>` : "";
  content.innerHTML = `
    <div class="badges">${badges}</div>
    <div class="name">${unit.name}</div>
    <div class="stats"><span class="atk">${unit.attack}</span><span class="sep">/</span><span class="hp">${Math.max(unit.health, 0)}</span></div>
    ${corner}
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

function pulse(side, slot, cls, duration) {
  const el = slotEl(side, slot);
  el.classList.add(cls);
  setTimeout(() => el.classList.remove(cls), duration);
}

// Flashed on a hit's target right after its new (already-resolved) health is
// on screen, so the floating damage callout and the stat it explains read as
// one event. Double rAF: the class has to actually paint before the CSS
// transition removing it can animate anything.
function flashHp(side, slot) {
  const el = slotEl(side, slot).querySelector(".hp");
  if (!el) return;
  el.classList.add("hit-flash");
  requestAnimationFrame(() => requestAnimationFrame(() => el.classList.remove("hit-flash")));
}

// "The party closes ranks" is a real rule (left-anchoring) with no visual of
// its own otherwise -- restarts its shimmer even if triggered again shortly
// after, via the reflow-then-reclass trick.
function shimmerRank(side) {
  const row = rowEl[side];
  row.classList.remove("closed-ranks");
  void row.offsetWidth;
  row.classList.add("closed-ranks");
  setTimeout(() => row.classList.remove("closed-ranks"), 550);
}

function logLine(text, cls, current) {
  const div = document.createElement("div");
  div.className = "line" + (cls ? ` ${cls}` : "") + (current ? " current" : "");
  div.textContent = text;
  logEl.appendChild(div);
  logEl.scrollTop = logEl.scrollHeight;
}

// ---------------------------------------------------------------------------
// Arrows + damage callouts for the Beat on screen
//
// Both read positions from the DOM (`getBoundingClientRect`), not from any
// model of "where a slot is" -- the grid already knows that, so nothing here
// has to duplicate it. Everything drawn belongs to whichever Beat is
// currently revealed; `clearOverlay` wipes it before the next one draws.
// ---------------------------------------------------------------------------

function clearOverlay() {
  // An <svg> with no explicit size defaults to a 300x150 viewport and clips
  // anything drawn outside it, regardless of how large `inset: 0` stretches
  // its CSS box -- so without this, content past that default height was
  // silently cut off even though it was positioned correctly. Set fresh each
  // time rather than once, so a resize between reveals can't leave it stale.
  //
  // `offsetWidth`/`offsetHeight` (layout size), not `getBoundingClientRect`
  // (screen size) -- the forced-landscape media query (see style.css)
  // rotates `.stage` 90deg on a portrait phone, and `getBoundingClientRect`
  // reports the *post-rotation* box. This SVG is a normal descendant of that
  // rotated `.stage`, so anything drawn inside it in local coordinates gets
  // that same rotation applied once by the browser -- sizing/positioning it
  // from already-rotated numbers rotated it a second time.
  const w = battlefieldEl.offsetWidth;
  const h = battlefieldEl.offsetHeight;
  arrowSvgEl.setAttribute("width", w);
  arrowSvgEl.setAttribute("height", h);
  arrowSvgEl.setAttribute("viewBox", `0 0 ${w} ${h}`);

  arrowLinesEl.innerHTML = "";
  // Damage labels live on the card they describe (placeDamageLabel), not in
  // a shared layer, so clearing them means finding them there.
  for (const el of document.querySelectorAll(".unit .dmg-label")) el.remove();
}

// Walks the `offsetParent` chain rather than `getBoundingClientRect`, so the
// result is in local, pre-rotation layout coordinates -- see clearOverlay's
// comment. Both the arrow overlay (an SVG whose local space gets the same
// ambient rotation applied once by the browser) and a card's own clash
// transform (a `translate()` composed *underneath* that same ambient
// rotation) need coordinates in this space, not screen space, or the
// rotation ends up applied twice.
function offsetRelativeTo(el, ancestor) {
  let x = 0;
  let y = 0;
  for (let node = el; node && node !== ancestor; node = node.offsetParent) {
    x += node.offsetLeft;
    y += node.offsetTop;
  }
  return { x, y };
}

function centerOf(side, slot) {
  const el = slotEl(side, slot);
  const { x, y } = offsetRelativeTo(el, battlefieldEl);
  return { x: x + el.offsetWidth / 2, y: y + el.offsetHeight / 2 };
}

function drawArrow(attackerSide, attackerSlot, targetSide, targetSlot, absorbed) {
  const a = centerOf(attackerSide, attackerSlot);
  const b = centerOf(targetSide, targetSlot);
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len = Math.hypot(dx, dy) || 1;
  const pad = 26; // clear of the cards themselves, so the arrowhead lands at an edge, not buried in stats
  const ux = dx / len;
  const uy = dy / len;
  const start = { x: a.x + ux * pad, y: a.y + uy * pad };
  const end = { x: b.x - ux * pad, y: b.y - uy * pad };

  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  path.setAttribute("d", `M${start.x},${start.y} L${end.x},${end.y}`);
  path.setAttribute("marker-end", `url(#arrow-${attackerSide.toLowerCase()})`);
  path.style.stroke = `var(--${SIDE_COLOR[attackerSide]})`;
  if (absorbed) path.classList.add("absorbed");
  arrowLinesEl.appendChild(path);
}

// Its own system, anchored on the card it describes (see .dmg-label in
// style.css) rather than placed by battlefield-relative math -- a child of
// the struck card, stacked upward via --stack when more than one hits it in
// the same Beat.
function placeDamageLabel(side, slot, text, absorbed, stackIndex) {
  const el = document.createElement("div");
  el.className = "dmg-label" + (absorbed ? " absorbed" : "");
  el.textContent = text;
  el.style.setProperty("--stack", String(stackIndex));
  slotEl(side, slot).appendChild(el);
}

// A Beat's exchange between two cards is one relationship, not one arrow per
// blow -- a Struck and its StruckBack are the same clash seen from both
// sides, so they collapse to a single, unidirectional arrow (the initiating
// blow's own direction) per pair engaged this Beat. Damage numbers stay
// per-blow: each hit still needs its own figure, just not its own arrow.
function drawBeatOverlay(step) {
  const stacked = new Map(); // "side:slot" -> how many labels already placed there this Beat
  const pairs = new Map(); // unordered {attacker,target} pair -> that pair's one arrow

  for (const cue of step.cues) {
    if (cue.kind !== "hit" && cue.kind !== "absorb") continue;
    const absorbed = cue.kind === "absorb";

    const aKey = `${cue.attackerSide}:${cue.attackerSlot}`;
    const bKey = `${cue.targetSide}:${cue.targetSlot}`;
    const pairKey = [aKey, bKey].sort().join("|");
    // The initiating blow (not the retaliation) decides the pair's arrow
    // direction; keep the first cue seen until a non-answering one arrives
    // to correct it, in case the log ever orders them the other way.
    const existing = pairs.get(pairKey);
    if (!existing || (existing.answering && !cue.answering)) {
      pairs.set(pairKey, {
        attackerSide: cue.attackerSide,
        attackerSlot: cue.attackerSlot,
        targetSide: cue.targetSide,
        targetSlot: cue.targetSlot,
        absorbed,
        answering: cue.answering,
      });
    }

    const key = `${cue.targetSide}:${cue.targetSlot}`;
    const stackIndex = stacked.get(key) ?? 0;
    stacked.set(key, stackIndex + 1);
    placeDamageLabel(cue.targetSide, cue.targetSlot, absorbed ? "blocked" : `-${cue.damage}`, absorbed, stackIndex);
  }

  for (const pair of pairs.values()) {
    drawArrow(pair.attackerSide, pair.attackerSlot, pair.targetSide, pair.targetSlot, pair.absorbed);
  }
}

// The actual BG attack motion: a card snaps toward the one it's striking
// and rubber-bands back (.unit.attacking's `clash` keyframe in style.css);
// the struck card gets a smaller recoil in the same direction, timed to
// land as the clash arrives (`.unit.recoiling`'s `recoil` keyframe). Both
// read the same --clash-x/--clash-y, the real attacker->target vector (the
// same one drawArrow uses), so a card visibly moves toward the card it's
// actually hitting rather than a generic "up" or "down".
function setClashVector(side, slot, attackerSide, attackerSlot, targetSide, targetSlot) {
  const a = centerOf(attackerSide, attackerSlot);
  const b = centerOf(targetSide, targetSlot);
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len = Math.hypot(dx, dy) || 1;
  const reach = 16; // px a card snaps toward its target
  const el = slotEl(side, slot);
  el.style.setProperty("--clash-x", `${(dx / len) * reach}px`);
  el.style.setProperty("--clash-y", `${(dy / len) * reach}px`);
}

// Transient flourish only -- attacking/shield/death/revive pulses that play
// out *before* a Beat's result is revealed. Nothing here changes what's on
// screen; `Player.revealStep` does that, once, atomically.
function animateCues(cues, speed) {
  for (const cue of cues) {
    if (cue.kind === "hit" || cue.kind === "absorb") {
      setClashVector(cue.attackerSide, cue.attackerSlot, cue.attackerSide, cue.attackerSlot, cue.targetSide, cue.targetSlot);
      setClashVector(cue.targetSide, cue.targetSlot, cue.attackerSide, cue.attackerSlot, cue.targetSide, cue.targetSlot);
      pulse(cue.attackerSide, cue.attackerSlot, "attacking", speed);
      pulse(cue.targetSide, cue.targetSlot, "recoiling", speed);
      if (cue.kind === "absorb") pulse(cue.targetSide, cue.targetSlot, "shield-flash", speed);
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
    this.timer = null;
    // boardAfter[k] = the Board once steps[0..k-1] have been committed.
    this.boardAfter = [initialBoard];
    let running = initialBoard;
    for (const step of steps) {
      if (step.kind === "beat") running = step.board;
      this.boardAfter.push(running);
    }
    this.revealStep(0);
  }

  get atStart() {
    return this.index === 0;
  }

  get done() {
    return this.index >= this.steps.length;
  }

  // The one place that renders a position. Everything is rebuilt fresh from
  // `steps` each call -- board, log, readouts, overlay -- so jumping to any
  // index by any path (stepping, replaying, skipping) always lands on
  // exactly the same result.
  revealStep(index) {
    this.index = index;
    renderBoard(this.boardAfter[index]);
    this.rebuildLog();
    this.updateReadouts();

    // The overlay belongs to whichever Beat most recently happened, even
    // when `last` is the closing "ended" entry (always preceded directly by
    // that Beat, since `steps` is [...beats, ended]) -- computed from
    // `steps`/`index` alone rather than left over from whatever the DOM
    // happened to already show, so jumping straight here (Skip to end from
    // anywhere) still reveals the actual finishing blow, not a stale one.
    const last = index > 0 ? this.steps[index - 1] : null;
    const beatForOverlay = last?.kind === "beat" ? last : index >= 2 ? this.steps[index - 2] : null;

    clearOverlay();
    if (beatForOverlay?.kind === "beat") drawBeatOverlay(beatForOverlay);
    if (last?.kind === "beat") {
      for (const cue of last.cues) {
        if (cue.kind === "hit") flashHp(cue.targetSide, cue.targetSlot);
        else if (cue.kind === "compact") shimmerRank(cue.side);
      }
    }
    this.updateButtons();
  }

  rebuildLog() {
    logEl.innerHTML = "";
    const committed = this.steps.slice(0, this.index);
    committed.forEach((step, i) => {
      const isLast = i === committed.length - 1;
      if (step.kind === "beat") {
        logLine(`-- beat ${step.beat} --`, "beat", isLast);
        for (const cue of step.cues) logLine(narrate(cue), null, isLast);
      } else {
        logLine(`== ${step.outcome} after ${step.beats} beats ==`, "beat", isLast);
      }
    });
    logEl.scrollTop = logEl.scrollHeight;
  }

  updateReadouts() {
    let beat = null;
    let ended = null;
    for (let i = 0; i < this.index; i++) {
      const step = this.steps[i];
      if (step.kind === "beat") beat = step.beat;
      else ended = step;
    }
    beatValueEl.textContent = beat !== null ? String(beat) : "—";
    resultValueEl.textContent = ended ? ended.outcome : "—";
    outcomeEl.textContent = ended ? `${ended.outcome} — ${ended.beats} beats` : "";
  }

  updateButtons() {
    prevBtn.disabled = this.atStart;
    nextBtn.disabled = this.done;
    playBtn.disabled = this.done;
    skipBtn.disabled = this.done;
  }

  stopAuto() {
    clearTimeout(this.timer);
    this.timer = null;
  }

  play() {
    if (this.done) return;
    this.stopAuto();
    playBtn.disabled = true;
    const tick = () => {
      if (this.done) {
        this.updateButtons();
        return;
      }
      const speed = Number(speedSelect.value);
      const upcoming = this.steps[this.index];
      if (upcoming.kind === "beat") animateCues(upcoming.cues, speed);
      this.timer = setTimeout(() => {
        this.revealStep(this.index + 1);
        tick();
      }, speed);
    };
    tick();
  }

  next() {
    if (this.done) return;
    this.stopAuto();
    this.revealStep(this.index + 1);
  }

  prev() {
    if (this.atStart) return;
    this.stopAuto();
    this.revealStep(this.index - 1);
  }

  skipToEnd() {
    this.stopAuto();
    this.revealStep(this.steps.length);
  }

  reset() {
    this.stopAuto();
    this.revealStep(0);
  }
}

// ---------------------------------------------------------------------------
// Shop: the Prep Phase. The opposing row is repurposed as the shop while
// it's on screen (relabeled "Shop", swapped back to "Opposing party" the
// moment a fight starts); the player row is the actual run -- it persists
// round to round, the same cards carrying over, only buy/sell/reorder
// touching it. Every action (buy, sell, reroll, freeze, upgrade) is told to
// bg-wasm and the *entire next RunState* comes back -- this class never
// computes gold, pool counts or a shop draw itself, only renders whatever
// RunState it was just handed.
// ---------------------------------------------------------------------------

class Shop {
  constructor(run, roster, onFight) {
    this.run = run;
    this.roster = roster; // the shop's UnitDef list, for offer name/stats/keywords by id
    this.onFight = onFight;
    this.drag = null;
    this.msgTimer = null;

    for (let i = 0; i < SLOTS; i++) {
      slotEl("Opposing", i).addEventListener("click", (e) => this.onOfferClick(e, i));
      slotEl("Player", i).addEventListener("click", (e) => this.onBoardClick(e, i));
      slotEl("Player", i).addEventListener("pointerdown", (e) => this.onPointerDown(e, i));
    }
    rerollBtn.addEventListener("click", () => this.reroll());
    freezeBtn.addEventListener("click", () => this.toggleFreeze());
    upgradeTavernBtn.addEventListener("click", () => this.upgradeTavern());
    fightBtn.addEventListener("click", () => this.fight());
  }

  defById(id) {
    return this.roster.find((d) => d.id === id);
  }

  // A shop offer is a UnitDef, not a Unit -- but it has exactly the fields
  // renderUnit reads (name/attack/health/keywords), so no adapting needed
  // beyond picking those fields out.
  offerAsUnit(defId) {
    const { name, attack, health, keywords } = this.defById(defId);
    return { name, attack, health, keywords };
  }

  render() {
    goldValueEl.textContent = String(this.run.gold);
    tierValueEl.textContent = String(this.run.tavern_tier);
    recordValueEl.textContent = `${this.run.wins}-${this.run.losses}`;

    const atMaxTier = this.run.tavern_tier >= 6;
    const upgradeCost = this.run.tavern_tier * 5;
    upgradeCostEl.textContent = atMaxTier ? "max" : `${upgradeCost}g`;
    upgradeTavernBtn.disabled = atMaxTier || this.run.gold < upgradeCost;
    rerollBtn.disabled = this.run.gold < 1;
    const shopFrozen = this.run.shop.length > 0 && this.run.shop.every((s) => s.frozen);
    freezeBtn.classList.toggle("active", shopFrozen);
    freezeBtn.disabled = this.run.shop.length === 0;

    for (let i = 0; i < SLOTS; i++) {
      const offer = this.run.shop[i];
      const offerUnit = offer ? this.offerAsUnit(offer.def) : null;
      renderUnit("Opposing", i, offerUnit, {
        offer: offer ? { frozen: offer.frozen, affordable: this.run.gold >= 3 } : null,
      });

      const boardUnit = this.run.board.slots[i];
      renderUnit("Player", i, boardUnit, { sellable: !!boardUnit });
      slotEl("Player", i).classList.toggle("draggable", !!boardUnit);
    }
  }

  showMessage(text) {
    shopMessageEl.textContent = text;
    clearTimeout(this.msgTimer);
    this.msgTimer = setTimeout(() => {
      shopMessageEl.textContent = "";
    }, 2500);
  }

  // Every shop action follows the same shape: send the current RunState (and
  // whatever the action needs) to bg-wasm, replace it with whatever comes
  // back, re-render. On a refusal (not enough gold, board full, ...) bg-wasm
  // throws with the engine's own reason -- shown, not silently swallowed.
  call(fn, ...args) {
    try {
      this.run = JSON.parse(fn(JSON.stringify(this.run), ...args));
      this.render();
    } catch (err) {
      this.showMessage(err?.message ?? String(err));
    }
  }

  buy(offer) {
    this.call(shopBuy, offer);
  }

  sell(slot) {
    this.call(shopSell, slot);
  }

  reroll() {
    this.call(shopReroll);
  }

  toggleFreeze() {
    this.call(shopToggleFreeze);
  }

  upgradeTavern() {
    this.call(shopUpgradeTavern);
  }

  onOfferClick(e, i) {
    if (this.run.shop[i]) this.buy(i);
  }

  onBoardClick(e, i) {
    if (e.target.closest(".sell-btn")) this.sell(i);
  }

  // Drag-to-reorder the board -- the same Pointer Events pattern 0.2 built
  // for this (one code path for mouse, touch and pen), now permuting
  // `run.board.slots` (bg-wasm's `[Option<Unit>; 8]`) directly instead of a
  // client-side roster array. Still never needs bg-sim's packing rule: a
  // reorder only permutes Units already there, so `slots` stays exactly
  // 8 long and left-packed throughout (splice removes one, then reinserts
  // that same one -- length is never actually disturbed).
  onPointerDown(e, slot) {
    // A press starting on the sell button is a click, not a drag -- letting
    // it start a drag anyway means onPointerUp's render() replaces the
    // button's own DOM node (a fresh .content) before the browser's
    // following click event can land on it, so the sell silently never
    // fires.
    if (e.target.closest(".sell-btn")) return;
    if (!this.run.board.slots[slot]) return;
    const el = slotEl("Player", slot);
    el.setPointerCapture(e.pointerId);
    const rect = el.getBoundingClientRect();
    this.drag = {
      pointerId: e.pointerId,
      fromSlot: slot,
      el,
      grabX: e.clientX - rect.left,
      grabY: e.clientY - rect.top,
      originLeft: rect.left,
      originTop: rect.top,
    };
    el.classList.add("dragging");
    const move = (ev) => this.onPointerMove(ev);
    const up = (ev) => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      this.onPointerUp(ev);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  }

  onPointerMove(e) {
    if (!this.drag) return;
    const dx = e.clientX - this.drag.grabX - this.drag.originLeft;
    const dy = e.clientY - this.drag.grabY - this.drag.originTop;
    this.drag.el.style.transform = `translate(${dx}px, ${dy}px)`;
    this.highlightDropTarget(this.slotAtPoint(e.clientX, e.clientY));
  }

  onPointerUp(e) {
    if (!this.drag) return;
    const over = this.slotAtPoint(e.clientX, e.clientY);
    this.drag.el.releasePointerCapture(this.drag.pointerId);
    this.drag.el.classList.remove("dragging");
    this.drag.el.style.transform = "";
    this.highlightDropTarget(null);

    const occupied = this.boardOccupiedCount();
    if (over !== null && over !== this.drag.fromSlot && over < occupied) {
      const slots = this.run.board.slots;
      const [moved] = slots.splice(this.drag.fromSlot, 1);
      slots.splice(over, 0, moved);
    }
    this.drag = null;
    this.render(); // also the snap-back, when the drop wasn't a valid target
  }

  boardOccupiedCount() {
    return this.run.board.slots.filter((u) => u !== null).length;
  }

  slotAtPoint(x, y) {
    // Excludes the card being dragged: its own rect has been CSS-transformed
    // to follow the pointer, so it visually sits wherever the pointer is --
    // testing it too would always match slot 0 against itself before ever
    // reaching the actual card underneath.
    const occupied = this.boardOccupiedCount();
    for (let i = 0; i < occupied; i++) {
      if (i === this.drag?.fromSlot) continue;
      const r = slotEl("Player", i).getBoundingClientRect();
      if (x >= r.left && x <= r.right && y >= r.top && y <= r.bottom) return i;
    }
    return null;
  }

  highlightDropTarget(slot) {
    const occupied = this.boardOccupiedCount();
    for (let i = 0; i < occupied; i++) {
      slotEl("Player", i).classList.toggle("drop-target", i === slot && i !== this.drag?.fromSlot);
    }
  }

  fight() {
    this.onFight(endTurn(JSON.stringify(this.run)));
  }
}

// bg-wasm runs both halves of a round -- the Prep Phase's shop and the
// Action Phase's fight -- so nothing here reimplements engine rules to
// interpret a RunState or a Resolution; every action sends the current one
// and gets the next one back.
import init, {
  resolve as resolveWasm,
  start_run as startRunWasm,
  shop_roster_json as shopRosterJson,
  shop_buy as shopBuy,
  shop_sell as shopSell,
  shop_reroll as shopReroll,
  shop_toggle_freeze as shopToggleFreeze,
  shop_upgrade_tavern as shopUpgradeTavern,
  end_turn as endTurn,
  apply_fight_result as applyFightResult,
  start_new_round as startNewRound,
} from "./pkg/bg_wasm.js";

async function boot() {
  await init();
  const roster = JSON.parse(shopRosterJson());

  let shop = null;
  let player = null; // the current fight's Player, once one exists
  let pendingResolutionJson = null; // this fight's Resolution, kept raw for apply_fight_result

  function newRun() {
    // The seed is a Rust u64, mapped to a JS BigInt because a JS number
    // can't hold the full range losslessly.
    const run = JSON.parse(startRunWasm(BigInt(Date.now())));
    shop = new Shop(run, roster, (boardJson) => enterFight(boardJson));
    enterShop();
  }

  function enterShop() {
    player?.stopAuto();
    player = null;
    pendingResolutionJson = null;
    logEl.innerHTML = "";
    outcomeEl.textContent = "";
    beatValueEl.textContent = "—";
    resultValueEl.textContent = "—";
    clearOverlay();
    opposingRankLabelEl.textContent = "Shop";
    shopControlsEl.hidden = false;
    playbackControlsEl.hidden = true;
    runOverControlsEl.hidden = true;
    shopMessageEl.textContent = "";
    shop.render();
  }

  function enterFight(boardJson) {
    // A fresh seed each fight, so re-fighting the same board (Reset, on the
    // fight itself) still replays exactly, but a new round never repeats
    // the last one's rolls.
    const seed = BigInt(Date.now());
    pendingResolutionJson = resolveWasm(boardJson, seed);
    const fight = JSON.parse(pendingResolutionJson);
    const steps = buildBeatSteps(fight.log, fight.boards);
    player = new Player(fight.initial_board, steps);
    opposingRankLabelEl.textContent = "Opposing party";
    shopControlsEl.hidden = true;
    playbackControlsEl.hidden = false;
  }

  // "Continue": tell the run what the fight decided, then either the run is
  // over (best of 3 -- two wins or two losses) or it's the next round's shop.
  function afterFight() {
    shop.run = JSON.parse(applyFightResult(JSON.stringify(shop.run), pendingResolutionJson));
    if (shop.run.wins >= 2 || shop.run.losses >= 2) {
      enterRunOver();
    } else {
      shop.run = JSON.parse(startNewRound(JSON.stringify(shop.run)));
      enterShop();
    }
  }

  function enterRunOver() {
    player?.stopAuto();
    shopControlsEl.hidden = true;
    playbackControlsEl.hidden = true;
    runOverControlsEl.hidden = false;
    runOverTextEl.textContent =
      shop.run.wins >= 2
        ? `Run won! Final record ${shop.run.wins}-${shop.run.losses}.`
        : `Run lost. Final record ${shop.run.wins}-${shop.run.losses}.`;
  }

  prevBtn.addEventListener("click", () => player?.prev());
  nextBtn.addEventListener("click", () => player?.next());
  playBtn.addEventListener("click", () => player?.play());
  resetBtn.addEventListener("click", () => player?.reset());
  skipBtn.addEventListener("click", () => player?.skipToEnd());
  continueBtn.addEventListener("click", () => afterFight());
  newRunBtn.addEventListener("click", () => newRun());
  logToggleEl.addEventListener("click", () => {
    const expanded = logToggleEl.getAttribute("aria-expanded") === "true";
    logToggleEl.setAttribute("aria-expanded", String(!expanded));
  });

  newRun();
}

boot();
