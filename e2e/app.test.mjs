// End-to-end test of the real app (Rust backend, IPC and data files),
// driven through tauri-driver and WebKitWebDriver. Linux only.
//
// Needs: tauri-driver (cargo install tauri-driver --locked), WebKitWebDriver
// (apt install webkit2gtk-driver), a display (e.g. xvfb-run), and a build of
// the app with the frontend embedded:
//
//   npm run build
//   (cd src-tauri && cargo build --features tauri/custom-protocol)
//   xvfb-run -a npm run test:e2e
//
// The app runs with HOME and XDG_* pointing at a temporary directory, so the
// test starts from a fresh install and never touches your real study data.

import { spawn } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const APP = path.resolve(process.argv[2] ?? 'src-tauri/target/debug/tauri-app');
const DRIVER = 'http://127.0.0.1:4444';
const ELEMENT = 'element-6066-11e4-a52e-4f735466cecf';
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const sandbox = fs.mkdtempSync(path.join(os.tmpdir(), 'offline-nihongo-e2e-'));
const DATA_DIR = path.join(sandbox, 'data', 'OfflineNihongo');

function startDriver() {
  const driver = spawn('tauri-driver', [], {
    stdio: ['ignore', 'ignore', 'inherit'],
    env: {
      ...process.env,
      HOME: sandbox,
      XDG_DATA_HOME: path.join(sandbox, 'data'),
      XDG_CONFIG_HOME: path.join(sandbox, 'config'),
      XDG_CACHE_HOME: path.join(sandbox, 'cache'),
      // Software rendering and no sandboxing for headless/container runs
      WEBKIT_DISABLE_COMPOSITING_MODE: '1',
      WEBKIT_DISABLE_DMABUF_RENDERER: '1',
      WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS: '1'
    }
  });
  return driver;
}

async function waitForDriver() {
  for (let i = 0; i < 50; i++) {
    try {
      await fetch(DRIVER + '/status');
      return;
    } catch {
      await sleep(200);
    }
  }
  throw new Error('tauri-driver did not start');
}

async function call(method, url, body) {
  const res = await fetch(DRIVER + url, { method, headers: { 'content-type': 'application/json' }, body: body ? JSON.stringify(body) : undefined });
  const json = await res.json();
  if (json.value && json.value.error) throw new Error(`${method} ${url}: ${json.value.error}: ${json.value.message}`);
  return json.value;
}

let sid;
const exec = (script, ...args) => call('POST', `/session/${sid}/execute/sync`, { script, args });
async function find(css, timeout = 15000) {
  const end = Date.now() + timeout;
  while (true) {
    try { const el = await call('POST', `/session/${sid}/element`, { using: 'css selector', value: css }); return el[ELEMENT]; }
    catch (e) { if (Date.now() > end) throw new Error('not found: ' + css); await sleep(200); }
  }
}
const click = async (css) => call('POST', `/session/${sid}/element/${await find(css)}/click`, {});
const text = async (css) => (await call('GET', `/session/${sid}/element/${await find(css)}/text`)).replace(/\s+/g, ' ').trim();
const type = async (css, value) => call('POST', `/session/${sid}/element/${await find(css)}/value`, { text: value });
async function clickText(css, label) {
  await find(css);
  const ok = await exec(`const el = [...document.querySelectorAll(arguments[0])].find(e => e.textContent.trim().includes(arguments[1])); if (!el) return false; el.click(); return true;`, css, label);
  if (!ok) throw new Error(`no ${css} containing "${label}"`);
}
const readJson = (name) => JSON.parse(fs.readFileSync(path.join(DATA_DIR, name), 'utf8'));
function check(label, ok, detail = '') { console.log(`${ok ? 'PASS' : 'FAIL'}  ${label}${detail ? '  — ' + detail : ''}`); if (!ok) process.exitCode = 1; }

if (!fs.existsSync(APP)) {
  console.error(`App binary not found: ${APP}`);
  process.exit(1);
}
const driver = startDriver();

