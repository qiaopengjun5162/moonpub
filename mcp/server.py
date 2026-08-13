#!/usr/bin/env python3
"""MoonPub MCP server — 把 moonpub CLI 子命令暴露为 MCP tools。

薄壳设计：每个 tool 拼出 argv → 调用 `moonpub` 二进制 → 把 stdout 作为结构化
结果返回。所有调用统一在全局位追加 `--json`（见 src/app.rs:453，非结构化命令
也会被包成 JSON 字符串），因此 tool 拿到的始终是合法 JSON，处理统一。

环境变量：
  MOONPUB_BIN      moonpub 可执行文件路径，默认 "moonpub"（需已在 PATH）
  MOONPUB_ARTICLES 默认 articles 根目录，会作为 --articles 传入；不设置则依赖
                   moonpub 自身的配置自动发现（从文章路径向上查找 moonpub.toml）
  MOONPUB_TIMEOUT  单条命令超时秒数，默认 300

运行：python3 server.py  （stdio 传输，供 MCP client 连接）
"""

import json
import os
import subprocess
from typing import Optional

from fastmcp import FastMCP

mcp = FastMCP("moonpub")

DEFAULT_TIMEOUT = int(os.environ.get("MOONPUB_TIMEOUT", "300"))


def _build_argv(args, articles: Optional[str]) -> list[str]:
    bin_path = os.environ.get("MOONPUB_BIN", "moonpub")
    argv = [bin_path, "--json"]
    if articles:
        argv += ["--articles", articles]
    argv += list(args)
    return argv


def _run(args, articles: Optional[str] = None, cwd: Optional[str] = None,
         timeout: int = DEFAULT_TIMEOUT) -> dict:
    argv = _build_argv(args, articles)
    try:
        proc = subprocess.run(
            argv, cwd=cwd, capture_output=True, text=True, timeout=timeout
        )
    except FileNotFoundError:
        return {
            "ok": False,
            "exit_code": -1,
            "error": f"moonpub binary not found: {argv[0]}",
            "stdout": "",
            "stderr": "",
        }
    except subprocess.TimeoutExpired:
        return {
            "ok": False,
            "exit_code": -1,
            "error": f"command timed out after {timeout}s: {' '.join(argv)}",
            "stdout": "",
            "stderr": "",
        }

    out = (proc.stdout or "").strip()
    try:
        data = json.loads(out) if out else None
    except json.JSONDecodeError:
        data = out
    return {
        "ok": proc.returncode == 0,
        "exit_code": proc.returncode,
        "json": data,
        "stdout": out,
        "stderr": (proc.stderr or "").strip(),
    }


def _result(res: dict) -> str:
    return json.dumps(res, ensure_ascii=False, indent=2)


# ---------------------------------------------------------------------------
# 诊断 / 状态类
# ---------------------------------------------------------------------------

@mcp.tool()
def doctor(articles: Optional[str] = None) -> str:
    """本地可用性诊断：检查 moonpub.toml、Articles 目录、微信/封面依赖等是否就绪。返回结构化报告。"""
    return _result(_run(["doctor"], articles=articles))


@mcp.tool()
def workspace(articles: Optional[str] = None) -> str:
    """展示工作区状态：目录结构、文章计数、各阶段（drafts/ready/published）概览。返回结构化 JSON。"""
    return _result(_run(["workspace"], articles=articles))


@mcp.tool()
def status(articles: Optional[str] = None) -> str:
    """展示当前文章集合的发布状态摘要。返回结构化 JSON。"""
    return _result(_run(["status"], articles=articles))


@mcp.tool()
def capabilities(articles: Optional[str] = None) -> str:
    """返回 moonpub 的能力元数据（命令、协议能力声明）。插件或 agent 用它判断可用功能。"""
    return _result(_run(["capabilities"], articles=articles))


@mcp.tool()
def list_drafts(articles: Optional[str] = None) -> str:
    """列出微信公众号草稿箱中的草稿（media_id 与标题）。返回文本。"""
    return _result(_run(["list-drafts"], articles=articles))


# ---------------------------------------------------------------------------
# 发布闭环
# ---------------------------------------------------------------------------

@mcp.tool()
def preflight(article: str, articles: Optional[str] = None) -> str:
    """发布前质量门：校验文章包三件套、HTML 排版审计、标题/摘要长度、图片完整性等。返回结构化检查结果。"""
    return _result(_run(["preflight", article], articles=articles))


@mcp.tool()
def render(article: str, author: Optional[str] = None, humanize: bool = False,
           articles: Optional[str] = None) -> str:
    """把 Markdown 文章渲染为微信公众号 HTML。可选 --author 指定作者，--humanize 做口语化润色。返回渲染结果文本。"""
    args = ["render", article]
    if author:
        args += ["--author", author]
    if humanize:
        args.append("--humanize")
    return _result(_run(args, articles=articles))


@mcp.tool()
def cover(article: str, style: Optional[str] = None, screenshot: bool = False,
          articles: Optional[str] = None) -> str:
    """生成文章封面。--style 指定封面风格（如 clean/minimal/geek-black）；--screenshot 才会真正产出 PNG 并写回 frontmatter 的 cover 字段。返回文本。"""
    args = ["cover", article]
    if style:
        args += ["--style", style]
    if screenshot:
        args.append("--screenshot")
    return _result(_run(args, articles=articles))


@mcp.tool()
def push(article: str, auto_render: bool = False, articles: Optional[str] = None) -> str:
    """推送文章到微信公众号草稿箱。--render 会在推送前自动重新渲染。返回结构化结果（含 media_id）。注意：封面必须已用 cover --screenshot 写回 frontmatter，否则报 40007。"""
    args = ["push", article]
    if auto_render:
        args.append("--render")
    return _result(_run(args, articles=articles))


