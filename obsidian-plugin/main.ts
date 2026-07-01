import { Notice, Plugin, PluginSettingTab, Setting } from "obsidian";
import { execFile, execFileSync } from "child_process";

interface MoonPubPluginSettings {
  moonpubPath: string;
  articlesRoot: string;
}

const DEFAULT_SETTINGS: MoonPubPluginSettings = {
  moonpubPath: "",
  articlesRoot: "",
};

export default class MoonPubPlugin extends Plugin {
  settings: MoonPubPluginSettings;
  private moonpubPath: string;

  async onload() {
    await this.loadSettings();
    this.moonpubPath = this.detectMoonpub();

    this.addSettingTab(new MoonPubSettingTab(this.app, this));

    this.addCommand({
      id: "moonpub-ship",
      name: "发布到微信公众号",
      callback: () => this.runShip(),
    });

    this.addCommand({
      id: "moonpub-preview",
      name: "预览文章",
      callback: () => this.runPreview(),
    });

    this.addCommand({
      id: "moonpub-ship-ai",
      name: "AI 润色后发布到公众号",
      callback: () => this.runShipAi(),
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

  private checkWechatRequirements(): boolean {
    if (!this.checkMoonpubInstalled()) return false;
    if (!process.env.WECHAT_APPID || !process.env.WECHAT_SECRET) {
      new Notice("❌ 请先设置环境变量 WECHAT_APPID 和 WECHAT_SECRET", 0);
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
    const args: string[] = [];
    if (this.settings.articlesRoot.trim()) {
      args.push("--articles", this.settings.articlesRoot.trim());
    }
    args.push(...subArgs, filePath);
    return args;
  }

  private runCmd(subcmd: string, requiresWechat: boolean, successMessage: string) {
    if (requiresWechat ? !this.checkWechatRequirements() : !this.checkMoonpubInstalled()) return;
    const filePath = this.getActiveFilePath();
    if (!filePath) return;

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

  private runShip() {
    this.runCmd("ship", true, "✅ 已推进到微信草稿，请去后台继续检查");
  }

  private runShipAi() {
    this.runCmd("ship --ai", true, "✅ 已完成 AI 润色并推进到微信草稿");
  }

  private runPreview() {
    this.runCmd("preview", false, "✅ 本地预览已完成");
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
