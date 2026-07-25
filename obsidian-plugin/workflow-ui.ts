export type ActiveContextKind = "markdown" | "photo" | "other" | "none";

export interface ModalLifecycle {
  close(): void;
  open(): void;
}

export function contextKindLabel(kind: ActiveContextKind): string {
  switch (kind) {
    case "markdown":
      return "当前打开的是 Markdown 文章";
    case "photo":
      return "当前打开的是图片";
    case "other":
      return "当前打开的是其他文件";
    case "none":
      return "当前没有打开文件";
  }
}

export function firstRunSteps(kind: ActiveContextKind): string[] {
  switch (kind) {
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
      return [
        "先从飞书素材入口开始，或先打开一篇 Markdown / 一张图片",
        "优先把第一次体验目标定在草稿和本地预览，而不是立即推微信",
        "看懂首页、结果页和下一步动作后，再进入真实发布链路",
      ];
  }
}

export function workspacePathLabel(path: string): string {
  const markers = ["/Inbox/", "/Articles/"];
  for (const marker of markers) {
    const index = path.indexOf(marker);
    if (index >= 0) return path.slice(index + 1);
  }
  return path;
}

export function replaceModal<T extends ModalLifecycle>(current: T | null, next: T): T {
  current?.close();
  next.open();
  return next;
}

export function needsPreviewRecipientPrompt(savedRecipient: string): boolean {
  return savedRecipient.trim().length === 0;
}

export function previewRecipientEnv(savedRecipient: string): Record<string, string> {
  const trimmed = savedRecipient.trim();
  return trimmed ? { WECHAT_PREVIEW_TO: trimmed } : {};
}

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
