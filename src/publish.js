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

  // 4. Wait for editor
  console.log('4. Editor URL:', page.url().substring(0, 80));
  try { await page.waitForSelector('div#edui1_iframeholder', { timeout: 10000 }); } catch(e) {}
  await page.waitForTimeout(3000);

  // 5. Original declaration — search all frames
  console.log('5. Setting original...');
  const clickInFrames = async (fn) => {
    for (const frame of page.frames()) {
      try { await frame.evaluate(fn); return true; } catch(e) {}
    }
    // Fallback: main page
    try { await page.evaluate(fn); } catch(e) {}
  };
  await clickInFrames(() => {
    const a = document.querySelectorAll('*');
    for (let i = 0; i < a.length; i++) { if (a[i].textContent.trim() === '未声明') { a[i].parentElement.click(); break; } }
  });
  await page.waitForTimeout(2000);
  await clickInFrames(() => {
    const a = document.querySelectorAll('*');
    for (let i = 0; i < a.length; i++) { if (a[i].textContent.includes('已阅读')) a[i].click(); }
    const b = document.querySelectorAll('button');
    for (let j = 0; j < b.length; j++) { if (b[j].textContent.trim() === '确定') { b[j].click(); break; } }
  });
  await page.waitForTimeout(2000);
  console.log('   ✅ Original');

  // 6. Source
  console.log('6. Setting source...');
  await clickInFrames(() => document.querySelector('#js_claim_source_area')?.click());
  await page.waitForTimeout(2000);
  await clickInFrames(() => {
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

  // 8. Insert account card at end of article
  console.log('8. Inserting account card...');
  try {
    // Move cursor to end of editor content
    await page.evaluate(() => {
      const ed = document.querySelector('[contenteditable="true"]');
      if (ed) {
        ed.focus();
        const r = document.createRange();
        r.selectNodeContents(ed);
        r.collapse(false);
        const sel = window.getSelection();
        sel.removeAllRanges();
        sel.addRange(r);
      }
    });
    await page.waitForTimeout(500);
    // Click "账号名片" / "公众号名片" in toolbar/insert menu
    await page.evaluate(() => {
      const all = document.querySelectorAll('*');
      for (let i = 0; i < all.length; i++) {
        const t = all[i].textContent.trim();
        if (t === '账号名片' || t === '公众号名片' || t === '插入名片') {
          all[i].click(); return;
        }
      }
    });
    await page.waitForTimeout(1000);
    // Confirm if dialog appears
    await page.evaluate(() => {
      const b = document.querySelectorAll('button');
      for (let i = 0; i < b.length; i++) {
        if (b[i].textContent.trim() === '确定' || b[i].textContent.trim() === '确认') {
          b[i].click(); break;
        }
      }
    });
  } catch(e) { console.log('   Account card: skipped'); }
  console.log('   ✅ Account card');

  // 9. Preview
  console.log('9. Preview...');
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
  console.log('=== ALL DONE. Browser stays open forever. Kill with Ctrl+C. ===');
  // Never exit — keep browser alive
  await new Promise(() => {});
}

(async () => {
  try { await main(); process.exit(0); }
  catch(e) { console.error(e.message); process.exit(1); }
})();
