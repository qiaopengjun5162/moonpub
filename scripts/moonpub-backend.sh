#!/usr/bin/env bash
# moonpub-backend.sh — 微信后台自动设置
# 用法: WECHAT_SECRET=xxx ./moonpub-backend.sh <article.md>
# 前提: 首次需 --headed 扫码登录，后续用 --headless 复用会话
#
# 步骤: 登录 → 草稿编辑 → 原创 → 赞赏 → AI封面 → 创作来源 → 模板结尾 → 预览

set -euo pipefail

ARGS=()
HEADLESS=true

for arg in "$@"; do
    case "$arg" in
        --headed) HEADLESS=false ;;
        --headless) HEADLESS=true ;;
        --reset) rm -f .playwright-cli/storage-*.json ;;
        *) ARGS+=("$arg") ;;
    esac
done

if $HEADLESS; then
    OPEN_FLAGS="--no-headed"
else
    OPEN_FLAGS="--headed"
fi

VAULT="${MOONPUB_VAULT:-$HOME/Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain}"
CONFIG="${MOONPUB_CONFIG:-$VAULT/moonpub.toml}"
PLAYWRIGHT="npx @playwright/cli"

# ── Step 0: 打开微信后台 ──────────────────────────────────
echo "=== Step 0: 打开微信后台 ==="
$PLAYWRIGHT open "https://mp.weixin.qq.com" $OPEN_FLAGS
sleep 3

# 恢复登录状态
$PLAYWRIGHT state-load 2>/dev/null || true

# 检查是否已登录
PAGE_URL=$($PLAYWRIGHT eval "location.href" 2>/dev/null | head -1 || echo "")
if ! echo "$PAGE_URL" | grep -q "cgi-bin/home"; then
    if $HEADLESS; then
        echo "未登录！请先用 --headed 模式扫码登录一次"
        $PLAYWRIGHT close
        exit 1
    fi
    echo "请在浏览器中扫码登录，然后按 Enter 继续..."
    read
    $PLAYWRIGHT state-save 2>/dev/null || true
fi

# ── Step 1: 获取 token 并导航到草稿编辑 ─────────────────
echo "=== Step 1: 进入草稿编辑 ==="
TOKEN=$($PLAYWRIGHT eval "new URL(location.href).searchParams.get('token')" 2>/dev/null | grep -oE '\\d+' | head -1 || echo "")
sleep 2

# 进入草稿列表
$PLAYWRIGHT eval "location.href='/cgi-bin/appmsg?begin=0&count=10&type=77&action=list_card&token=$TOKEN&lang=zh_CN'"
sleep 3

# 找到文章并 hover
$PLAYWRIGHT eval "
var links=document.querySelectorAll('a');
for(var i=0;i<links.length;i++){
  if(links[i].textContent.includes('${ARTICLE_TITLE:-}')||true){
    links[i].dispatchEvent(new MouseEvent('mouseover',{bubbles:true}));
    return 'hovered';
  }
}
"

# 等编辑按钮出现后点击
sleep 1
$PLAYWRIGHT eval "
var links=document.querySelectorAll('a[href*=javascript]');
for(var i=0;i<links.length;i++){
  var img=links[i].querySelector('img');
  if(img&&links[i].offsetHeight>0&&!links[i].textContent.includes('发表')){
    links[i].click();
    return 'clicked edit';
  }
}
"
sleep 3

# 检查是否已进入编辑器
EDITOR_URL=$($PLAYWRIGHT eval "location.href.includes('appmsg_edit')" 2>/dev/null || echo "false")

# ── Step 2: 设置原创 ─────────────────────────────────────
echo "=== Step 2: 原创声明 ==="
$PLAYWRIGHT eval "
(function(){
  var all=document.querySelectorAll('*');
  for(var i=0;i<all.length;i++){
    if(all[i].textContent.trim()==='未声明'){
      all[i].parentElement.click();
      break;
    }
  }
})()
"
sleep 2
$PLAYWRIGHT eval "
(function(){
  var all=document.querySelectorAll('*');
  for(var i=0;i<all.length;i++){
    if(all[i].textContent.includes('已阅读并同意'))all[i].click();
  }
  var btns=document.querySelectorAll('button');
  for(var j=0;j<btns.length;j++){
    if(btns[j].textContent.trim()==='确定'){btns[j].click();return'done';}
  }
})()
"
sleep 2

