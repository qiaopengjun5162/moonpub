import { App, Modal, Notice, Plugin, PluginSettingTab, Setting } from "obsidian";
import { execFile, execFileSync } from "child_process";

interface MoonPubPluginSettings {
  moonpubPath: string;
  articlesRoot: string;
}

interface MoonPubCapabilityTarget {
  id: string;
  display_name: string;
  kind: string;
  command: string[];
  article_arg: string;
  required_env: string[];
  required_config: string[];
  requires_network: boolean;
  requires_browser: boolean;
  risk: string;
  next_step: string;
}

interface MoonPubCapabilitiesPayload {
  schema_version: string;
  moonpub_version: string;
  targets: MoonPubCapabilityTarget[];
}

interface MoonPubCheckPayload {
  command: string;
  article_path: string;
  html_path: string;
  draft_json_path: string;
  media_id_path: string;
  has_markdown: boolean;
  has_html: boolean;
  has_draft_json: boolean;
  has_media_id: boolean;
  publishable: boolean;
  next_command: string;
  next_step: string;
}

interface MoonPubStatusFile {
  file: string;
  slug: string;
  latest_status: string;
  latest_detail: string;
}

interface MoonPubStatusStage {
  stage: string;
  count: number;
  files: MoonPubStatusFile[];
}

interface MoonPubWorkspaceCapability {
  id: string;
  kind: string;
  requires_network: boolean;
  requires_browser: boolean;
  next_step: string;
}

interface MoonPubWorkspacePayload {
  command: string;
  workspace_kind: string;
  entry_path: string;
  entry_path_label: string;
  total_articles: number;
  stage_counts: Record<string, number>;
  stages: MoonPubStatusStage[];
  capabilities: MoonPubWorkspaceCapability[];
  next_command: string;
  next_step: string;
}

interface MoonPubIntakeDraftPayload {
  command: string;
  inbox_path: string;
  draft_path: string;
  html_path?: string;
  action: string;
  next_command: string;
  pushed?: boolean;
  media_id?: string;
  stage?: string;
  next_step?: string;
}

interface MoonPubActiveContext {
  kind: "markdown" | "photo" | "other" | "none";
  path?: string;
  recommendedAction: string;
}

const PHOTO_EXTENSIONS = new Set(["jpg", "jpeg", "png", "heic", "webp"]);

class MoonPubArticleModal extends Modal {
  constructor(app: App, private payload: MoonPubCheckPayload) {
    super(app);
  }

  onOpen() {
    const { contentEl } = this;
    contentEl.empty();

    contentEl.createEl("h2", { text: "MoonPub 当前文章工作台" });
    contentEl.createEl("p", {
      text: `文章路径：${this.payload.article_path}`,
    });
    contentEl.createEl("p", {
      text: `当前状态：${this.payload.publishable ? "可继续发布" : "还没到可发布状态"}`,
    });

    const statusList = contentEl.createEl("ul");
    this.createStatusItem(statusList, "Markdown", this.payload.has_markdown);
    this.createStatusItem(statusList, "HTML", this.payload.has_html, this.payload.html_path);
    this.createStatusItem(
      statusList,
      "draft.json",
      this.payload.has_draft_json,
      this.payload.draft_json_path,
    );
    this.createStatusItem(
      statusList,
      "media_id",
      this.payload.has_media_id,
      this.payload.media_id_path,
    );

    contentEl.createEl("h3", { text: "推荐下一步" });
    const nextList = contentEl.createEl("ul");
    nextList.createEl("li", { text: this.payload.next_step });
    nextList.createEl("li", { text: this.payload.next_command });

    const hint = contentEl.createEl("p");
    hint.setText("推荐先把本地产物补齐，再决定是否推进到微信草稿。");
  }

  onClose() {
    this.contentEl.empty();
  }

  private createStatusItem(
    container: HTMLElement,
    label: string,
    ok: boolean,
    path?: string,
  ) {
    const suffix = ok ? "ok" : "missing";
    const text = path ? `${label}：${suffix}（${path}）` : `${label}：${suffix}`;
    container.createEl("li", { text });
  }
}

