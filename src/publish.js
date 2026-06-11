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

  // 3. Click the 2nd weui icon button (=编辑, pencil icon) in 1st card
  console.log('3. Clicking edit button...');
  const clicked = await page.evaluate(() => {
    const cards = document.querySelectorAll('.weui-desktop-card__action');
    if (!cards.length) return false;
    // Find all icon buttons in the first card's action area
    const btns = cards[0].querySelectorAll('a.weui-desktop-icon20.weui-desktop-icon-btn');
    // 2nd button = edit (1st=delete, 2nd=edit, 3rd=publish)
    if (btns.length >= 2) { btns[1].click(); return true; }
    return false;
  });
  console.log('   Clicked:', clicked);
  await page.waitForTimeout(3000);

  // 4. Wait for editor to fully load
  console.log('4. Waiting for editor...');
  await page.waitForTimeout(5000);
  try { await page.waitForSelector('div#edui1_iframeholder', { timeout: 15000 }); } catch(e) {}
  await page.waitForTimeout(3000);
  console.log('   Editor URL:', page.url().substring(0, 80));

  // 5. Original — Playwright native click (works cross-iframe)
  console.log('5. Setting original...');
  try { await page.click('text=未声明', { timeout: 5000 }); } catch(e) {}
  await page.waitForTimeout(2000);
  try { await page.click('text=已阅读并同意', { timeout: 3000 }); } catch(e) {}
  try { await page.click('button:has-text("确定")', { timeout: 3000 }); } catch(e) {}
  await page.waitForTimeout(2000);
  console.log('   ✅ Original');

  // 6. Reward (赞赏)
  console.log('6. Setting reward...');
  try { await page.click('text=赞赏', { timeout: 3000 }); } catch(e) {}
  await page.waitForTimeout(1000);
  try { await page.click('text=开启赞赏', { timeout: 2000 }); } catch(e) {}
  console.log('   ✅ Reward');

  // 7. Source
  console.log('7. Setting source...');
  try { await page.click('#js_claim_source_area', { timeout: 3000 }); } catch(e) {}
  await page.waitForTimeout(2000);
  try { await page.click('text=个人观点，仅供参考', { timeout: 3000 }); } catch(e) {}
  try { await page.click('button:has-text("确认")', { timeout: 3000 }); } catch(e) {}
  await page.waitForTimeout(2000);
  console.log('   ✅ Source');

  // 8. Collection (合集)
  console.log('8. Setting collection...');
  try { await page.click('text=合集', { timeout: 3000 }); } catch(e) {}
  await page.waitForTimeout(1000);
  console.log('   ✅ Collection');

  // 9. Save
  console.log('9. Saving...');
  try { await page.click('button:has-text("保存为草稿")', { timeout: 5000 }); } catch(e) {}
  await page.waitForTimeout(3000);
  console.log('   ✅ Save');

  // 10. Insert account card at end of article
  console.log('10. Inserting account card...');
  try {
    // Step A: Move cursor to end in the editor iframe
    const frameEl = await page.waitForSelector('iframe[src*="appmsg_edit"]', { timeout: 10000 });
    if (frameEl) {
      const editorFrame = await frameEl.contentFrame();
      if (editorFrame) {
        await editorFrame.evaluate(() => {
          const ed = document.querySelector('[contenteditable="true"]') || document.body;
          if (ed) {
            ed.focus();
            const r = document.createRange();
            r.selectNodeContents(ed);
            r.collapse(false);
            window.getSelection().removeAllRanges();
            window.getSelection().addRange(r);
          }
        });
      }
    }
    await page.waitForTimeout(500);

    // Step B: Click "..." (three dots) on top toolbar
    console.log('    点击导航栏 [...] ...');
    // Try multiple known selectors for the three-dots button
    try { await page.click('.js_editor_insert_more', { timeout: 3000 }); } catch(e) {}
    try { await page.click('[class*="more"]', { timeout: 2000 }); } catch(e) {}
    try { await page.click('i.weui-desktop-icon-more', { timeout: 2000 }); } catch(e) {}
    // Fallback: find by SVG three-dots pattern
    try {
      await page.evaluate(() => {
        const all = document.querySelectorAll('*');
        for (let i = 0; i < all.length; i++) {
          const t = all[i].textContent.trim();
          if (t === '…' || t === '...' || t === '更多') { all[i].click(); return; }
        }
      });
    } catch(e) {}
    await page.waitForTimeout(1000);

    // Step C: Click "账号名片" in dropdown
    console.log('    点击账号名片...');
    try { await page.click('text=账号名片', { timeout: 5000 }); } catch(e) {}
    await page.waitForTimeout(1500);

    // Step D: Confirm dialog
    try { await page.click('button:has-text("确定")', { timeout: 3000 }); } catch(e) {}
    try { await page.click('button:has-text("确认")', { timeout: 3000 }); } catch(e) {}
    console.log('   ✅ Account card');
  } catch(e) { console.log('   Account card:', e.message); }

  // 11. Preview
  console.log('11. Preview...');
  try { await page.click('button:has-text("预览")', { timeout: 5000 }); } catch(e) {}
  await page.waitForTimeout(2000);
  try { await page.click('text=公众号列表预览', { timeout: 3000 }); } catch(e) {}
  try { await page.click('button:has-text("确定")', { timeout: 3000 }); } catch(e) {}
  console.log('   ✅ Preview');
  console.log('=== ALL DONE. Browser stays open forever. Kill with Ctrl+C. ===');
  // Never exit — keep browser alive
  await new Promise(() => {});
}

(async () => {
  try { await main(); process.exit(0); }
  catch(e) { console.error(e.message); process.exit(1); }
})();