try {
  await waitForDriver();
  const session = await call('POST', '/session', { capabilities: { alwaysMatch: { 'tauri:options': { application: APP } } } });
  sid = session.sessionId;

  // Home page on a fresh install
  check('home page loads with the Today panel', (await text('.today')).includes('Start learning kanji'), await text('.today'));

  // Flashcards: add 5 cards through the real backend
  await clickText('.section-tile', 'Kanji Flashcards');
  await find('.add-button');
  await clickText('.add-button', '+5 Cards');
  await sleep(500);
  check('notice after adding cards', (await text('.notice')) === 'Added 5 new cards.', await text('.notice'));
  const cards = readJson('srs_cards.json');
  check('srs_cards.json written with 5 cards', cards.length === 5, cards.map((c) => c.character).join(''));

  // Study session: Again on the first card (it should come back), Good on the rest
  await clickText('.study-button', 'Learn New Cards');
  const seen = [];
  for (let i = 0; i < 12; i++) {
    if (await exec('return !!document.querySelector(".session-complete")')) break;
    await find('.study-card');
    const ch = await exec('return document.querySelector(".study-card .kanji-character")?.textContent.trim() || document.querySelector(".recall-placeholder") && "?"');
    seen.push(ch);
    await exec('document.querySelector(".show-answer-button")?.click(); return true;');
    await find('.grade-buttons');
    await click(i === 0 ? '.grade-button.fail' : '.grade-button.good');
    await sleep(400);
  }
  check('failed card returned later in the session', seen.length === 6 && seen.lastIndexOf(seen[0]) > 0, seen.join(' '));
  check('session summary', (await text('.session-complete')).toLowerCase().includes('reviews completed: 6'), await text('.session-summary'));
  const history = readJson('review_history.json');
  const today = Object.values(history)[0];
  check('review_history.json records today', today && today.reviews === 6 && today.correct === 5, JSON.stringify(history));

  // Kanji database: romaji search, real stroke diagram, add to deck
  await exec('document.querySelector(".home-button").click(); return true;');
  await clickText('.section-tile', 'Kanji Database');
  await find('.kanji-card');
  await type('.search-input', 'mizu');
  await sleep(800);
  check('romaji search finds 水 first', (await text('.kanji-card .kanji-character')) === '水', await text('.kanji-stats'));
  await click('.kanji-card');
  await find('.svg-wrapper svg');
  await sleep(500);
  check('real stroke order diagram for 水', (await text('.stroke-counter')) === 'Stroke 4 of 4', await text('.stroke-counter'));
  await click('.deck-button.add');
  await find('.deck-button.remove');
  check('水 added to srs_cards.json', readJson('srs_cards.json').some((c) => c.character === '水'));
  await exec('document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true })); window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" })); return true;');
  await sleep(300);

  // KanjiVG fallback diagram (bug #1): 喘 only has a KanjiVG file
  await exec('const i = document.querySelector(".search-input"); i.value = ""; i.dispatchEvent(new Event("input", { bubbles: true })); return true;');
  await type('.search-input', '喘');
  await sleep(800);
  await click('.kanji-card');
  await find('.svg-wrapper svg');
  await sleep(500);
  check('KanjiVG diagram shown for 喘', (await text('.stroke-counter')) === 'Stroke 12 of 12', await text('.stroke-counter'));
  await exec('window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" })); return true;');

  // Settings: export a backup
  await exec('document.querySelector(".home-button").click(); return true;');
  await clickText('.section-tile', 'Settings');
  await clickText('.setting-button', 'Export');
  await find('.data-message');
  const message = await text('.data-message');
  const exported = message.match(/Backup saved to (\S+\.json)/)?.[1];
  check('backup exported', !!exported && fs.existsSync(exported), message);
  if (exported) {
    const backup = JSON.parse(fs.readFileSync(exported, 'utf8'));
    check('backup contents', backup.format === 'offline-nihongo-backup' && backup.cards.length === 6, `${backup.cards.length} cards`);
  }

  // Kana chart: real kana SVG with grouped strokes
  await exec('document.querySelector(".home-button").click(); return true;');
  await clickText('.section-tile', 'Kana Chart');
  await clickText('.kana-button', 'あ');
  await find('.kana-svg-wrapper svg');
  await sleep(300);
  check('あ has 3 strokes (split pieces grouped)', (await text('.stroke-counter')).endsWith('of 3'), await text('.stroke-counter'));
  await exec('window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" })); return true;');

  // Kana game with the real kana data; save a best time
  await exec('document.querySelector(".home-button").click(); return true;');
  await clickText('.section-tile', 'Kana Speed Game');
  check('kana game preview (46 basic hiragana)', (await text('.preview-info')).includes('46 characters'), await text('.preview-info'));
  await click('.start-button');
  const romaji = { あ:'a',い:'i',う:'u',え:'e',お:'o',か:'ka',き:'ki',く:'ku',け:'ke',こ:'ko',さ:'sa',し:'shi',す:'su',せ:'se',そ:'so',た:'ta',ち:'chi',つ:'tsu',て:'te',と:'to',な:'na',に:'ni',ぬ:'nu',ね:'ne',の:'no',は:'ha',ひ:'hi',ふ:'fu',へ:'he',ほ:'ho',ま:'ma',み:'mi',む:'mu',め:'me',も:'mo',や:'ya',ゆ:'yu',よ:'yo',ら:'ra',り:'ri',る:'ru',れ:'re',ろ:'ro',わ:'wa',を:'wo',ん:'n' };
  for (let i = 0; i < 60; i++) {
    const kana = await exec('return document.querySelector(".current-kana")?.textContent.trim() || null');
    if (!kana) break;
    await type('.romaji-input', romaji[kana]);
  }
  await find('.results-screen');
  check('kana game completes with a new best time', (await text('.results-screen')).includes('New Best Time'), await text('.final-stats'));
  await sleep(500);
  const times = readJson('game_times.json');
  check('game_times.json written', typeof times.best_times['hiragana-seion-basic'] === 'number', JSON.stringify(times));
} catch (e) {
  console.log('ERROR', e.message);
  process.exitCode = 1;
} finally {
  if (sid) await call('DELETE', `/session/${sid}`).catch(() => {});
  driver.kill();
  fs.rmSync(sandbox, { recursive: true, force: true });
}
