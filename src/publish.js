const pw = require('playwright-core');
const path = require('path');
const PROFILE = path.join(process.env.HOME, 'Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain/.moonpub/chrome-profile');

async function main() {
  const mode = process.argv[2] || 'configure';
  const browser = await pw.chromium.launchPersistentContext(PROFILE, { headless: false, channel: 'chrome' });
  const pages = browser.pages();
  for (let i = 1; i < pages.length; i++) await pages[i].close();
  const page = pages[0] || await browser.newPage();

  if (mode === 'login') {
    await page.goto('https://mp.weixin.qq.com');
    console.log('Scan QR, then close.');
    await new Promise(r => setTimeout(r, 120000));
    await browser.close(); return;
  }

  // 1. Login
  console.log('1. Login...');
  await page.goto('https://mp.weixin.qq.com');
  await page.waitForTimeout(4000);
  let url = page.url();
  if (!url.includes('cgi-bin/home')) {
    console.log('Please scan QR code...');
    await page.waitForFunction(() => location.href.includes('cgi-bin/home'), { timeout: 120000 }).catch(() => {});
    url = page.url();
  }
  console.log('   Logged in');
  const token = url.split('token=')[1]?.split('&')[0] || '';

  // 2. Drafts list
  console.log('2. Drafts list...');
  await page.goto(`https://mp.weixin.qq.com/cgi-bin/appmsg?begin=0&count=10&type=77&action=list_card&token=${token}&lang=zh_CN`);
  await page.waitForTimeout(5000);

  // 3. Click 2nd icon button (=edit) in first card
  console.log('3. Click edit...');
  const clicked = await page.evaluate(() => {
    const cards = document.querySelectorAll('.weui-desktop-card__action');
    if (!cards.length) return false;
    const btns = cards[0].querySelectorAll('a.weui-desktop-icon20.weui-desktop-icon-btn');
    if (btns.length >= 2) { btns[1].click(); return true; }
    return false;
  });
  console.log('   Clicked:', clicked);
  await page.waitForTimeout(4000);

  // 4. Editor
  console.log('4. Editor...');
  await page.waitForSelector('iframe[src*="appmsg_edit"]', { timeout: 20000 }).catch(() => {});
  await page.waitForTimeout(3000);
  const f = page.frameLocator('iframe[src*="appmsg_edit"]');
  console.log('   URL:', page.url().substring(0,80));

  // 5. Original — via frameLocator
  console.log('5. Original...');
  try { await f.locator('text=未声明').click({ timeout: 5000 }); } catch(e) {}
  await page.waitForTimeout(2000);
  try { await f.locator('text=已阅读并同意').click({ timeout: 3000 }); } catch(e) {}
  try { await f.locator('button:has-text("确定")').click({ timeout: 3000 }); } catch(e) {}
  console.log('   ✅ Original');

  // 6. Reward
  console.log('6. Reward...');
  try { await f.locator('text=赞赏').click({ timeout: 3000 }); } catch(e) {}
  await page.waitForTimeout(1000);
  try { await f.locator('text=开启赞赏').click({ timeout: 2000 }); } catch(e) {}
  console.log('   ✅ Reward');

  // 7. Source
  console.log('7. Source...');
  try { await f.locator('#js_claim_source_area').click({ timeout: 3000 }); } catch(e) {}
  await page.waitForTimeout(2000);
  try { await f.locator('text=个人观点，仅供参考').click({ timeout: 3000 }); } catch(e) {}
  try { await f.locator('button:has-text("确认")').click({ timeout: 3000 }); } catch(e) {}
  console.log('   ✅ Source');

  // 8. Collection
  console.log('8. Collection...');
  try { await f.locator('text=合集').click({ timeout: 3000 }); } catch(e) {}
  await page.waitForTimeout(1000);
  console.log('   ✅ Collection');

  // 9. Account card — click #editor_showmore → #js_editor_insertProfile
  console.log('9. Account card...');
  try {
    await f.locator('[contenteditable="true"]').focus();
    await page.waitForTimeout(500);
    await page.click('#editor_showmore', { timeout: 3000 });
    await page.waitForTimeout(800);
    await page.click('#js_editor_insertProfile', { timeout: 3000 });
    await page.waitForTimeout(1500);
    await page.click('button:has-text("确定")', { timeout: 3000 });
    console.log('   ✅ Account card');
  } catch(e) { console.log('   Skipped:', e.message); }

  // 10. Save
  console.log('10. Save...');
  try { await f.locator('button:has-text("保存为草稿")').click({ timeout: 5000 }); } catch(e) {}
  await page.waitForTimeout(3000);
  console.log('   ✅ Save');

  // 11. Preview
  console.log('11. Preview...');
  try { await f.locator('button:has-text("预览")').click({ timeout: 5000 }); } catch(e) {}
  await page.waitForTimeout(2000);
  try { await page.click('text=公众号列表预览', { timeout: 3000 }); } catch(e) {}
  try { await page.click('button:has-text("确定")', { timeout: 3000 }); } catch(e) {}
  console.log('   ✅ Preview');

  console.log('=== ALL DONE. Browser stays open. ===');
  await new Promise(() => {});
}

(async () => { try { await main(); } catch(e) { console.error(e.message); process.exit(1); } })();