class MoonPubWorkspaceModal extends Modal {
  constructor(
    app: App,
    private payload: MoonPubWorkspacePayload,
    private activeContext: MoonPubActiveContext,
    private actions: {
      openCurrentArticle: () => void;
      previewCurrentArticle: () => void;
      intakeFeishu: () => void;
      intakePhotos: () => void;
    },
  ) {
    super(app);
  }

  onOpen() {
    const { contentEl } = this;
    contentEl.empty();

    contentEl.createEl("h2", { text: "MoonPub 首页工作台" });
    contentEl.createEl("p", {
      text: `当前入口：${this.payload.entry_path_label}`,
    });
    contentEl.createEl("p", {
      text: `工作区类型：${this.payload.workspace_kind}；文章总数：${this.payload.total_articles}`,
    });

    contentEl.createEl("h3", { text: "当前上下文" });
    const contextList = contentEl.createEl("ul");
    contextList.createEl("li", {
      text: `当前类型：${this.describeContextKind(this.activeContext.kind)}`,
    });
    if (this.activeContext.path) {
      contextList.createEl("li", {
        text: `当前路径：${this.activeContext.path}`,
      });
    }
    contextList.createEl("li", {
      text: `当前更推荐：${this.activeContext.recommendedAction}`,
    });

    contentEl.createEl("h3", { text: "首次建议" });
    const firstRunList = contentEl.createEl("ol");
    for (const step of this.firstRunSteps()) {
      firstRunList.createEl("li", { text: step });
    }

    const stageList = contentEl.createEl("ul");
    this.createStageItem(stageList, "drafts", "草稿中");
    this.createStageItem(stageList, "ready", "待发布");
    this.createStageItem(stageList, "published", "已发布");

    contentEl.createEl("h3", { text: "推荐下一步" });
    const nextList = contentEl.createEl("ul");
    nextList.createEl("li", { text: this.payload.next_step });
    nextList.createEl("li", { text: this.payload.next_command });

    contentEl.createEl("h3", { text: "首页入口" });
    const actionRow = contentEl.createDiv();
    actionRow.style.display = "flex";
    actionRow.style.flexWrap = "wrap";
    actionRow.style.gap = "8px";

    this.createActionButton(actionRow, "检查当前文章", this.actions.openCurrentArticle);
    this.createActionButton(actionRow, "预览当前文章", this.actions.previewCurrentArticle);
    this.createActionButton(actionRow, "导入最近飞书妙记", this.actions.intakeFeishu);
    this.createActionButton(actionRow, "导入当前图片目录", this.actions.intakePhotos);

    const riskyCapabilities = this.payload.capabilities.filter(
      (capability) => capability.requires_network || capability.requires_browser,
    );
    if (riskyCapabilities.length > 0) {
      contentEl.createEl("h3", { text: "风险边界" });
      const riskList = contentEl.createEl("ul");
      for (const capability of riskyCapabilities) {
        const riskText = [
          capability.id,
          capability.requires_network ? "会联网" : "",
          capability.requires_browser ? "会打开或控制 Chrome" : "",
          capability.next_step,
        ]
          .filter(Boolean)
          .join("｜");
        riskList.createEl("li", { text: riskText });
      }
    }

    const hint = contentEl.createEl("p");
    hint.setText("推荐先看工作区，再看当前文章，再决定要不要进入微信草稿。");
  }

  onClose() {
    this.contentEl.empty();
  }

  private createStageItem(container: HTMLElement, stageName: string, label: string) {
    const count = this.payload.stage_counts[stageName] ?? 0;
    const stage = this.payload.stages.find((item) => item.stage === stageName);
    const sample = stage?.files.slice(0, 2).map((file) => file.file).join("、");
    const text = sample ? `${label}：${count}（例如 ${sample}）` : `${label}：${count}`;
    container.createEl("li", { text });
  }

  private createActionButton(container: HTMLElement, label: string, action: () => void) {
    const button = container.createEl("button", { text: label });
    button.addEventListener("click", () => {
      action();
    });
  }

