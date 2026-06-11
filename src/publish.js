const pw = require('playwright-core');
const path = require('path');

const PROFILE = path.join(
  process.env.HOME, 'Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain/.moonpub/chrome-profile'
);

async function main() {
  const mode = process.argv[2] || 'configure';
  const browser = await pw.chromium.launchPersistentContext(PROFILE, {
    headless: false, channel: 'chrome',
  });
  const page = browser.pages()[0] || await browser.newPage();

  if (mode === 'login') {
    await page.goto('https://mp.weixin.qq.com');
    console.log('Scan QR, then close browser.');
    await new Promise(r => setTimeout(r, 120000));
    await browser.close();
    return;
  }

  // ── configure ──
  // 1. Login
  console.log('1. Checking login...');
  await page.goto('https://mp.weixin.qq.com');
  await page.waitForTimeout(4000);
  let url = page.url();
  if (!url.includes('cgi-bin/home')) {
    console.log('Please scan QR code...');
    await page.waitForFunction(() => location.href.includes('cgi-bin/home'), { timeout: 120000 });
    url = page.url();
  }
  console.log('   Logged in');
  const token = url.split('token=')[1]?.split('&')[0] || '';

  // 2. Drafts list
  console.log('2. Opening drafts list...');
  await page.goto(
    `https://mp.weixin.qq.com/cgi-bin/appmsg?begin=0&count=10&type=77&action=list_card&token=${token}&lang=zh_CN`
  );
  await page.waitForTimeout(5000);

  // 3. Hover first card → click edit
  console.log('3. Hovering first card...');
  const enterEditor = await page.evaluate(() => {
    // Find first card by "更新于" text
    const all = document.querySelectorAll('*');
    let card = null;
    for (let i = 0; i < all.length; i++) {
      if (all[i].children.length === 0 && all[i].textContent.includes('更新于')) {
        card = all[i];
        while (card && card.tagName !== 'A') card = card.parentElement;
        break;
      }
    }
    if (!card) return { error: 'card not found' };

    // Hover to reveal edit button
    card.dispatchEvent(new MouseEvent('mouseover', { bubbles: true }));
    // Click the card itself — may navigate to editor
    card.click();
    return { ok: true };
  });
  if (enterEditor.error) { console.log('   Error:', enterEditor.error); }
  await page.waitForTimeout(3000);

  // 4. Wait for editor
  console.log('4. Editor URL:', page.url().substring(0, 80));
  try { await page.waitForSelector('div#edui1_iframeholder', { timeout: 10000 }); } catch(e) {}
  await page.waitForTimeout(3000);

  // 5. Original declaration
  console.log('5. Setting original...');
  await page.evaluate(() => {
    const a = document.querySelectorAll('*');
    for (let i = 0; i < a.length; i++) { if (a[i].textContent.trim() === '未声明') { a[i].parentElement.click(); break; } }
  });
  await page.waitForTimeout(2000);
  await page.evaluate(() => {
    const a = document.querySelectorAll('*');
    for (let i = 0; i < a.length; i++) { if (a[i].textContent.includes('已阅读')) a[i].click(); }
    const b = document.querySelectorAll('button');
    for (let j = 0; j < b.length; j++) { if (b[j].textContent.trim() === '确定') { b[j].click(); break; } }
  });
  await page.waitForTimeout(2000);
  console.log('   ✅ Original');

  // 6. Source
  console.log('6. Setting source...');
  await page.evaluate(() => document.querySelector('#js_claim_source_area')?.click());
  await page.waitForTimeout(2000);
  await page.evaluate(() => {
    const a = document.querySelectorAll('*');
    for (let i = 0; i < a.length; i++) { if (a[i].textContent.trim() === '个人观点，仅供参考') { a[i].click(); break; } }
    const b = document.querySelectorAll('button');
    for (let j = 0; j < b.length; j++) { if (b[j].textContent.trim() === '确认') { b[j].click(); break; } }
  });
  await page.waitForTimeout(2000);
  console.log('   ✅ Source');

  // 7. Save
  console.log('7. Saving...');
  await page.evaluate(() => {
    const b = document.querySelectorAll('button');
    for (let i = 0; i < b.length; i++) { if (b[i].textContent.trim() === '保存为草稿') { b[i].click(); break; } }
  });
  await page.waitForTimeout(3000);
  console.log('   ✅ Save');

  // 8. Preview
  console.log('8. Preview...');
  await page.evaluate(() => {
    const b = document.querySelectorAll('button');
    for (let i = 0; i < b.length; i++) { if (b[i].textContent.trim() === '预览') { b[i].click(); break; } }
  });
  await page.waitForTimeout(2000);
  await page.evaluate(() => {
    const a = document.querySelectorAll('label');
    for (let i = 0; i < a.length; i++) { if (a[i].textContent.includes('公众号列表预览')) a[i].click(); }
    const b = document.querySelectorAll('button');
    for (let j = 0; j < b.length; j++) { if (b[j].textContent.trim() === '确定') { b[j].click(); break; } }
  });
  console.log('   ✅ Preview');
  console.log('=== DONE. Browser stays open. Press Enter in terminal to close. ===');
  await new Promise(r => process.stdin.once('data', r));
  await browser.close();
}

(async () => {
  try { await main(); process.exit(0); }
  catch(e) { console.error(e.message); process.exit(1); }
})();
