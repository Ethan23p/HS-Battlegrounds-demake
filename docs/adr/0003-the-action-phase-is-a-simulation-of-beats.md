---
status: accepted
---

# The Action Phase is a left-to-right sweep of Beats

In Battlegrounds, combat alternates: one side attacks, then the other, with a coin flip
breaking the tie over who starts. Whether a Unit ever acts depends on whether something
killed it first, so outcomes hinge on an ordering players can't fully see.

Instead, the Action Phase **sweeps Slot by Slot from left to right**. At each Slot, the
two facing Units resolve **synchronously** — that moment is a **Beat**.

Two ideas, easy to confuse: a framing — an auto-battler's combat is *a simulation playing
out*, and discrete per-unit actions are a holdover from card games, not something the form
demands — and the mechanism that serves it — a positional sweep, so the narrative is
sequential (Beat follows Beat, left to right) while resolution within any one moment is
symmetric. You watch it unfold; in any given Slot, nobody goes first.

## Consequences

- **The opening coin flip disappears**, along with a large share of the Action Phase's
  variance — much of Battlegrounds' randomness is really just "who swung first."
- **There is no targeting decision at all.** Slot *i* faces Slot *i*. Every scrap of
  target-selection randomness leaves the game, and it makes **ordering the Party the
  central skill** of the Prep Phase, giving the shopping half real depth with no extra
  machinery.
- **Trades become mutual.** Two 3/3s facing each other kill each other; intuitions
  carried from Battlegrounds about a good board are suspect.
- **A Beat is a pure function** from world-state to world-state, trivially testable, and
  it hands the frontend its pacing: a renderer animates Beats, in order, knowing nothing
  else.
- Poisonous is markedly stronger when every exchange is mutual.
- **The keywords survive, contrary to a first reading.** Windfury is whatever a Unit
  would do once in a Beat, done twice — the Beat *is* the turn it takes twice. Taunt
  keeps meaning because removing target *choice* doesn't remove *position*: Effects may
  still have positional implications, so Taunt is protection of neighbours rather than a
  redirect. Its exact rule is unsettled.

## The sweep, precisely

- The sweep **repeats** from Slot 1 after Slot 8 — a single pass would make health nearly
  meaningless, since few Units would ever be struck twice.
- Deaths apply at the **end of the Beat** that caused them, so a Unit killed in Slot 3 is
  visibly gone by Slot 4. Deferring to the sweep's end would have corpses fighting on.
- A Slot occupied by only one side has that Unit **strike the opposing Player directly**
  — Battlegrounds' damage-on-loss, relocated. A gap facing their strength is a positional
  mistake you can make and see. *(Provisional: to be revisited.)*

## Open

Taunt's exact positional rule.