  private describeContextKind(kind: MoonPubActiveContext["kind"]): string {
    switch (kind) {
      case "markdown":
        return "当前打开的是 Markdown 文章";
      case "photo":
        return "当前打开的是图片";
      case "other":
        return "当前打开的是其他文件";
      default:
        return "当前没有打开文件";
    }
  }

  private firstRunSteps(): string[] {
    switch (this.activeContext.kind) {
      case "markdown":
        return [
          "先检查当前文章，确认 HTML、draft.json 和 media_id 缺什么",
          "再预览当前文章，先把本地排版看顺",
          "确认看懂当前文章工作台后，再决定是否推进到微信草稿",
        ];
      case "photo":
        return [
          "先导入当前图片目录，生成照片草稿和本地预览",
          "回到生成的草稿，确认这组照片被整理得是否符合预期",
          "看懂结果工作台后，再决定是否继续推进到微信草稿",
        ];
      case "other":
        return [
          "当前文件不是最适合的入口，先打开一篇 Markdown 或一张图片",
          "如果你现在没有现成文章，也可以直接走飞书素材入口",
          "先停在草稿和本地预览，不要一上来就直接发文",
        ];
      case "none":
      default:
        return [
          "先从飞书素材入口开始，或先打开一篇 Markdown / 一张图片",
          "优先把第一次体验目标定在草稿和本地预览，而不是立即推微信",
          "看懂首页、结果页和下一步动作后，再进入真实发布链路",
        ];
    }
  }
}

class MoonPubIntakeResultModal extends Modal {
  constructor(
    app: App,
    private title: string,
    private payload: MoonPubIntakeDraftPayload,
    private actions: {
      openDraft: () => void;
      checkDraft: () => void;
      previewDraft: () => void;
      pushDraft?: () => void;
    },
  ) {
    super(app);
  }

  onOpen() {
    const { contentEl } = this;
    contentEl.empty();

    contentEl.createEl("h2", { text: this.title });
    contentEl.createEl("p", {
      text: `本次动作：${this.payload.action === "updated" ? "更新已有草稿" : "新建草稿"}`,
    });

    const filesList = contentEl.createEl("ul");
    filesList.createEl("li", { text: `Inbox：${this.payload.inbox_path}` });
    filesList.createEl("li", { text: `Draft：${this.payload.draft_path}` });
    if (this.payload.html_path) {
      filesList.createEl("li", { text: `HTML 预览：${this.payload.html_path}` });
    }
    if (this.payload.pushed) {
      filesList.createEl("li", {
        text: `微信草稿：已推进（media_id: ${this.payload.media_id ?? "unknown"}）`,
      });
    } else {
      filesList.createEl("li", { text: "微信草稿：本次还未推进" });
    }

    contentEl.createEl("h3", { text: "推荐下一步" });
    const nextList = contentEl.createEl("ul");
    nextList.createEl("li", {
      text: this.payload.next_step ?? "先检查草稿和本地预览，再决定是否推进到微信草稿",
    });
    nextList.createEl("li", { text: this.payload.next_command });

    contentEl.createEl("h3", { text: "继续操作" });
    const actionsRow = contentEl.createDiv();
    actionsRow.style.display = "flex";
    actionsRow.style.flexWrap = "wrap";
    actionsRow.style.gap = "8px";

    this.createActionButton(actionsRow, "打开草稿", this.actions.openDraft);
    this.createActionButton(actionsRow, "检查草稿", this.actions.checkDraft);
    this.createActionButton(actionsRow, "预览草稿", this.actions.previewDraft);
    if (!this.payload.pushed && this.actions.pushDraft) {
      this.createActionButton(actionsRow, "推进到微信草稿", this.actions.pushDraft);
    }

    const hint = contentEl.createEl("p");
    hint.setText("推荐先回到草稿继续改，再决定是否直接推进到微信草稿或去微信后台检查。");
  }

  onClose() {
    this.contentEl.empty();
  }

  private createActionButton(container: HTMLElement, label: string, action: () => void) {
    const button = container.createEl("button", { text: label });
    button.addEventListener("click", () => {
      action();
    });
  }
}

const DEFAULT_SETTINGS: MoonPubPluginSettings = {
  moonpubPath: "",
  articlesRoot: "",
};

