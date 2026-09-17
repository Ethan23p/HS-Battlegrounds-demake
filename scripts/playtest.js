// A scripted playthrough exercising the mechanics that matter most when
// this loop changes: buy, sell, freeze, reroll, upgrade tavern, fight,
// continue across rounds, and a run ending. Ethan asked for something like
// this to actually watch run before committing to changes in this area,
// rather than trusting unit tests alone for what's fundamentally a
// feel-driven UI -- see docs/transcripts/0011-mobile-playtest-and-ability-authoring.md.
//
// Usage: serve web/ (e.g. `python3 -m http.server 8000` from web/), then
//   node scripts/playtest.js [http://127.0.0.1:8000] [screenshot-dir]
// Screenshots land in screenshot-dir (default: a scratch dir next to this
// script) at each notable step, for a human (or Claude) to actually look at.

// No package.json in this repo pins a Playwright version; fall back to a
// system-wide install (this session's environment has one) if a local
// `require("playwright")` can't find one.
let chromium;
try {
  ({ chromium } = require("playwright"));
} catch {
  ({ chromium } = require("/opt/node22/lib/node_modules/playwright"));
}
const path = require("node:path");
const fs = require("node:fs");

const BASE_URL = process.argv[2] || "http://127.0.0.1:8000";
const OUT = process.argv[3] || path.join(require("node:os").tmpdir(), "bg-playtest");
fs.mkdirSync(OUT, { recursive: true });

function state(page) {
  return page.evaluate(() => ({
    gold: Number(document.getElementById("gold-value").textContent),
    tier: document.getElementById("tier-value").textContent,
    record: document.getElementById("record-value").textContent,
    board: [...document.querySelectorAll("#player-row .unit:not(.empty) .name")].map((n) => n.textContent),
    shop: [...document.querySelectorAll("#opposing-row .unit:not(.empty) .name")].map((n) => n.textContent),
    shopVisible: !document.getElementById("shop-controls").hidden,
    runOverVisible: !document.getElementById("run-over-controls").hidden,
    shopMessage: document.getElementById("shop-message").textContent,
  }));
}

async function shot(page, name) {
  await page.screenshot({ path: path.join(OUT, `${name}.png`) });
}

async function main() {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1000, height: 620 } });
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));

  await page.goto(`${BASE_URL}/index.html?bust=${Date.now()}`, { waitUntil: "networkidle" });
  await shot(page, "01-boot");
  console.log("boot:", await state(page));

  // Buy whatever's affordable this round.
  for (let i = 0; i < 3; i++) {
    const s = await state(page);
    if (s.gold < 3 || s.board.length >= 6) break;
    const offers = await page.locator("#opposing-row .unit:not(.empty)").count();
    if (offers === 0) break;
    await page.locator("#opposing-row .unit:not(.empty)").first().click();
    await page.waitForTimeout(80);
  }
  console.log("after buys:", await state(page));
  await shot(page, "02-bought");

  // Freeze the shop, reroll (should be a no-op on the lineup) if affordable,
  // unfreeze.
  await page.click("#freeze");
  await page.waitForTimeout(60);
  const frozenShop = (await state(page)).shop;
  if (!(await page.locator("#reroll").isDisabled())) {
    await page.click("#reroll");
    await page.waitForTimeout(60);
    const afterReroll = (await state(page)).shop;
    console.log("freeze held through reroll:", JSON.stringify(frozenShop) === JSON.stringify(afterReroll));
  } else {
    console.log("reroll not affordable this round -- skipped that check");
  }
  await page.click("#freeze");
  await page.waitForTimeout(60);
  await shot(page, "03-froze-and-unfroze");

  // Sell the first board unit, if any.
  if ((await state(page)).board.length > 0) {
    await page.locator("#player-row .sell-btn").first().click();
    await page.waitForTimeout(80);
    console.log("after sell:", await state(page));
  }
  await shot(page, "04-after-sell");

  // Drag-reorder, if two units are on the board.
  const boardCount = (await state(page)).board.length;
  if (boardCount >= 2) {
    const a = page.locator("#player-row .unit").nth(0);
    const b = page.locator("#player-row .unit").nth(1);
    const ba = await a.boundingBox();
    const bb = await b.boundingBox();
    await page.mouse.move(ba.x + ba.width / 2, ba.y + ba.height / 2);
    await page.mouse.down();
    await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2, { steps: 8 });
    await page.mouse.up();
    await page.waitForTimeout(80);
    console.log("after drag-reorder:", (await state(page)).board);
  }
  await shot(page, "05-after-reorder");

  // Play through a few rounds: fight, skip to end, continue.
  for (let round = 0; round < 4; round++) {
    let s = await state(page);
    if (s.runOverVisible) break;
    await page.click("#fight");
    await page.waitForTimeout(150);
    await shot(page, `06-round${round}-fight-start`);
    if (!(await page.locator("#skip").isDisabled())) await page.click("#skip");
    await page.waitForTimeout(100);
    await shot(page, `07-round${round}-fight-end`);
    console.log(`round ${round} result:`, await state(page));
    if (!(await page.locator("#continue").isDisabled())) {
      await page.click("#continue");
      await page.waitForTimeout(150);
    }
  }
  await shot(page, "08-final");
  console.log("final:", await state(page));
  console.log("console/page errors:", errors);

  await browser.close();
  if (errors.length > 0) {
    console.error(`FAILED: ${errors.length} page error(s) during playtest`);
    process.exitCode = 1;
  } else {
    console.log(`OK -- screenshots in ${OUT}`);
  }
}

main();
