# MoonPub Obsidian 插件 BRAT/社区市场上架准备 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让普通用户能通过 BRAT 一键安装 Obsidian 插件，并为后续社区市场上架补齐发布产物；同时让插件保存的微信预览接收人也能被 CLI 在终端复用。

**Architecture:** 复用现有 GitHub release 流程，新增一个 job 把 `main.js` + `manifest.json` (+ `styles.css`) 打包成 `moonpub-obsidian-plugin-X.Y.Z.zip` 并随 GitHub Release 发布；插件版本号与 CLI 对齐；在 `runShipWithRecipientCheck` 保存接收人时同时写入项目级 `.moonpub/preview_to`，使插件设置和终端命令共享同一份默认值。

**Tech Stack:** TypeScript, esbuild, GitHub Actions, Obsidian plugin manifest, BRAT

---

## File Structure

| File | Change | Responsibility |
|------|--------|----------------|
| `obsidian-plugin/manifest.json` | Modify | 插件 ID、版本号、最小 Obsidian 版本等元数据，Obsidian/BRAT 安装时读取 |
| `obsidian-plugin/package.json` | Modify | 与 manifest 版本保持一致，避免内部版本混乱 |
| `obsidian-plugin/styles.css` | Create (empty) | Obsidian 插件可选样式文件；BRAT 和部分用户期望 release 里有它 |
| `obsidian-plugin/main.ts` | Modify | 保存微信预览接收人时同时写入 `.moonpub/preview_to` |
| `obsidian-plugin/workflow-ui.test.mjs` | Modify | 新增 `.moonpub/preview_to` 辅助函数单元测试 |
| `.github/workflows/release.yml` | Modify | 新增插件打包 job，把插件产物附加到 GitHub Release |
| `obsidian-plugin/README.md` | Modify | 补充 BRAT 安装步骤和社区市场上架前置说明 |

---

## Task 1: 对齐插件版本号到 CLI 版本

**Files:**
- Modify: `obsidian-plugin/manifest.json`
- Modify: `obsidian-plugin/package.json`
- Modify: `obsidian-plugin/package-lock.json`
- Test: `npm test` (typecheck + existing tests)

- [ ] **Step 1: 修改 manifest.json 版本**

  ```json
  {
    "id": "moonpub",
    "name": "MoonPub",
    "version": "0.4.2",
    "minAppVersion": "1.5.0",
    "description": "从 Obsidian 调用本地 MoonPub，把 Markdown 推进到微信公众号草稿并辅助后台配置。",
    "author": "Paxon Qiao",
    "authorUrl": "https://paxonqiao.com",
    "isDesktopOnly": true
  }
  ```

- [ ] **Step 2: 修改 package.json 版本**

  ```json
  {
    "name": "obsidian-moonpub",
    "version": "0.4.2",
    "description": "从 Obsidian 调用本地 MoonPub 发布副驾驶",
    ...
  }
  ```

- [ ] **Step 3: 重新生成 package-lock.json**

  Run:
  ```bash
  cd /Users/qiaopengjun/Code/Rust/moonpub/obsidian-plugin
  npm install
  ```

  Expected: `package-lock.json` 里的 `"version": "0.4.2"` 与 `package.json` 一致。

- [ ] **Step 4: 运行测试**

  Run:
  ```bash
  cd /Users/qiaopengjun/Code/Rust/moonpub/obsidian-plugin
  npm test
  ```

  Expected: `workflow-ui.test.mjs` 全部通过，typecheck 无错误。

- [ ] **Step 5: Commit**

  ```bash
  git add obsidian-plugin/manifest.json obsidian-plugin/package.json obsidian-plugin/package-lock.json
  git commit -m "chore(obsidian-plugin): sync plugin version to 0.4.2"
  ```

---

## Task 2: 创建空的 styles.css

**Files:**
- Create: `obsidian-plugin/styles.css`
- Modify: `.github/workflows/release.yml` (见 Task 4)