@mcp.tool()
def preview(article: str, no_open: bool = False, articles: Optional[str] = None) -> str:
    """本地 HTML 预览。--no-open 只生成预览文件不打开浏览器。返回结构化结果（含预览文件路径）。"""
    args = ["preview", article]
    if no_open:
        args.append("--no-open")
    return _result(_run(args, articles=articles))


@mcp.tool()
def publish(article: str, target: str = "wechat-draft", auto_render: bool = False,
            articles: Optional[str] = None) -> str:
    """发布文章到指定 target（如 wechat-draft）。--render 推送前自动重渲染。返回文本。"""
    args = ["publish", article, "--target", target]
    if auto_render:
        args.append("--render")
    return _result(_run(args, articles=articles))


@mcp.tool()
def ship(article: str, style: Optional[str] = None, ai: bool = False,
         articles: Optional[str] = None) -> str:
    """一键发布预览：渲染+推送+（可选 AI 润色）的完整闭环。--style 指定排版主题，--ai 启用 AI 流程。返回文本。"""
    args = ["ship", article]
    if style:
        args += ["--style", style]
    if ai:
        args.append("--ai")
    return _result(_run(args, articles=articles))


@mcp.tool()
def mark_ready(article: str, articles: Optional[str] = None) -> str:
    """将文章从 drafts 标记为 ready（进入可发布队列）。返回文本。"""
    return _result(_run(["mark-ready", article], articles=articles))


@mcp.tool()
def delete_draft(media_id: str, articles: Optional[str] = None) -> str:
    """按 media_id 删除微信公众号草稿箱中的某条草稿。返回文本。"""
    return _result(_run(["delete-draft", media_id], articles=articles))


# ---------------------------------------------------------------------------
# 微信浏览器自动化
# ---------------------------------------------------------------------------

@mcp.tool()
def login(articles: Optional[str] = None) -> str:
    """打开微信网页版登录流程（需扫码）。登录态用于后续 push/test-yulan。返回文本。"""
    return _result(_run(["login"], articles=articles))


@mcp.tool()
def test_yulan(title: Optional[str] = None, to: Optional[str] = None,
               articles: Optional[str] = None) -> str:
    """手机预览：把最新/指定草稿发到手机微信预览。--title 指定标题，--to 指定接收的微信号(wxid)。需先 login。返回文本。"""
    args = ["test-yulan"]
    if title:
        args += ["--title", title]
    if to:
        args += ["--to", to]
    return _result(_run(args, articles=articles))


# ---------------------------------------------------------------------------
# 创作 / 草稿
# ---------------------------------------------------------------------------

@mcp.tool()
def new(title: str, articles: Optional[str] = None) -> str:
    """基于标题新建一篇空白文章草稿（含 frontmatter 模板）。返回文本。"""
    return _result(_run(["new", title], articles=articles))


@mcp.tool()
def write(idea: str, articles: Optional[str] = None) -> str:
    """根据一个想法(idea)用 AI 生成文章初稿。返回文本。"""
    return _result(_run(["write", idea], articles=articles))


# ---------------------------------------------------------------------------
# 素材 intake
# ---------------------------------------------------------------------------

@mcp.tool()
def intake_feishu(source: str, draft: bool = False, preview: bool = False,
                  no_open: bool = False, push: bool = False,
                  articles: Optional[str] = None) -> str:
    """飞书素材进入 Inbox 并可选生成草稿。source 取值：文件路径 | "latest" | "minute-token:<token>" | "query:<关键词>"。--draft 生成草稿，--preview 本地预览，--no-open 不打开浏览器，--push 直接推进到微信草稿。返回结构化结果。"""
    if source.startswith("minute-token:"):
        src_args = ["--minute-token", source.split(":", 1)[1]]
    elif source.startswith("query:"):
        src_args = ["--query", source.split(":", 1)[1]]
    elif source == "latest":
        src_args = ["--latest"]
    else:
        src_args = [source]
    args = ["intake", "feishu", *src_args]
    if draft:
        args.append("--draft")
    if preview:
        args.append("--preview")
    if no_open:
        args.append("--no-open")
    if push:
        args.append("--push")
    return _result(_run(args, articles=articles))


@mcp.tool()
def intake_photos(inputs: list[str], draft: bool = False, preview: bool = False,
                  no_open: bool = False, push: bool = False,
                  analyze_images: bool = False, articles: Optional[str] = None) -> str:
    """照片素材进入 Inbox/Photos 并可选生成草稿。inputs 为照片文件或目录列表。--analyze-images 启用图像分析（仅 OpenAI，需人工核对）。其余 flag 同 intake_feishu。返回结构化结果。"""
    args = ["intake", "photos", *inputs]
    if analyze_images:
        args.append("--analyze-images")
    if draft:
        args.append("--draft")
    if preview:
        args.append("--preview")
    if no_open:
        args.append("--no-open")
    if push:
        args.append("--push")
    return _result(_run(args, articles=articles))


# ---------------------------------------------------------------------------
# 通用 escape hatch
# ---------------------------------------------------------------------------

@mcp.tool()
def run(command: str, args: str = "", articles: Optional[str] = None) -> str:
    """通用执行任意 moonpub 子命令。command 为子命令名（如 preflight/radar/export），args 为空格分隔的剩余参数（按空格拆分后拼接）。用于覆盖上述未单独封装的命令。"""
    parts = command.split()
    if args:
        parts += args.split()
    return _result(_run(parts, articles=articles))


if __name__ == "__main__":
    mcp.run(transport="stdio")
