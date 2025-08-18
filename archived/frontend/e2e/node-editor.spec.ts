import { test, expect } from '@playwright/test';

const pageUrl = '/e2e/node-editor.html';

// Provide default blocks and clear storage before each test
const initScript = () => {
  localStorage.clear();
  window.getInitialBlocks = () => {
    const pos = JSON.parse(localStorage.getItem('block-pos') || '{"x":100,"y":100}');
    return [
      {
        visual_id: '1',
        kind: 'function',
        x: pos.x,
        y: pos.y,
        translations: { en: 'Main', ru: '\u0413\u043b\u0430\u0432\u043d\u0430\u044f', es: 'Principal' },
        links: []
      },
      {
        visual_id: '2',
        kind: 'function',
        x: pos.x + 150,
        y: pos.y,
        translations: { en: 'Second', ru: '\u0412\u0442\u043e\u0440\u043e\u0439', es: 'Segundo' },
        links: []
      }
    ];
  };
};

test.beforeEach(async ({ page }) => {
  await page.addInitScript(initScript);
});

test('block parsing and locale switch', async ({ page }) => {
  await page.goto(pageUrl);
  let label = await page.evaluate(() => window.vc.blocks[0].label);
  expect(label).toBe('Main');
  await page.evaluate(() => changeLocale('ru'));
  label = await page.evaluate(() => window.vc.blocks[0].label);
  expect(label).toBe('\u0413\u043b\u0430\u0432\u043d\u0430\u044f');
  await page.evaluate(() => changeLocale('es'));
  label = await page.evaluate(() => window.vc.blocks[0].label);
  expect(label).toBe('Principal');
});

test('block position persists after reload', async ({ page }) => {
  await page.goto(pageUrl);
  const canvas = page.locator('#canvas');
  const box = await canvas.boundingBox();
  const start = await page.evaluate(() => ({ x: window.vc.blocksData[0].x, y: window.vc.blocksData[0].y }));
  await page.mouse.move(box.x + start.x + 10, box.y + start.y + 10);
  await page.mouse.down();
  await page.mouse.move(box.x + start.x + 80, box.y + start.y + 80);
  await page.mouse.up();
  await page.evaluate(() => {
    const b = window.vc.blocksData[0];
    localStorage.setItem('block-pos', JSON.stringify({ x: b.x, y: b.y }));
  });
  await page.reload();
  await page.goto(pageUrl);
  const pos = await page.evaluate(() => ({ x: window.vc.blocksData[0].x, y: window.vc.blocksData[0].y }));
  expect(pos.x).toBeGreaterThan(start.x);
});

test('metadata editing updates translation and stores note', async ({ page }) => {
  await page.goto(pageUrl);
  await page.evaluate(() => {
    updateBlock('1', { translations: { en: 'Changed' } });
    updateBlock('1', { data: { note: 'AI note' } });
  });
  const label = await page.evaluate(() => window.vc.blocks[0].label);
  expect(label).toBe('Changed');
  const note = await page.evaluate(() => window.vc.blocksData[0].data.note);
  expect(note).toBe('AI note');
});

test('connections update when blocks move', async ({ page }) => {
  await page.goto(pageUrl);
  await page.evaluate(() => connectBlocks('1', '2'));
  const canvas = page.locator('#canvas');
  const box = await canvas.boundingBox();
  const start = await page.evaluate(() => ({ x: window.vc.blocksData[1].x, y: window.vc.blocksData[1].y }));
  await page.mouse.move(box.x + start.x + 10, box.y + start.y + 10);
  await page.mouse.down();
  await page.mouse.move(box.x + start.x + 70, box.y + start.y + 40);
  await page.mouse.up();
  const targetX = await page.evaluate(() => window.vc.connections[0][1].x);
  expect(targetX).toBeGreaterThan(start.x);
});

test('export removes @VISUAL_META comments', async ({ page }) => {
  await page.goto(pageUrl);
  const cleaned = await page.evaluate(() => exportClean("// @VISUAL_META {\"id\":\"1\"}\nconsole.log('hi');"));
  expect(cleaned).not.toContain('@VISUAL_META');
});

test('import parses @VISUAL_META comments', async ({ page }) => {
  await page.goto(pageUrl);
  await page.evaluate(() => importContent("// @VISUAL_META {\"id\":\"a\",\"x\":123,\"y\":77,\"translations\":{\"en\":\"T\"}}\ncode"));
  const data = await page.evaluate(() => ({ x: window.vc.blocksData[0].x, y: window.vc.blocksData[0].y, label: window.vc.blocks[0].label }));
  expect(data.x).toBe(123);
  expect(data.y).toBe(77);
  expect(data.label).toBe('T');
});

test('websocket synchronization via storage events', async ({ browser }) => {
  const context1 = await browser.newContext();
  const page1 = await context1.newPage();
  await page1.addInitScript(initScript);
  await page1.goto(pageUrl);

  const context2 = await browser.newContext();
  const page2 = await context2.newPage();
  await page2.addInitScript(initScript);
  await page2.goto(pageUrl);

  const box = await page1.locator('#canvas').boundingBox();
  const start = await page1.evaluate(() => ({ x: window.vc.blocksData[0].x, y: window.vc.blocksData[0].y }));
  await page1.mouse.move(box.x + start.x + 5, box.y + start.y + 5);
  await page1.mouse.down();
  await page1.mouse.move(box.x + start.x + 80, box.y + start.y + 80);
  await page1.mouse.up();
  await page1.evaluate(() => {
    const b = window.vc.blocksData[0];
    localStorage.setItem('block-pos', JSON.stringify({ x: b.x, y: b.y }));
  });

  await page2.waitForFunction(() => window.vc.blocksData[0].x > 100);
  const pos2 = await page2.evaluate(() => ({ x: window.vc.blocksData[0].x, y: window.vc.blocksData[0].y }));
  expect(pos2.x).toBeGreaterThan(100);

  await context1.close();
  await context2.close();
});
