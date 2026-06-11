// MoonPub browser automation — Playwright (proven, reliable).
const pw = require('playwright-core');
const fs = require('fs');
const path = require('path');

const PROFILE = path.join(
  process.env.HOME,
  'Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain/.moonpub/chrome-profile'
);

async function main() {
  const mode = process.argv[2] || 'configure';

  const browser = await pw.chromium.launchPersistentContext(PROFILE, {
    headless: false,
    channel: 'chrome',
  });
  const page = browser.pages()[0] || await browser.newPage();

  if (mode === 'login') {
    await page.goto('https://mp.weixin.qq.com');
    console.log('Scan QR code with WeChat, then close the browser.');
    await new Promise(r => setTimeout(r, 120000));
    await browser.close();
    return;
  }

  // ── configure mode ──
  // Step 1: Home page → extract token
  await page.goto('https://mp.weixin.qq.com');
  await page.waitForTimeout(3000);
  let url = page.url();
  if (!url.includes('cgi-bin/home')) {
    console.log('Please scan QR code...');
    await page.waitForFunction(
      () => location.href.includes('cgi-bin/home'),
      { timeout: 120000 }
    );
    url = page.url();
  }
  console.log('1. Logged in');
  const token = url.split('token=')[1]?.split('&')[0] || '';

  // Step 2: Drafts list page
  await page.goto(
    `https://mp.weixin.qq.com/cgi-bin/appmsg?begin=0&count=10&type=77&action=list_card&token=${token}&lang=zh_CN`
  );
  await page.waitForTimeout(5000);
  console.log('2. Drafts list loaded');

  // Step 3: Hover first card → find edit button → click
  const clicked = await page.evaluate(() => {
    const cards = document.querySelectorAll('.appmsg_card_wrp, .appmsg_card, [class*="card"]');
    if (!cards.length) return false;
    const card = cards[0];
    card.scrollIntoView({ block: 'center' });
    card.dispatchEvent(new MouseEvent('mouseover', { bubbles: true }));
    const btns = card.querySelectorAll('a');
    const found = [];
    for (let i = 0; i < btns.length; i++) {
      if (btns[i].title === '编辑' || btns[i].textContent.trim() === '编辑') {
        found.push(btns[i]);
      }
    }
    if (found.length >= 2) { found[1].click(); return true; }
    if (found.length === 1) { found[0].click(); return true; }
    // Fallback: extract appmsgid and redirect
    const href = card.getAttribute('href') || card.querySelector('a')?.getAttribute('href') || '';
    if (href.includes('appmsgid=')) {
      window.location.href = href;
      return true;
    }
    card.click();
    return true;
  });
  console.log('3. Card clicked:', clicked);

  // Step 4: Wait for editor
  await page.waitForTimeout(5000);
  console.log('4. Editor URL:', page.url().substring(0, 80));

  // Step 5: Original declaration
  await page.evaluate(() => {
    const all = document.querySelectorAll('*');
    for (let i = 0; i < all.length; i++) {
      if (all[i].textContent.trim() === '未声明') { all[i].parentElement.click(); break; }
    }
  });
  await page.waitForTimeout(2000);
  await page.evaluate(() => {
    const all = document.querySelectorAll('*');
    for (let i = 0; i < all.length; i++) { if (all[i].textContent.includes('已阅读')) all[i].click(); }
    const btns = document.querySelectorAll('button');
    for (let j = 0; j < btns.length; j++) { if (btns[j].textContent.trim() === '确定') { btns[j].click(); break; } }
  });
  await page.waitForTimeout(2000);
  console.log('5. Original: done');

  // Step 6: Source
  await page.evaluate(() => document.querySelector('#js_claim_source_area')?.click());
  await page.waitForTimeout(2000);
  await page.evaluate(() => {
    const all = document.querySelectorAll('*');
    for (let i = 0; i < all.length; i++) { if (all[i].textContent.trim() === '个人观点，仅供参考') { all[i].click(); break; } }
    const btns = document.querySelectorAll('button');
    for (let j = 0; j < btns.length; j++) { if (btns[j].textContent.trim() === '确认') { btns[j].click(); break; } }
  });
  await page.waitForTimeout(2000);
  console.log('6. Source: done');

  // Step 7: Save
  await page.evaluate(() => {
    const btns = document.querySelectorAll('button');
    for (let i = 0; i < btns.length; i++) {
      if (btns[i].textContent.trim() === '保存为草稿') { btns[i].click(); break; }
    }
  });
  await page.waitForTimeout(3000);
  console.log('7. Save: done');

  // Step 8: Preview
  await page.evaluate(() => {
    const btns = document.querySelectorAll('button');
    for (let i = 0; i < btns.length; i++) {
      if (btns[i].textContent.trim() === '预览') { btns[i].click(); break; }
    }
  });
  await page.waitForTimeout(2000);
  await page.evaluate(() => {
    const all = document.querySelectorAll('label');
    for (let i = 0; i < all.length; i++) { if (all[i].textContent.includes('公众号列表预览')) all[i].click(); }
    const btns = document.querySelectorAll('button');
    for (let j = 0; j < btns.length; j++) { if (btns[j].textContent.trim() === '确定') { btns[j].click(); break; } }
  });
  console.log('8. Preview: done');
  console.log('=== DONE. Press Enter in terminal to close. ===');
  await new Promise(r => process.stdin.once('data', r));
  await browser.close();
}

(async () => {
  try {
    await main();
    console.log('ALL DONE');
    process.exit(0);
  } catch (e) {
    console.error('FATAL:', e.message);
    process.exit(1);
  }
})();