export default class MoonPubPlugin extends Plugin {
  settings: MoonPubPluginSettings;
  private moonpubPath: string;
  private capabilitiesCache: MoonPubCapabilitiesPayload | null = null;

  async onload() {
    await this.loadSettings();
    this.moonpubPath = this.detectMoonpub();

    this.addSettingTab(new MoonPubSettingTab(this.app, this));

    this.addCommand({
      id: "moonpub-ship",
      name: "发布到微信公众号",
      callback: () => void this.runShip(),
    });

    this.addCommand({
      id: "moonpub-preview",
      name: "预览文章",
      callback: () => void this.runPreview(),
    });

    this.addCommand({
      id: "moonpub-check",
      name: "检查当前文章状态",
      callback: () => void this.runCheck(),
    });

    this.addCommand({
      id: "moonpub-status",
      name: "查看整体文章池状态",
      callback: () => void this.runStatus(),
    });

    this.addCommand({
      id: "moonpub-home",
      name: "打开 MoonPub 首页",
      callback: () => void this.runStatus(),
    });

    this.addCommand({
      id: "moonpub-intake-feishu-preview",
      name: "导入最近一条飞书妙记并生成草稿预览",
      callback: () => void this.runFeishuLatestPreview(),
    });

    this.addCommand({
      id: "moonpub-intake-feishu-push",
      name: "导入最近一条飞书妙记并推进到微信草稿",
      callback: () => void this.runFeishuLatestPush(),
    });

    this.addCommand({
      id: "moonpub-intake-photos-preview",
      name: "导入当前图片所在目录并生成照片草稿预览",
      callback: () => void this.runPhotoDirectoryPreview(),
    });

    this.addCommand({
      id: "moonpub-ship-ai",
      name: "AI 润色后发布到公众号",
      callback: () => void this.runShipAi(),
    });
  }

  private async loadSettings() {
    this.settings = Object.assign({}, DEFAULT_SETTINGS, await this.loadData());
  }

  async saveSettings() {
    await this.saveData(this.settings);
    this.moonpubPath = this.detectMoonpub();
  }

  private detectMoonpub(): string {
    if (this.settings.moonpubPath && this.testCmd(this.settings.moonpubPath)) {
      return this.settings.moonpubPath;
    }

    const envPath = process.env.MOONPUB_PATH;
    if (envPath && this.testCmd(envPath)) return envPath;

    const commonPaths = [
      "/usr/local/bin/moonpub",
      "/opt/homebrew/bin/moonpub",
      "/usr/bin/moonpub",
      `${process.env.USERPROFILE ?? ""}\\.cargo\\bin\\moonpub.exe`,
      "C:\\Program Files\\moonpub\\moonpub.exe",
    ].filter(Boolean);

    for (const path of commonPaths) {
      if (this.testCmd(path)) return path;
    }

    try {
      execFileSync("moonpub", ["--help"], { stdio: "ignore" });
      return "moonpub";
    } catch {
      return "";
    }
  }

  private testCmd(path: string): boolean {
    try {
      execFileSync(path, ["--help"], { stdio: "ignore" });
      return true;
    } catch {
      return false;
    }
  }

  private checkMoonpubInstalled(): boolean {
    if (!this.moonpubPath || !this.testCmd(this.moonpubPath)) {
      new Notice("❌ MoonPub 未安装或路径无效，请先在插件设置里检查 moonpub 路径", 0);
      return false;
    }
    return true;
  }

  private getActiveFilePath(): string | null {
    const file = this.app.workspace.getActiveFile();
    if (!file) {
      new Notice("请先打开一个 Markdown 文件");
      return null;
    }
    return (this.app.vault.adapter as { getFullPath(path: string): string }).getFullPath(file.path);
  }

  private buildArgs(subcmd: string, filePath: string): string[] {
    const subArgs = subcmd.split(" ").filter(Boolean);
    const args = this.buildRootArgs();
    args.push(...subArgs, filePath);
    return args;
  }

  private buildRootArgs(): string[] {
    const args: string[] = [];
    if (this.settings.articlesRoot.trim()) {
      args.push("--articles", this.settings.articlesRoot.trim());
    }
    return args;
  }

