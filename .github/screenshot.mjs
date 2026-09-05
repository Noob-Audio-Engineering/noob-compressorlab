// Photograph the plug-in actually running, for the plug-in manager's banner.
//
// It drives the real page against the real engine --- the standalone serves
// exactly what the plug-in embeds, over exactly the same bridge --- so the
// picture is the plug-in rather than a mock-up of it. A banner that is a
// drawing of an interface goes stale the moment the interface changes; this
// one cannot, because it is taken from the build it ships beside.
//
//   node .github/screenshot.mjs <url> <out.png> [width] [height]

import { chromium } from 'playwright';

const [url, out, w = '1200', h = '700'] = process.argv.slice(2);
if (!url || !out) {
  console.error('usage: screenshot.mjs <url> <out.png> [width] [height]');
  process.exit(2);
}

const browser = await chromium.launch();
const page = await browser.newPage({
  viewport: { width: Number(w), height: Number(h) },
  deviceScaleFactor: 2,
});

const problems = [];
page.on('pageerror', (e) => problems.push(String(e)));
page.on('requestfailed', (r) => problems.push(`${r.url()} ${r.failure()?.errorText}`));

await page.goto(url, { waitUntil: 'networkidle', timeout: 60_000 });

// The panel only mounts once the manifest has arrived, so waiting for content
// is waiting for a real connection --- a screenshot of the loading state would
// be a picture of nothing and would still be a valid PNG.
await page.waitForFunction(
  () => {
    const app = document.querySelector('#app');
    return app && app.children.length > 0 && (app.innerText || '').trim().length > 40;
  },
  { timeout: 60_000 },
);

// A moment for the first stream frames, so meters and curves are drawn rather
// than empty.
await page.waitForTimeout(2500);

await page.screenshot({ path: out });
await browser.close();

if (problems.length) {
  console.error('the page reported problems while being photographed:');
  for (const p of problems.slice(0, 8)) console.error('  ' + p);
  process.exit(1);
}
console.log(`wrote ${out}`);