- [ ] **Step 1: 创建空样式文件**

  ```css
  /* MoonPub Obsidian plugin styles - currently no custom CSS needed. */
  ```

  说明：Obsidian 插件不强制要求 `styles.css`，但 BRAT 下载 release 时如果仓库习惯附带该文件，会减少用户疑惑；空文件即可，不引入任何样式。

- [ ] **Step 2: Commit**

  ```bash
  git add obsidian-plugin/styles.css
  git commit -m "chore(obsidian-plugin): add empty styles.css for release packaging"
  ```

---

## Task 3: 插件保存预览接收人时同步写入 `.moonpub/preview_to`

**Files:**
- Modify: `obsidian-plugin/workflow-ui.ts`
- Modify: `obsidian-plugin/main.ts`
- Modify: `obsidian-plugin/workflow-ui.test.mjs`
- Test: `npm test`

- [ ] **Step 1: 在 workflow-ui.ts 新增持久化辅助函数**

  ```typescript
  export function previewToFilePath(articlesRoot: string): string {
    return `${articlesRoot}/.moonpub/preview_to`;
  }

  export async function persistPreviewTo(articlesRoot: string, wxid: string): Promise<void> {
    const root = articlesRoot.trim();
    const id = wxid.trim();
    if (!root || !id) return;

    try {
      const fs = await import("node:fs/promises");
      const path = await import("node:path");
      const dir = path.join(root, ".moonpub");
      await fs.mkdir(dir, { recursive: true });
      await fs.writeFile(path.join(dir, "preview_to"), id, "utf8");
    } catch {
      // Project-level persistence is best-effort; plugin settings remain the source of truth.
    }
  }
  ```

- [ ] **Step 2: 在 main.ts 的 runShipWithRecipientCheck 中调用 persistPreviewTo**

  修改这段（当前第 1637-1649 行附近）：

  ```typescript
  new MoonPubPreviewRecipientModal(this.app, {
    saveAndRun: async (wxid) => {
      const trimmed = wxid.trim();
      if (trimmed) {
        this.settings.wechatPreviewTo = trimmed;
        await this.saveSettings();
        await persistPreviewTo(this.settings.articlesRoot, trimmed);
      }
      await this.runCmdForPath(filePath, subcmd, successMessage, "wechat-draft");
    },
    skipAndRun: async () => {
      await this.runCmdForPath(filePath, subcmd, successMessage, "wechat-draft");
    },
  }).open();
  ```

  并在文件顶部 `import` 中加入 `persistPreviewTo`：

  ```typescript
  import {
    ActiveContextKind,
    contextKindLabel,
    firstRunSteps,
    needsPreviewRecipientPrompt,
    persistPreviewTo,
    previewRecipientEnv,
    replaceModal,
    workspacePathLabel,
  } from "./workflow-ui";
  ```

- [ ] **Step 3: 新增单元测试**

  在 `workflow-ui.test.mjs` 末尾追加：

  ```javascript
  import { mkdtemp, readFile, rm } from "node:fs/promises";
  import { tmpdir } from "node:os";
  import { join } from "node:path";

  test("preview_to project-level persistence writes to .moonpub/preview_to", async () => {
    const dir = await mkdtemp(join(tmpdir(), "moonpub-preview-to-"));
    try {
      await ui.persistPreviewTo(dir, "my-wxid");
      const content = await readFile(join(dir, ".moonpub", "preview_to"), "utf8");
      assert.equal(content, "my-wxid");
    } finally {
      await rm(dir, { force: true, recursive: true });
    }
  });

  test("preview_to path helper joins articles root", () => {
    assert.equal(ui.previewToFilePath("/vault/articles"), "/vault/articles/.moonpub/preview_to");
  });
  ```

- [ ] **Step 4: 运行测试**

  Run:
  ```bash
  cd /Users/qiaopengjun/Code/Rust/moonpub/obsidian-plugin
  npm test
  ```

  Expected: 新增 2 个测试通过，原有测试不变。