  private normalizePath(path: string): string {
    return path.replace(/\\/g, "/");
  }

  private isPhotoPath(path: string): boolean {
    const normalized = this.normalizePath(path);
    const ext = normalized.split(".").pop()?.toLowerCase();
    return ext ? PHOTO_EXTENSIONS.has(ext) : false;
  }

  private relativeVaultPath(absolutePath: string): string | null {
    const vaultBase = this.normalizePath(this.app.vault.adapter.basePath ?? "");
    const target = this.normalizePath(absolutePath);
    if (!vaultBase || !target.startsWith(`${vaultBase}/`)) {
      return null;
    }
    return target.slice(vaultBase.length + 1);
  }

  private async openDraftInVault(absolutePath: string): Promise<boolean> {
    const relativePath = this.relativeVaultPath(absolutePath);
    if (!relativePath) return false;

    const file = this.app.vault.getAbstractFileByPath(relativePath);
    if (!file || !("path" in file)) return false;

    const leaf = this.app.workspace.getLeaf(true);
    await leaf.openFile(file);
    return true;
  }

  private async focusDraftPath(absolutePath: string): Promise<void> {
    const opened = await this.openDraftInVault(absolutePath);
    if (!opened) {
      new Notice(`📄 草稿已生成：${absolutePath}`, 10_000);
    }
  }

  private getActiveAssetPath(): string | null {
    const file = this.app.workspace.getActiveFile();
    if (!file) {
      new Notice("请先打开一张图片文件", 10_000);
      return null;
    }
    return (this.app.vault.adapter as { getFullPath(path: string): string }).getFullPath(file.path);
  }

  private getActiveContext(): MoonPubActiveContext {
    const file = this.app.workspace.getActiveFile();
    if (!file) {
      return {
        kind: "none",
        recommendedAction: "先从插件首页进入飞书素材入口，或打开一篇文章后再检查当前文章",
      };
    }

    const fullPath = (this.app.vault.adapter as { getFullPath(path: string): string }).getFullPath(file.path);
    const normalized = this.normalizePath(fullPath);
    if (normalized.endsWith(".md")) {
      return {
        kind: "markdown",
        path: normalized,
        recommendedAction: "先检查当前文章，再决定是否做本地预览或继续发布",
      };
    }
    if (this.isPhotoPath(normalized)) {
      return {
        kind: "photo",
        path: normalized,
        recommendedAction: "先导入当前图片目录，生成照片草稿和本地预览",
      };
    }
    return {
      kind: "other",
      path: normalized,
      recommendedAction: "先打开一篇 Markdown 或一张图片，再使用更贴合当前内容的入口",
    };
  }

  private loadCapabilities(): Promise<MoonPubCapabilitiesPayload | null> {
    if (this.capabilitiesCache) return Promise.resolve(this.capabilitiesCache);
    if (!this.checkMoonpubInstalled()) return Promise.resolve(null);

    return new Promise((resolve) => {
      execFile(this.moonpubPath, ["capabilities", "--json"], { env: process.env, timeout: 15_000 }, (err, stdout, stderr) => {
        if (err) {
          const msg = (stderr || err.message || "unknown capabilities error").trim();
          console.warn("moonpub capabilities error:", msg);
          resolve(null);
          return;
        }

        try {
          const parsed = JSON.parse(stdout) as MoonPubCapabilitiesPayload;
          if (!parsed.schema_version || !Array.isArray(parsed.targets)) {
            resolve(null);
            return;
          }
          this.capabilitiesCache = parsed;
          resolve(parsed);
        } catch (parseError) {
          console.warn("moonpub capabilities parse error:", parseError);
          resolve(null);
        }
      });
    });
  }

  private async showCapabilityNotice(capabilityId: string) {
    const payload = await this.loadCapabilities();
    const target = payload?.targets.find((item) => item.id === capabilityId);
    if (!target) return;

    const hints: string[] = [];
    if (target.requires_network) hints.push("会触达外部服务");
    if (target.requires_browser) hints.push("可能打开或控制 Chrome");
    if (target.required_env.length > 0) {
      hints.push(`通常需要 ${target.required_env.join(" / ")}；MoonPub 也会继续读取 .env 和 ~/.moonpub.env`);
    }
    if (target.required_config.length > 0) {
      hints.push(`依赖配置 ${target.required_config.join(" / ")}`);
    }
    hints.push("最终发布仍需你在微信后台确认");

    new Notice(`⚠ 发布前提示：${hints.join("；")}`, 10_000);
  }

