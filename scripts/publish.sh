#!/usr/bin/env bash
# publish.sh — 一键发布到微信公众号 + Zola 博客
#
# 用法:
#   ./publish.sh Articles/ready/my-article.md
#
# 前置条件:
#   1. Articles/ready/<slug>.draft.json 已存在（WeChat 草稿用，由 Claude Code 生成）
#   2. md2wechat 已配置好 AppID/AppSecret（~/.config/md2wechat/config.yaml）
#   3. Zola 博客 git remote 可正常推送

set -euo pipefail

# ── 配置 ────────────────────────────────────────────────────
VAULT="/Users/qiaopengjun/Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain"
BLOG_DIR="/Users/qiaopengjun/Blog/myblog"
# ────────────────────────────────────────────────────────────

# 参数检查
if [[ $# -eq 0 ]]; then
    echo "用法: $0 <article.md>"
    echo "示例: $0 Articles/ready/读人性的弱点-三个处世密码.md"
    exit 1
fi

INPUT="$1"
[[ "$INPUT" = /* ]] || INPUT="${VAULT}/${INPUT}"
[[ -f "$INPUT" ]] || { echo "❌ 文件不存在: $INPUT"; exit 1; }

SLUG=$(basename "$INPUT" .md)
ARTICLE_DIR=$(dirname "$INPUT")
DRAFT_JSON="${ARTICLE_DIR}/${SLUG}.draft.json"

echo "▶ 发布: ${SLUG}"
echo ""

# ── 解析 frontmatter（写临时 JSON 避免 shell 转义问题）────────
META_FILE=$(mktemp /tmp/publish_meta.XXXXXX.json)
trap "rm -f ${META_FILE}" EXIT

python3 - "$INPUT" "$META_FILE" << 'PYEOF'
import sys, re, json
from datetime import date as today

src, out = sys.argv[1], sys.argv[2]
text = open(src).read()

def get(k):
    m = re.search(rf'^{k}:\s*(.+)', text, re.MULTILINE)
    return m.group(1).strip().strip('"') if m else ''

title = get('title')
date  = get('date') or today.today().isoformat()

tags_m = re.search(r'^tags:\s*\[(.+?)\]', text, re.MULTILINE)
tags   = [t.strip().strip('"') for t in tags_m.group(1).split(',')] if tags_m else ['读书笔记']

json.dump({'title': title, 'date': date, 'tags': tags}, open(out, 'w'), ensure_ascii=False)
PYEOF

TITLE=$(python3 -c "import json; print(json.load(open('${META_FILE}'))['title'])")
DATE=$(python3  -c "import json; print(json.load(open('${META_FILE}'))['date'])")

echo "  标题: ${TITLE}"
echo "  日期: ${DATE}"
echo ""

# ── Part 0: HTML 结构验证 ────────────────────────────────────
echo "[验证] 检查 HTML 结构..."

HTML_FILE="${ARTICLE_DIR}/${SLUG}.html"
if [[ -f "$HTML_FILE" ]]; then
    python3 "${VAULT}/validate_html.py" "${HTML_FILE}"
else
    echo "  ⚠️  ${SLUG}.html 不存在，跳过验证"
fi

echo ""

# ── Part 1: 微信公众号 ───────────────────────────────────────
echo "[1/3] 推送微信草稿..."

WECHAT_OK=false
MEDIA_ID=""

if [[ ! -f "$DRAFT_JSON" ]]; then
    echo "  ⚠️  ${SLUG}.draft.json 不存在"
    echo "     请先让 Claude Code 生成 HTML 和 draft.json，再运行此脚本"
else
    cd "$VAULT"
    RESULT_FILE=$(mktemp /tmp/wechat_result.XXXXXX.json)
    md2wechat create_draft "${DRAFT_JSON}" --json > "${RESULT_FILE}" 2>/dev/null || echo '{}' > "${RESULT_FILE}"

    MEDIA_ID=$(python3 -c "
import json, sys
d = json.load(open('${RESULT_FILE}'))
print(d.get('data', {}).get('media_id', ''))
" 2>/dev/null || true)
    rm -f "${RESULT_FILE}"

    if [[ -n "$MEDIA_ID" ]]; then
        echo "  ✅ 推送成功"
        echo "  media_id: ${MEDIA_ID}"
        echo "$MEDIA_ID" > "${ARTICLE_DIR}/${SLUG}.media_id"
        WECHAT_OK=true
    else
        CURRENT_IP=$(curl -s --connect-timeout 5 --noproxy '*' https://api.ipify.org 2>/dev/null || echo "获取失败")
        echo "  ❌ 推送失败"
        echo "  当前 IP: ${CURRENT_IP} — 请确认已加入微信 API IP 白名单后重试"
    fi
fi

echo ""

# ── Part 2: 生成 Zola 博客文章 ──────────────────────────────
echo "[2/3] 生成 Zola 博客文章..."

ZOLA_FILE="${BLOG_DIR}/content/${DATE}-${SLUG}.md"

python3 - "$INPUT" "$META_FILE" "$ZOLA_FILE" << 'PYEOF'
import sys, re, json

src, meta_file, dst = sys.argv[1], sys.argv[2], sys.argv[3]
text = open(src).read()
meta = json.load(open(meta_file))

# 分离 frontmatter 和正文
fm_end = re.match(r'^---\s*\n.*?\n---\s*\n', text, re.DOTALL)
body = text[fm_end.end():].lstrip() if fm_end else text

# 把微信 CDN banner 图替换为博客本地路径，去掉"点个赞"文字
body = re.sub(
    r'!\[.*?\]\(https?://mmbiz\.qpic\.cn/[^\)]+\)',
    '![关注寻月隐君](/images/wechat-follow.png)',
    body
)
# 去掉微信专属的"点个赞"提示文字
body = re.sub(r'\n点个"赞".*?「寻月者」.*?。\n?', '\n', body)

# 构建 Zola TOML frontmatter
tags_toml = ', '.join(f'"{t}"' for t in meta['tags'])
toml = (
    f'+++\n'
    f'title = "{meta["title"]}"\n'
    f'description = "{meta["title"]}"\n'
    f'date = {meta["date"]}T00:00:00Z\n'
    f'[taxonomies]\n'
    f'categories = ["读书"]\n'
    f'tags = [{tags_toml}]\n'
    f'+++\n'
    f'\n'
    f'<!-- more -->\n'
    f'\n'
)

with open(dst, 'w') as f:
    f.write(toml + body)

print(f"  ✅ 已生成: {dst}")
PYEOF

echo ""

# ── Part 3: Git commit + push ────────────────────────────────
echo "[3/3] Git commit + push..."

cd "$BLOG_DIR"
git add "content/${DATE}-${SLUG}.md"
git commit -m "post: ${TITLE}"
git push

echo "  ✅ 完成"
echo ""

# ── 更新文章状态 ready → published ──────────────────────────
PUBLISHED_DIR="${VAULT}/Articles/published"
mkdir -p "$PUBLISHED_DIR"
mv "$INPUT" "${PUBLISHED_DIR}/${SLUG}.md"
[[ -f "${ARTICLE_DIR}/${SLUG}.html" ]]       && mv "${ARTICLE_DIR}/${SLUG}.html"       "${PUBLISHED_DIR}/${SLUG}.html"
[[ -f "$DRAFT_JSON" ]] && mv "$DRAFT_JSON" "${PUBLISHED_DIR}/${SLUG}.draft.json"
echo "  ✅ 文章已移至 Articles/published/"
echo ""

# ── 总结 ─────────────────────────────────────────────────────
echo "══════════════════════════════════════════"
if $WECHAT_OK; then
    echo "微信公众号 ✅  草稿已推送"
    echo ""
    echo "  接下来（手动 ~1 分钟）："
    echo "  1. 公众号后台设置封面图、原创声明、赞赏、合集"
    echo "  2. 完成后发布，或运行定时发布:"
    echo "     ${VAULT}/freepublish_submit.sh ${MEDIA_ID}"
    # Hermes 通知（后台运行，不阻塞）
    ~/.local/bin/hermes chat -q "微信草稿推送成功 ✅
文章：${TITLE}
media_id：${MEDIA_ID}
博客：https://paxonqiao.com
下一步：公众号后台设置封面图、原创声明，然后发布。" -Q 2>/dev/null &
else
    echo "微信公众号 ⚠️   推送失败，修复 IP 后重新运行此脚本"
    # Hermes 通知失败原因
    CURRENT_IP=$(curl -s --connect-timeout 5 --noproxy '*' https://api.ipify.org 2>/dev/null || echo "获取失败")
    ~/.local/bin/hermes chat -q "微信草稿推送失败 ❌
文章：${TITLE}
原因：IP 不在白名单
当前 IP：${CURRENT_IP}
请到 https://developers.weixin.qq.com/platform/ 添加后重试。" -Q 2>/dev/null &
fi
echo ""
echo "Zola 博客    ✅  https://paxonqiao.com"
echo "══════════════════════════════════════════"