- [ ] **Step 5: Commit**

  ```bash
  git add obsidian-plugin/workflow-ui.ts obsidian-plugin/main.ts obsidian-plugin/workflow-ui.test.mjs
  git commit -m "feat(obsidian-plugin): persist preview recipient to .moonpub/preview_to"
  ```

---

## Task 4: 在 Release Workflow 中打包并发布 Obsidian 插件产物

**Files:**
- Modify: `.github/workflows/release.yml`
- Test: 通过 `act` 或下次打 tag 后观察 GitHub Release（无法本地完整测试时只验证 YAML 语法）

- [ ] **Step 1: 在 release.yml 的 `test` job 后新增 `build-obsidian-plugin` job**

  在 `build:` job 之前插入：

  ```yaml
  build-obsidian-plugin:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm
          cache-dependency-path: obsidian-plugin/package-lock.json

      - name: Build plugin
        working-directory: obsidian-plugin
        run: |
          npm ci
          npm test
          npm run build

      - name: Package Obsidian plugin
        working-directory: obsidian-plugin
        run: |
          mkdir -p ../dist-obsidian
          zip -j ../dist-obsidian/moonpub-obsidian-plugin-${{ github.ref_name }}.zip main.js manifest.json styles.css

      - name: Upload Obsidian plugin artifact
        uses: actions/upload-artifact@v4
        with:
          name: moonpub-obsidian-plugin
          path: dist-obsidian/*
  ```

- [ ] **Step 2: 修改 `release` job 下载并附加插件产物**

  在 `release` job 的 `steps:` 中，在 `Generate changelog` 之前加入下载：

  ```yaml
      - uses: actions/download-artifact@v4
        with:
          path: dist
          pattern: moonpub-*
          merge-multiple: true
  ```

  原有的 `actions/download-artifact` 调用（下载 CLI 产物）已经使用 `pattern: moonpub-*`，会把 `moonpub-obsidian-plugin` artifact 也下载到 `dist/` 目录，因此只需保留一份。确认最终 `dist/` 目录下包含：
  - `moonpub-linux-amd64.tar.gz`
  - `moonpub-macos-arm64.tar.gz`
  - ...
  - `moonpub-obsidian-plugin-v0.4.2.zip`

  `softprops/action-gh-release` 的 `files: dist/*` 会把它们全部上传。

- [ ] **Step 3: 验证 YAML 语法**

  Run:
  ```bash
  cd /Users/qiaopengjun/Code/Rust/moonpub
  python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))" && echo "YAML OK"
  ```

  Expected: 输出 `YAML OK`，无解析错误。

- [ ] **Step 4: Commit**

  ```bash
  git add .github/workflows/release.yml
  git commit -m "ci(release): package obsidian plugin assets for BRAT"
  ```

---

## Task 5: 更新插件 README，补充 BRAT 安装说明

**Files:**
- Modify: `obsidian-plugin/README.md`

- [ ] **Step 1: 替换“安装方式”章节**

  找到第 67-118 行左右的“安装方式”并替换为：

  ```markdown
  ## 安装方式

  推荐普通用户通过 **BRAT** 安装，以获得自动更新；技术用户也可以手动复制目录。

  ### 方式一：BRAT 安装（推荐）

  1. 在 Obsidian 的第三方插件市场里安装并启用 **BRAT**。
  2. 打开 BRAT 设置，点击 `Add Beta plugin with frozen version` 或 `Add Beta plugin`。
  3. 填入仓库地址：
     ```
     https://github.com/qiaopengjun5162/moonpub
     ```
  4. BRAT 会自动下载最新 Release 中的 `moonpub-obsidian-plugin-vX.Y.Z.zip`，解压到 `.obsidian/plugins/moonpub/`。
  5. 进入 Obsidian `设置 → 第三方插件`，启用 `MoonPub`。
  6. （首次使用）按下面“首次配置”步骤填写 `Articles 根目录` 和 `MoonPub 可执行文件路径`。

  ### 方式二：手动复制（开发/测试）

  1. 先安装 MoonPub CLI，确保终端里能运行：
     ```bash
     moonpub --help
     ```
  2. 把本仓库中的 `obsidian-plugin/` 复制到你的 vault：
     ```text
     .obsidian/plugins/moonpub/
     ```
  3. 在插件目录中运行：
     ```bash
     npm ci
     npm test
     npm run build
     ```
  4. 在 Obsidian 中启用第三方插件里的 `MoonPub`。
  ```