  private async runCmd(subcmd: string, successMessage: string, capabilityId?: string) {
    if (!this.checkMoonpubInstalled()) return;
    const filePath = this.getActiveFilePath();
    if (!filePath) return;
    await this.runCmdForPath(filePath, subcmd, successMessage, capabilityId);
  }

  private async runCmdForPath(
    filePath: string,
    subcmd: string,
    successMessage: string,
    capabilityId?: string,
  ) {
    if (!this.checkMoonpubInstalled()) return;
    if (capabilityId) await this.showCapabilityNotice(capabilityId);

    const args = this.buildArgs(subcmd, filePath);
    const notice = new Notice(`🚀 ${subcmd}...`, 0);

    execFile(this.moonpubPath, args, { env: process.env, timeout: 300_000 }, (err, stdout, stderr) => {
      notice.hide();
      if (err) {
        const msg = (stderr || err.message || "未知错误").trim();
        new Notice(`❌ ${msg.slice(0, 120)}`, 0);
        console.error("moonpub error:", msg);
        return;
      }

      new Notice(successMessage);
      if (stdout.trim()) console.log("moonpub:", stdout);
    });
  }

  private async runPreviewForPath(filePath: string) {
    await this.runCmdForPath(filePath, "preview", "✅ 本地预览已完成");
  }

  private async runPushForPath(filePath: string) {
    await this.runCmdForPath(
      filePath,
      "push --render",
      "✅ 已推进到微信草稿，请去后台继续检查",
      "wechat-draft",
    );
  }

  private async runCheckForPath(filePath: string) {
    if (!this.checkMoonpubInstalled()) return;

    const args = this.buildArgs("check --json", filePath);
    const notice = new Notice("🔎 检查当前文章状态...", 0);

    execFile(this.moonpubPath, args, { env: process.env, timeout: 60_000 }, (err, stdout, stderr) => {
      notice.hide();
      if (err) {
        const msg = (stderr || err.message || "未知错误").trim();
        new Notice(`❌ ${msg.slice(0, 120)}`, 0);
        console.error("moonpub error:", msg);
        return;
      }

      try {
        const payload = JSON.parse(stdout) as MoonPubCheckPayload;
        const summary = [
          `publishable: ${payload.publishable ? "yes" : "no"}`,
          `html: ${payload.has_html ? "ok" : "missing"}`,
          `draft_json: ${payload.has_draft_json ? "ok" : "missing"}`,
          `media_id: ${payload.has_media_id ? "ok" : "missing"}`,
          `next: ${payload.next_command}`,
        ].join("；");

        new Notice(`📋 ${summary}`, 10_000);
        new MoonPubArticleModal(this.app, payload).open();
        console.log("moonpub check:", payload);
      } catch (parseError) {
        console.error("moonpub check parse error:", parseError);
        new Notice("⚠ 状态检查已完成，但返回结果不是预期 JSON；请看控制台日志", 10_000);
        if (stdout.trim()) console.log("moonpub check raw:", stdout);
      }
    });
  }

  private async runShip() {
    await this.runCmd("ship", "✅ 已推进到微信草稿，请去后台继续检查", "wechat-draft");
  }

  private async runShipAi() {
    await this.runCmd("ship --ai", "✅ 已完成 AI 润色并推进到微信草稿", "wechat-draft");
  }

  private async runPreview() {
    await this.runCmd("preview", "✅ 本地预览已完成");
  }

  private async runCheck() {
    const filePath = this.getActiveFilePath();
    if (!filePath) return;
    await this.runCheckForPath(filePath);
  }

