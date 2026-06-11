#!/usr/bin/env bash
# freepublish_submit.sh — 通过 API 发布微信公众号草稿
#
# 用法:
#   ./freepublish_submit.sh <media_id>
#   ./freepublish_submit.sh <media_id> "2026-05-25 09:00"  # 定时发布
#
# 在公众号后台完成原创声明、赞赏、合集设置后运行此脚本

set -euo pipefail

APPID="wxa110db6ca17c95a6"
# AppSecret 从环境变量读取，不写入脚本
# 运行前执行: export WECHAT_SECRET="your_secret"
SECRET="${WECHAT_SECRET:-}"

if [[ -z "$SECRET" ]]; then
    echo "❌ 请先设置环境变量: export WECHAT_SECRET='your_appsecret'"
    exit 1
fi

if [[ $# -eq 0 ]]; then
    echo "用法: $0 <media_id> [publish_time]"
    echo "示例: $0 EmukC2rjB9X3nj6feGSEr8..."
    echo "      $0 EmukC2rjB9X3nj6feGSEr8... '2026-05-25 09:00'"
    exit 1
fi

MEDIA_ID="$1"
PUBLISH_TIME="${2:-}"

# 获取 access_token
echo "▶ 获取 access_token..."
TOKEN_RESP=$(curl -s "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid=${APPID}&secret=${SECRET}")
ACCESS_TOKEN=$(python3 -c "import json,sys; d=json.loads('${TOKEN_RESP}'); print(d.get('access_token',''))")

if [[ -z "$ACCESS_TOKEN" ]]; then
    echo "❌ 获取 token 失败: ${TOKEN_RESP}"
    exit 1
fi

# 构建请求体
if [[ -n "$PUBLISH_TIME" ]]; then
    # 转换为 Unix 时间戳
    TS=$(date -jf "%Y-%m-%d %H:%M" "${PUBLISH_TIME}" "+%s" 2>/dev/null || \
         date -d "${PUBLISH_TIME}" "+%s" 2>/dev/null)
    BODY="{\"media_id\":\"${MEDIA_ID}\",\"publish_time\":${TS}}"
    echo "▶ 定时发布: ${PUBLISH_TIME}"
else
    BODY="{\"media_id\":\"${MEDIA_ID}\"}"
    echo "▶ 立即发布..."
fi

# 提交发布
RESULT=$(curl -s -X POST \
    "https://api.weixin.qq.com/cgi-bin/freepublish/submit?access_token=${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d "${BODY}")

python3 - "$RESULT" << 'PYEOF'
import json, sys

d = json.loads(sys.argv[1])
errcode = d.get('errcode', 0)
ERRORS = {
    48001: "API 功能未授权 — 此接口仅限已认证公众号，未认证账号请直接在后台手动发布",
    53503: "草稿未通过发布检查 — 请检查草稿内容",
    53504: "请前往公众平台官网使用草稿",
    53505: "请前往公众平台官网手动保存成功后再发布",
}
if errcode == 0:
    print(f"  ✅ 发布任务已提交  publish_id: {d.get('publish_id', '')}")
    print("     最终结果会通过服务器回调推送，或在公众号后台查看")
else:
    hint = ERRORS.get(errcode, d.get('errmsg', ''))
    print(f"  ❌ 发布失败 errcode={errcode}: {hint}")
    sys.exit(1)
PYEOF
