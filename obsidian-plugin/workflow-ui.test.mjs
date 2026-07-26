import assert from "node:assert/strict";
import { after, test } from "node:test";
import { build } from "esbuild";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const outputDir = await mkdtemp(join(tmpdir(), "moonpub-plugin-tests-"));
const outputFile = join(outputDir, "workflow-ui.mjs");

await build({
  entryPoints: ["workflow-ui.ts"],
  bundle: true,
  format: "esm",
  outfile: outputFile,
  platform: "node",
  target: "node20",
});

const ui = await import(pathToFileURL(outputFile).href);

after(async () => {
  await rm(outputDir, { force: true, recursive: true });
});

test("homepage context labels and first-run guidance stay aligned", () => {
  assert.equal(ui.contextKindLabel("markdown"), "当前打开的是 Markdown 文章");
  assert.match(ui.firstRunSteps("markdown")[0], /检查当前文章/);

  assert.equal(ui.contextKindLabel("photo"), "当前打开的是图片");
  assert.match(ui.firstRunSteps("photo")[0], /导入当前图片目录/);

  assert.equal(ui.contextKindLabel("none"), "当前没有打开文件");
  assert.match(ui.firstRunSteps("none")[0], /飞书素材入口/);
});

test("workspace paths hide private prefixes when a known workspace segment exists", () => {
  assert.equal(ui.workspacePathLabel("/private/vault/Inbox/Feishu/note.md"), "Inbox/Feishu/note.md");
  assert.equal(ui.workspacePathLabel("/private/vault/Articles/drafts/article.md"), "Articles/drafts/article.md");
  assert.equal(ui.workspacePathLabel("relative/article.md"), "relative/article.md");
});

test("replacing the homepage closes the old modal before opening the new one", () => {
  const events = [];
  const oldModal = { close: () => events.push("old:close"), open: () => events.push("old:open") };
  const newModal = { close: () => events.push("new:close"), open: () => events.push("new:open") };

  assert.equal(ui.replaceModal(oldModal, newModal), newModal);
  assert.deepEqual(events, ["old:close", "new:open"]);
});

test("preview recipient prompt only appears when no recipient is saved", () => {
  assert.equal(ui.needsPreviewRecipientPrompt(""), true);
  assert.equal(ui.needsPreviewRecipientPrompt("   "), true);
  assert.equal(ui.needsPreviewRecipientPrompt("my-wxid"), false);
});

test("preview recipient env injects WECHAT_PREVIEW_TO only when saved", () => {
  assert.deepEqual(ui.previewRecipientEnv(""), {});
  assert.deepEqual(ui.previewRecipientEnv("  my-wxid  "), { WECHAT_PREVIEW_TO: "my-wxid" });
});

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

test("preview_to persistence is best-effort when root is empty", async () => {
  await assert.doesNotReject(async () => {
    await ui.persistPreviewTo("", "my-wxid");
  });
});
