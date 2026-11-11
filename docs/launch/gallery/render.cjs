// Install Playwright in your tooling environment; this does not use a signed-in profile.
// Optional: set PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH to a local Chromium executable.
const { chromium } = require('playwright');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

async function main() {
  const browser = await chromium.launch({
    headless: true,
    ...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH
      ? { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH } : {}),
  });
  try {
    const page = await browser.newPage({ viewport: { width: 1270, height: 760 }, deviceScaleFactor: 1 });
    await page.goto(pathToFileURL(path.join(__dirname, 'gallery.html')).href);
    await page.evaluate(() => document.fonts.ready);
    for (const id of ['01-your-devices', '02-the-files', '03-the-model', '04-get-started']) {
      await page.locator(`[id="${id}"]`).screenshot({ path: path.join(__dirname, `${id}.png`) });
    }
    await page.setViewportSize({ width: 240, height: 240 });
    await page.goto(pathToFileURL(path.join(__dirname, 'logo.svg')).href);
    await page.screenshot({ path: path.join(__dirname, 'logo.png'), omitBackground: true });
    console.log('Rendered logo and four gallery PNGs.');
  } finally { await browser.close(); }
}
main().catch(error => { console.error(error); process.exitCode = 1; });
