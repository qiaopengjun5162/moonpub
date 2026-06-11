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

  // 4. Wait for editor + create frameLocator for the controls iframe
  console.log('4. Waiting for editor...');
  await page.waitForSelector('iframe[src*="appmsg_edit"]', { timeout: 20000 }).catch(() => {});
  await page.waitForTimeout(3000);
  const editorFrame = page.frameLocator('iframe[src*="appmsg_edit"]');

  // 5. Original — via frameLocator (cross-iframe)
  console.log('5. Setting original...');
  try { await editorFrame.locator('text=未声明').click({ timeout: 5000 }); } catch(e) {}
  await page.waitForTimeout(2000);
  try { await editorFrame.locator('text=已阅读并同意').click({ timeout: 3000 }); } catch(e) {}
  try { await editorFrame.locator('button:has-text("确定")').click({ timeout: 3000 }); } catch(e) {}
  await page.waitForTimeout(2000);
  console.log('   ✅ Original');

  // 6. Reward
  console.log('6. Setting reward...');
  try { await editorFrame.locator('text=赞赏').click({ timeout: 3000 }); } catch(e) {}
  await page.waitForTimeout(1000);
  try { await editorFrame.locator('text=开启赞赏').click({ timeout: 2000 }); } catch(e) {}
  console.log('   ✅ Reward');

  // 7. Source
  console.log('7. Setting source...');
  try { await editorFrame.locator('#js_claim_source_area').click({ timeout: 3000 }); } catch(e) {}
  await page.waitForTimeout(2000);
  try { await editorFrame.locator('text=个人观点，仅供参考').click({ timeout: 3000 }); } catch(e) {}
  try { await editorFrame.locator('button:has-text("确认")').click({ timeout: 3000 }); } catch(e) {}
  console.log('   ✅ Source');

  // 8. Collection
  console.log('8. Setting collection...');
  try { await editorFrame.locator('text=合集').click({ timeout: 3000 }); } catch(e) {}
  await page.waitForTimeout(1000);
  try { await editorFrame.locator('[class*=dropdown] li, [class*=menu] li').first().click({ timeout: 2000 }); } catch(e) {}
  console.log('   ✅ Collection');

  // 9. Save
  console.log('9. Saving...');
  try { await editorFrame.locator('button:has-text("保存为草稿")').click({ timeout: 5000 }); } catch(e) {}
  await page.waitForTimeout(3000);
  console.log('   ✅ Save');

  // 10. Account card — toolbar "..." on top level, card name in dropdown
  console.log('10. Inserting account card...');
  try {
    // Focus end of editor
    await editorFrame.locator('[contenteditable="true"]').focus();
    await page.waitForTimeout(500);
    // Click toolbar "..." — try both top-level and frame
    console.log('    Clicking [...] menu...');
    try { await page.click('.js_editor_insert_more', { timeout: 3000 }); } catch(e) {}
    try { await editorFrame.locator('.js_editor_insert_more, i[class*="more"]').click({ timeout: 2000 }); } catch(e) {}
    await page.waitForTimeout(800);
    // Click "账号名片" in dropdown
    console.log('    Clicking 账号名片...');
    try { await page.click('text=账号名片', { timeout: 3000 }); } catch(e) {}
    try { await editorFrame.locator('text=账号名片').click({ timeout: 2000 }); } catch(e) {}
    await page.waitForTimeout(1500);
    // Confirm
    try { await page.click('button:has-text("确定")', { timeout: 3000 }); } catch(e) {}
    console.log('   ✅ Account card');
  } catch(e) { console.log('   Account card skipped:', e.message); }

  // 11. Preview
  console.log('11. Preview...');
  try { await editorFrame.locator('button:has-text("预览")').click({ timeout: 5000 }); } catch(e) {}
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