  private async runStatus() {
    if (!this.checkMoonpubInstalled()) return;
    if (!this.settings.articlesRoot.trim()) {
      new Notice("请先在插件设置里配置 Articles 根目录，再查看整体文章池状态", 10_000);
      return;
    }

    const args = [...this.buildRootArgs(), "workspace", "--json"];
    const notice = new Notice("🗂 查看整体工作区状态...", 0);

    execFile(this.moonpubPath, args, { env: process.env, timeout: 60_000 }, (err, stdout, stderr) => {
      notice.hide();
      if (err) {
        const msg = (stderr || err.message || "未知错误").trim();
        new Notice(`❌ ${msg.slice(0, 120)}`, 0);
        console.error("moonpub error:", msg);
        return;
      }

      try {
        const payload = JSON.parse(stdout) as MoonPubWorkspacePayload;
        const stageCount = (stageName: string) => payload.stage_counts[stageName] ?? 0;
        const riskyTargets = payload.capabilities
          .filter((capability) => capability.requires_network || capability.requires_browser)
          .map((capability) => capability.id)
          .join(", ");
        const summary = [
          `entry: ${payload.entry_path_label}`,
          `drafts: ${stageCount("drafts")}`,
          `ready: ${stageCount("ready")}`,
          `published: ${stageCount("published")}`,
          `next: ${payload.next_command}`,
          riskyTargets ? `risky: ${riskyTargets}` : "",
        ].join("；");

        new Notice(`🗂 ${summary}`, 10_000);
        new MoonPubWorkspaceModal(this.app, payload, {
          kind: this.getActiveContext().kind,
          path: this.getActiveContext().path,
          recommendedAction: this.getActiveContext().recommendedAction,
        }, {
          openCurrentArticle: () => void this.runCheck(),
          previewCurrentArticle: () => void this.runPreview(),
          intakeFeishu: () => void this.runFeishuLatestPreview(),
          intakePhotos: () => void this.runPhotoDirectoryPreview(),
        }).open();
        console.log("moonpub workspace:", payload);
      } catch (parseError) {
        console.error("moonpub workspace parse error:", parseError);
        new Notice("⚠ 整体工作区状态已查询，但返回结果不是预期 JSON；请看控制台日志", 10_000);
        if (stdout.trim()) console.log("moonpub workspace raw:", stdout);
      }
    });
  }

  private async runFeishuLatestPreview() {
    const tip = [
      "会读取飞书妙记并调用 AI 生成草稿",
      "会生成本地 HTML 预览",
      "推荐先检查草稿和预览，再决定是否推进到微信草稿",
    ].join("；");
    new Notice(`⚠ 飞书入口提示：${tip}`, 10_000);
    await this.runStructuredIntakeCommand(
      ["intake", "feishu", "--latest", "--draft", "--preview", "--json"],
      "🪶 正在导入最近一条飞书妙记并生成草稿预览...",
      "✅ 飞书草稿和本地预览已生成",
      "MoonPub 飞书结果工作台",
    );
  }

  private async runFeishuLatestPush() {
    const tip = [
      "会读取飞书妙记并调用 AI 生成草稿",
      "会继续推到微信公众号草稿",
      "后续仍建议去微信后台检查预览和发布设置",
    ].join("；");
    new Notice(`⚠ 飞书入口提示：${tip}`, 10_000);
    await this.runStructuredIntakeCommand(
      ["intake", "feishu", "--latest", "--draft", "--push", "--json"],
      "🪶 正在导入最近一条飞书妙记并推进到微信草稿...",
      "✅ 飞书内容已推进到微信草稿",
      "MoonPub 飞书结果工作台",
    );
  }

  private async runPhotoDirectoryPreview() {
    const assetPath = this.getActiveAssetPath();
    if (!assetPath) return;
    if (!this.isPhotoPath(assetPath)) {
      new Notice("当前文件不是受支持的图片格式；请先打开 jpg/png/heic/webp 图片", 10_000);
      return;
    }
    const photoDir = this.normalizePath(assetPath).split("/").slice(0, -1).join("/");
    const tip = [
      "会把当前图片所在目录当成一组照片素材导入",
      "会调用 AI 生成草稿，并产出本地 HTML 预览",
      "适合整理同一天或同一组生活照片",
    ].join("；");
    new Notice(`⚠ 照片入口提示：${tip}`, 10_000);
    await this.runStructuredIntakeCommand(
      ["intake", "photos", photoDir, "--draft", "--preview", "--json"],
      "🖼 正在导入当前图片所在目录并生成照片草稿预览...",
      "✅ 照片草稿和本地预览已生成",
      "MoonPub 照片结果工作台",
    );
  }