- [ ] **Step 2: 在 README 末尾新增“社区市场上架状态”小节**

  在 README 末尾追加：

  ```markdown
  ## 社区市场上架状态

  - [x] 插件 manifest、main.js、styles.css 已随 Release 发布
  - [ ] 已提交 PR 到 [obsidianmd/obsidian-releases](https://github.com/obsidianmd/obsidian-releases)
  - [ ] 已通过社区市场审核

  在通过社区市场审核前，请先用 **BRAT** 安装。
  ```

- [ ] **Step 3: Commit**

  ```bash
  git add obsidian-plugin/README.md
  git commit -m "docs(obsidian-plugin): add BRAT install instructions and market checklist"
  ```

---

## Task 6: 验证整链条（本地预演）

**Files:**
- 全部已修改文件
- Test: `npm test`, `npm run build`, GitHub Actions YAML 语法

- [ ] **Step 1: 本地构建插件产物**

  Run:
  ```bash
  cd /Users/qiaopengjun/Code/Rust/moonpub/obsidian-plugin
  npm ci
  npm test
  npm run build
  ```

  Expected: 生成 `main.js`、`manifest.json`、`styles.css`（新建）在同一目录。

- [ ] **Step 2: 手动打包验证**

  Run:
  ```bash
  cd /Users/qiaopengjun/Code/Rust/moonpub/obsidian-plugin
  zip -j /tmp/moonpub-obsidian-plugin-v0.4.2.zip main.js manifest.json styles.css
  unzip -l /tmp/moonpub-obsidian-plugin-v0.4.2.zip
  ```

  Expected: zip 内包含 `main.js`、`manifest.json`、`styles.css` 三个文件。

- [ ] **Step 3: 运行 Rust CI 检查**

  Run:
  ```bash
  cd /Users/qiaopengjun/Code/Rust/moonpub
  cargo fmt --all -- --check
  cargo clippy --all-targets --all-features --tests --benches -- -D warnings
  cargo nextest run --all-features
  ```

  Expected: 全部通过。

- [ ] **Step 4: Commit 最终调整（如有）**

  ```bash
  git add .
  git commit -m "chore: final checks for obsidian plugin release packaging"
  ```

---

## Self-Review

**1. Spec coverage:**
- BRAT 一键安装 → Task 4 在 Release 中打包插件 assets，Task 5 在 README 中写 BRAT 步骤。
- 自动更新 → BRAT 自动跟踪 GitHub Release，Task 4 确保每次 tag 都上传 zip。
- 版本一致 → Task 1 对齐 manifest/package/CLI 版本。
- 社区市场上架前置 → Task 5 的 checklist 和 Task 4 的 release assets 已补齐。
- preview_to 项目级共享 → Task 3 在保存时写入 `.moonpub/preview_to`。

**2. Placeholder scan:**
- 无 TBD/TODO。
- 所有代码块包含完整代码。
- 所有命令包含完整命令和预期输出。

**3. Type consistency:**
- `persistPreviewTo` 签名在 Task 3 定义后，在 main.ts 中调用一致。
- 版本号统一为 `0.4.2`（与 `Cargo.toml` 一致）。

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-07-25-obsidian-plugin-brat-release.md`.**

Two execution options:

**1. Subagent-Driven (recommended)** - Dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach?
