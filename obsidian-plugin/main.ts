import { Notice, Plugin, PluginSettingTab, Setting } from "obsidian";
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
    if (!this.checkMoonpubInstalled()) return;
    const filePath = this.getActiveFilePath();
    if (!filePath) return;

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
        console.log("moonpub check:", payload);
      } catch (parseError) {
        console.error("moonpub check parse error:", parseError);
        new Notice("⚠ 状态检查已完成，但返回结果不是预期 JSON；请看控制台日志", 10_000);
        if (stdout.trim()) console.log("moonpub check raw:", stdout);
      }
    });
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
        console.log("moonpub workspace:", payload);
      } catch (parseError) {
        console.error("moonpub workspace parse error:", parseError);
        new Notice("⚠ 整体工作区状态已查询，但返回结果不是预期 JSON；请看控制台日志", 10_000);
        if (stdout.trim()) console.log("moonpub workspace raw:", stdout);
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
