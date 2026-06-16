import { Plugin, Notice, Platform } from "obsidian";
import { exec, execSync } from "child_process";

export default class MoonPubPlugin extends Plugin {
  private moonpubPath: string;

  async onload() {
    // Detect moonpub binary
    this.moonpubPath = this.detectMoonpub();

    // --- Command: Publish to WeChat ---
    this.addCommand({
      id: "moonpub-ship",
      name: "发布到微信公众号",
      callback: () => this.runShip(),
    });

    // --- Command: Render & Preview ---
    this.addCommand({
      id: "moonpub-preview",
      name: "预览文章",
      callback: () => this.runCmd("preview"),
    });

    // --- Command: AI Polish then Publish ---
    this.addCommand({
      id: "moonpub-ship-ai",
      name: "AI 润色后发布到公众号",
      callback: () => this.runShipAi(),
    });
  }

  private detectMoonpub(): string {
    // 1. Explicit config
    const configured = (this as any).app?.plugins?.plugins?.moonpub?.settings?.moonpubPath;
    if (configured && this.testCmd(configured)) return configured;

    // 2. Common paths
    if (Platform.isMacOS) {
      const paths = ["/usr/local/bin/moonpub", "/opt/homebrew/bin/moonpub"];
      for (const p of paths) if (this.testCmd(p)) return p;
    }
    if (Platform.isLinux) {
      if (this.testCmd("/usr/local/bin/moonpub")) return "/usr/local/bin/moonpub";
    }
    if (Platform.isWin) {
      const paths = [
        `${process.env.USERPROFILE}\\.cargo\\bin\\moonpub.exe`,
        `C:\\Program Files\\moonpub\\moonpub.exe`,
      ];
      for (const p of paths) if (this.testCmd(p)) return p;
    }

    // 3. Try PATH
    try {
      execSync("moonpub --help", { stdio: "ignore" });
      return "moonpub";
    } catch {}

    return "";
  }

  private testCmd(path: string): boolean {
    try {
      execSync(`"${path}" --help`, { stdio: "ignore" });
      return true;
    } catch {
      return false;
    }
  }

  private checkRequirements(): boolean {
    if (!this.moonpubPath || !this.testCmd(this.moonpubPath)) {
      new Notice("❌ MoonPub 未安装。请先安装 moonpub：https://github.com/qiaopengjun5162/moonpub", 0);
      return false;
    }
    if (!process.env.WECHAT_APPID || !process.env.WECHAT_SECRET) {
      new Notice("❌ 请设置环境变量 WECHAT_APPID 和 WECHAT_SECRET", 0);
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
    return (this.app.vault.adapter as any).getFullPath(file.path);
  }

  private runCmd(subcmd: string) {
    if (!this.checkRequirements()) return;
    const filePath = this.getActiveFilePath();
    if (!filePath) return;

    const cmd = `"${this.moonpubPath}" ${subcmd} "${filePath}"`;
    const notice = new Notice(`🚀 ${subcmd}...`, 0);

    exec(cmd, { env: process.env, timeout: 300_000 }, (err, stdout, stderr) => {
      notice.hide();
      if (err) {
        const msg = stderr || err.message || "未知错误";
        new Notice(`❌ ${msg.slice(0, 80)}`);
        console.error("moonpub error:", msg);
      } else {
        new Notice("✅ 完成！去微信草稿箱检查吧");
        console.log("moonpub:", stdout);
      }
    });
  }

  private runShip() {
    this.runCmd("ship");
  }

  private runShipAi() {
    this.runCmd("ship --ai");
  }

  onunload() {}
}