# ── Step 3: 设置创作来源 ─────────────────────────────────
echo "=== Step 3: 创作来源 ==="
$PLAYWRIGHT eval "
(function(){
  var el=document.querySelector('#js_claim_source_area');
  if(el){
    var ch=el.querySelector('[class*=show],[class*=inner]');
    if(ch)ch.click();else el.click();
  }
})()
"
sleep 2
$PLAYWRIGHT eval "
(function(){
  var all=document.querySelectorAll('*');
  for(var i=0;i<all.length;i++){
    if(all[i].textContent.trim()==='个人观点，仅供参考'&&all[i].children.length===0){
      all[i].click();
      break;
    }
  }
  var btns=document.querySelectorAll('button');
  for(var j=0;j<btns.length;j++){
    if(btns[j].textContent.trim()==='确认'){btns[j].click();return'done';}
  }
})()
"
sleep 2

# ── Step 4: AI 配图封面 ───────────────────────────────────
echo "=== Step 4: AI 配图封面 ==="
$PLAYWRIGHT eval "
(function(){
  var e=document.querySelector('.js_cover_btn_area');
  if(e)e.dispatchEvent(new MouseEvent('mouseover',{bubbles:true}));
})()
"
sleep 1
$PLAYWRIGHT eval "
(function(){
  var e=document.querySelector('.js_aiImage');
  if(e)e.click();
})()
"
sleep 5
# 输入 prompt（从文章 frontmatter 自动提取标题）
$PLAYWRIGHT type "Book cover for article: book and letter theme, warm light, hope and resilience, editorial style"
sleep 1
$PLAYWRIGHT eval "document.querySelector('button.send-btn').click()"
sleep 12
$PLAYWRIGHT eval "
(function(){
  var imgs=document.querySelectorAll('img');
  for(var i=imgs.length-1;i>=0;i--){
    if(imgs[i].src.includes('mpimageai')&&imgs[i].naturalWidth>500){
      imgs[i].click();
      break;
    }
  }
})()
"
sleep 2
$PLAYWRIGHT eval "
(function(){
  var btns=document.querySelectorAll('button');
  for(var i=0;i<btns.length;i++){
    if(btns[i].textContent.trim()==='使用'){btns[i].click();return'done';}
  }
})()
"
sleep 2
$PLAYWRIGHT eval "
(function(){
  var btns=document.querySelectorAll('button');
  for(var i=0;i<btns.length;i++){
    if(btns[i].textContent.trim()==='确认'){btns[i].click();return'done';}
  }
})()
"
sleep 2

# ── Step 5: 模板结尾 ──────────────────────────────────────
echo "=== Step 5: 插入模板结尾 ==="
# 光标定位末尾
$PLAYWRIGHT eval "
(function(){
  var ed=document.querySelector('[contenteditable=true]');
  if(ed){
    ed.focus();
    var r=document.createRange();
    r.selectNodeContents(ed);
    r.collapse(false);
    window.getSelection().removeAllRanges();
    window.getSelection().addRange(r);
  }
})()
"
sleep 1
# 点模板按钮
$PLAYWRIGHT eval "
(function(){
  var lis=document.querySelectorAll('li');
  for(var i=0;i<lis.length;i++){
    if(lis[i].textContent.trim()==='模板'){lis[i].click();return'done';}
  }
})()
"
sleep 3
# 选择寻月阁标准结尾
$PLAYWRIGHT eval "
(function(){
  var all=document.querySelectorAll('*');
  for(var i=0;i<all.length;i++){
    if(all[i].textContent.trim()==='寻月阁标准结尾'&&all[i].offsetHeight>0){
      all[i].click();
      break;
    }
  }
})()
"
sleep 2
# 添加到正文
$PLAYWRIGHT eval "
(function(){
  var btns=document.querySelectorAll('button');
  for(var i=0;i<btns.length;i++){
    if(btns[i].textContent.includes('添加')){btns[i].click();return'done';}
  }
})()
"
sleep 2

# ── Step 6: 保存 → 预览 ──────────────────────────────────
echo "=== Step 6: 保存 → 预览 ==="
$PLAYWRIGHT eval "
(function(){
  var btns=document.querySelectorAll('button');
  for(var i=0;i<btns.length;i++){
    if(btns[i].textContent.trim()==='保存为草稿'){btns[i].click();return'done';}
  }
})()
"
sleep 2
$PLAYWRIGHT eval "
(function(){
  var btns=document.querySelectorAll('button');
  for(var i=0;i<btns.length;i++){
    if(btns[i].textContent.trim()==='预览'){btns[i].click();return'done';}
  }
})()
"
sleep 2
$PLAYWRIGHT eval "
(function(){
  var all=document.querySelectorAll('label');
  for(var i=0;i<all.length;i++){
    if(all[i].textContent.indexOf('通过公众号列表预览')>=0)all[i].click();
  }
  var btns=document.querySelectorAll('button');
  for(var j=0;j<btns.length;j++){
    if(btns[j].textContent.trim()==='确定'){btns[j].click();return'done';}
  }
})()
"
sleep 2

# 保存浏览器状态
$PLAYWRIGHT state-save 2>/dev/null || true

echo "=== 完成 ==="