  private async runStructuredIntakeCommand(
    args: string[],
    runningMessage: string,
    successMessage: string,
    modalTitle: string,
  ) {
    if (!this.checkMoonpubInstalled()) return;
    if (!this.settings.articlesRoot.trim()) {
      new Notice("请先在插件设置里配置 Articles 根目录，再使用素材入口", 10_000);
      return;
    }

    const rootArgs = [...this.buildRootArgs(), ...args];
    const notice = new Notice(runningMessage, 0);

    execFile(this.moonpubPath, rootArgs, { env: process.env, timeout: 300_000 }, (err, stdout, stderr) => {
      notice.hide();
      if (err) {
        const msg = (stderr || err.message || "未知错误").trim();
        new Notice(`❌ ${msg.slice(0, 120)}`, 0);
        console.error("moonpub feishu intake error:", msg);
        return;
      }

      try {
        const payload = JSON.parse(stdout) as MoonPubIntakeDraftPayload;
        void this.openDraftInVault(payload.draft_path).then((opened) => {
          if (opened) {
            new Notice("📄 已在 Obsidian 中打开生成的草稿", 8_000);
          }
        });
        const summary = [
          `action: ${payload.action}`,
          `draft: ${payload.draft_path}`,
          payload.html_path ? `html: ${payload.html_path}` : "",
          payload.pushed ? `media_id: ${payload.media_id ?? "unknown"}` : "",
          payload.next_step ? `next: ${payload.next_step}` : `next: ${payload.next_command}`,
        ]
          .filter(Boolean)
          .join("；");

        new Notice(`${successMessage}｜${summary}`, 12_000);
        new MoonPubIntakeResultModal(this.app, modalTitle, payload, {
          openDraft: () => void this.focusDraftPath(payload.draft_path),
          checkDraft: () => void this.runCheckForPath(payload.draft_path),
          previewDraft: () => void this.runPreviewForPath(payload.draft_path),
          pushDraft: payload.pushed
            ? undefined
            : () => void this.runPushForPath(payload.draft_path),
        }).open();
        console.log("moonpub feishu intake:", payload);
      } catch (parseError) {
        console.error("moonpub feishu intake parse error:", parseError);
        new Notice("⚠ 飞书链路已执行，但返回结果不是预期 JSON；请看控制台日志", 10_000);
        if (stdout.trim()) console.log("moonpub feishu intake raw:", stdout);
      }
    });
  }
}

class MoonPubSettingTab extends PluginSettingTab {
  plugin: MoonPubPlugin;

  constructor(app: any, plugin: MoonPubPlugin) {
    super(app, plugin);
    this.plugin = plugin;
  }

  display(): void {
    const { containerEl } = this;
    containerEl.empty();

    containerEl.createEl("h2", { text: "MoonPub 插件设置" });

    new Setting(containerEl)
      .setName("MoonPub 可执行文件路径")
      .setDesc("可留空，插件会自动尝试常见路径和 PATH。")
      .addText((text) =>
        text
          .setPlaceholder("/usr/local/bin/moonpub")
          .setValue(this.plugin.settings.moonpubPath)
          .onChange(async (value) => {
            this.plugin.settings.moonpubPath = value.trim();
            await this.plugin.saveSettings();
          }),
      );

    new Setting(containerEl)
      .setName("Articles 根目录")
      .setDesc("可选。设置后插件会自动以 --articles <path> 调用 moonpub，适合你的 Vault 不是文章根目录的情况。")
      .addText((text) =>
        text
          .setPlaceholder("/Users/you/Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain")
          .setValue(this.plugin.settings.articlesRoot)
          .onChange(async (value) => {
            this.plugin.settings.articlesRoot = value.trim();
            await this.plugin.saveSettings();
          }),
      );
  }
}
