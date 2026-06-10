# MoonPub 浏览器自动化参考

## 工具

使用 `@playwright/cli`，通过 Bash 通道执行浏览器操作，Token 开销远低于 MCP。

```bash
npm install -g @playwright/cli@latest
```

## 微信后台自动化流程

### 1. 首次登录（需扫码一次）

```bash
# 有头模式打开微信后台
playwright-cli open "https://mp.weixin.qq.com" --headed --persistent
# 页面显示后扫码登录
# 后续可通过持久化会话恢复，无需重新扫码
```

### 2. 持久化会话（关键！）

```bash
# 登录成功后保存状态
playwright-cli state-save

# 下次启动后恢复
playwright-cli state-load
# 注：state-load 可能报错但 cookies 已生效，检查 URL 确认登录状态
```

### 3. 草稿列表 → 编辑

```bash
# 直接导航到草稿列表
playwright-cli eval "location.href='/cgi-bin/appmsg?begin=0&count=10&type=77&action=list_card&token=<token>&lang=zh_CN'"

# 找文章 (snapshot 后用 grep 搜标题)
playwright-cli snapshot | grep "文章标题关键词"

# Hover 文章标题 → 显示操作按钮（编辑/删除/发表）
playwright-cli hover <ref>

# 点击编辑按钮（hover 后出现的第一个 link 图标）
# 需要 snapshot 获取确切 ref
playwright-cli click <edit-ref>
```

### 4. 直接跳转编辑器

```bash
# 若已知 appmsgid 和 token，可直接导航到编辑器
playwright-cli eval "location.href='/cgi-bin/appmsg?t=media/appmsg_edit&action=edit&type=77&appmsgid=<id>&token=<token>&lang=zh_CN'"
```

### 5. 原创声明

```bash
# 找到"未声明"文字并点击其父元素
playwright-cli eval "document.querySelectorAll('*').forEach(el=>{if(el.textContent.trim()==='未声明')el.parentElement.click()})"

# 等待弹窗→勾协议→点确定
playwright-cli eval "document.querySelectorAll('*').forEach(el=>{if(el.textContent.includes('已阅读并同意'))el.click()})"
playwright-cli eval "document.querySelectorAll('button').forEach(b=>{if(b.textContent.trim()==='确定')b.click()})"
```

### 6. 赞赏

```bash
# 点击赞赏区域
playwright-cli eval "document.querySelectorAll('*').forEach(el=>{if(el.textContent.trim()==='赞赏'&&el.children.length===0)el.parentElement.click()})"
# 弹窗中选择"赞赏作者"→勾协议→确定
```

### 7. AI 配图封面

```bash
# Hover 封面区域
playwright-cli eval "document.querySelector('.js_cover_btn_area').dispatchEvent(new MouseEvent('mouseover',{bubbles:true}))"

# 点 AI 配图按钮
playwright-cli eval "document.querySelector('.js_aiImage').click()"

# 输入 prompt
playwright-cli type "prompt内容描述"

# 点击发送按钮 (注：只有 click，使用 send-btn 类名)
playwright-cli eval "document.querySelector('button.send-btn').click()"

# 等生成→选图→使用→确认
playwright-cli eval "document.querySelectorAll('img').forEach(i=>{if(i.src.includes('mpimageai')&&i.naturalWidth>500)i.click()})"
playwright-cli eval "document.querySelectorAll('button').forEach(b=>{if(b.textContent.trim()==='使用')b.click()})"
playwright-cli eval "document.querySelectorAll('button').forEach(b=>{if(b.textContent.trim()==='确认')b.click()})"
```

### 8. 创作来源

```bash
# 点击"未添加"区域
playwright-cli click <ref-for-未添加-next-to-创作来源>
# 选择"个人观点，仅供参考"
```

### 9. 模板插入（寻月阁标准结尾）

```bash
# 步骤1：光标定位到编辑器末尾
playwright-cli eval "document.querySelector('[contenteditable=true]').focus(); var r=document.createRange(); r.selectNodeContents(document.querySelector('[contenteditable=true]')); r.collapse(false); window.getSelection().removeAllRanges(); window.getSelection().addRange(r)"

# 步骤2：点击工具栏"模板"按钮
playwright-cli eval "document.querySelectorAll('li').forEach(l=>{if(l.textContent.trim()==='模板')l.click()})"

# 步骤3：等待模板列表→选"寻月阁标准结尾"
playwright-cli eval "document.querySelectorAll('*').forEach(el=>{if(el.textContent.trim()==='寻月阁标准结尾'&&el.offsetHeight>0)el.click()})"

# 步骤4：点击"添加到正文"
playwright-cli eval "document.querySelectorAll('button').forEach(b=>{if(b.textContent.includes('添加'))b.click()})"
```

### 10. 预览

```bash
# 点预览按钮
playwright-cli eval "document.querySelectorAll('button').forEach(b=>{if(b.textContent.trim()==='预览')b.click()})"
# 选"通过公众号列表预览"→确定
```

## 常见问题

### 1. ref 过期
**现象**：点击报 `Ref eXXX not found`
**原因**：页面 DOM 更新后 ref 失效
**解决**：重新 `snapshot` 获取新 ref

### 2. 弹窗重叠
**现象**：多个弹窗同时存在，点击被遮挡
**解决**：`document.querySelectorAll('button').forEach(b=>{if(b.textContent.trim()==='取消')b.click()})`

### 3. 会话丢失 (about:blank)
**现象**：长时间未操作后页面变成 about:blank
**原因**：微信后台 session 超时，或 playwright-cli 进程被终止
**解决**：`close` + `open --persistent`，然后 `state-load`

### 4. update-draft 后设置重置
**现象**：通过 API 更新草稿内容后，原创/赞赏/封面设置被重置
**原因**：WeChat API 更新草稿时某些元数据可能被清空
**解决**：API 更新后需在浏览器中重新设置原创+赞赏+封面+模板

### 5. extract_first_article 解析失败
**现象**：`[{` 搜索失败，JSON 有换行和缩进
**原因**：手写 JSON 的 `build_draft_json` 产出的 JSON 带 `\n` 和空格缩进
**解决**：改为从 `"articles"` 字符串定位，跳过 `[` 和空白后再找第一个 `{`

### 6. 已安装的二进制文件过期
**现象**：`cargo install --path .` 后再改代码，运行 `moonpub` 用的是旧版本
**原因**：`cargo install` 编译 release 模式安装到 `~/.cargo/bin/`
**解决**：代码修改后重新 `cargo install --path . --quiet`

### 7. AI 配图 prompt 输入注意事项
- prompt 需足够详细，包含书名/文章名
- 使用英文可获得更好效果
- 点击 `button.send-btn` 发送，不是按 Enter
- 新生成的图片在列表末尾（img index 较大）

### 8. playwright-cli eval 中 JS 语法限制
- 不能用 `const`、`let`（会报 SyntaxError），必须用 `var`
- 不能用 `return` 嵌套在复杂语句里，需拆分函数
- eval 参数是单行字符串，需要 IIFE 包裹
