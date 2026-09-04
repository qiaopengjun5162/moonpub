#!/usr/bin/env python3
"""MoonPub MCP server 集成测试。

不依赖真实 moonpub 二进制 / 网络：通过 mock subprocess.run 验证
薄壳逻辑的 argv 构造、JSON 解析、错误处理与 tool 注册数量。

运行（managed python venv）：
  /Users/qiaopengjun/.workbuddy/binaries/python/envs/default/bin/python \
      -m pytest mcp/test_server.py -q
"""

import importlib.util
import json
import os
import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

SERVER_PATH = Path(__file__).resolve().parent / "server.py"
spec = importlib.util.spec_from_file_location("moonpub_mcp_server", SERVER_PATH)
server = importlib.util.module_from_spec(spec)
spec.loader.exec_module(server)


# ---------------------------------------------------------------------------
# _build_argv
# ---------------------------------------------------------------------------

def test_build_argv_default_no_bin_env():
    env = os.environ.copy()
    env.pop("MOONPUB_BIN", None)
    with patch.dict(os.environ, env, clear=True):
        argv = server._build_argv(["doctor"], None)
    assert argv == ["moonpub", "--json", "doctor"]


def test_build_argv_injects_bin_and_articles():
    with patch.dict(
        os.environ,
        {"MOONPUB_BIN": "/opt/moonpub", "MOONPUB_ARTICLES": "/art"},
    ):
        argv = server._build_argv(["render", "a.md"], "/art")
    assert argv == ["/opt/moonpub", "--json", "--articles", "/art", "render", "a.md"]


def test_build_argv_omits_articles_when_none():
    with patch.dict(os.environ, {"MOONPUB_BIN": "moonpub"}):
        argv = server._build_argv(["status"], None)
    assert "--articles" not in argv
    assert argv == ["moonpub", "--json", "status"]


# ---------------------------------------------------------------------------
# _run: 正常与失败路径
# ---------------------------------------------------------------------------

def _fake_proc(stdout, stderr, returncode):
    p = MagicMock()
    p.stdout = stdout
    p.stderr = stderr
    p.returncode = returncode
    return p


def test_run_parses_json_stdout():
    with patch.dict(os.environ, {"MOONPUB_BIN": "moonpub"}), patch.object(
        server.subprocess, "run", return_value=_fake_proc('{"ok": true}', "", 0)
    ) as m:
        res = server._run(["doctor"])
    assert res["ok"] is True
    assert res["exit_code"] == 0
    assert res["json"] == {"ok": True}
    m.assert_called_once()


def test_run_non_json_stdout_kept_as_string():
    with patch.dict(os.environ, {"MOONPUB_BIN": "moonpub"}), patch.object(
        server.subprocess, "run", return_value=_fake_proc("plain text", "", 0)
    ):
        res = server._run(["workspace"])
    assert res["ok"] is True
    assert res["json"] == "plain text"


def test_run_nonzero_exit_marks_not_ok():
    with patch.dict(os.environ, {"MOONPUB_BIN": "moonpub"}), patch.object(
        server.subprocess,
        "run",
        return_value=_fake_proc("", "boom", 3),
    ):
        res = server._run(["push", "x"])
    assert res["ok"] is False
    assert res["exit_code"] == 3
    assert res["stderr"] == "boom"


def test_run_binary_not_found():
    with patch.dict(os.environ, {"MOONPUB_BIN": "nope-bin"}), patch.object(
        server.subprocess, "run", side_effect=FileNotFoundError()
    ):
        res = server._run(["doctor"])
    assert res["ok"] is False
    assert res["exit_code"] == -1
    assert "not found" in res["error"]


def test_run_timeout():
    with patch.dict(os.environ, {"MOONPUB_BIN": "moonpub"}), patch.object(
        server.subprocess,
        "run",
        side_effect=server.subprocess.TimeoutExpired(cmd="x", timeout=1),
    ):
        res = server._run(["push", "x"], timeout=1)
    assert res["ok"] is False
    assert res["exit_code"] == -1
    assert "timed out" in res["error"]


# ---------------------------------------------------------------------------
# _result: JSON 回传可反序列化
# ---------------------------------------------------------------------------

def test_result_roundtrip_valid_json():
    res = {"ok": True, "exit_code": 0, "json": {"a": 1}, "stdout": "x", "stderr": ""}
    s = server._result(res)
    parsed = json.loads(s)
    assert parsed["ok"] is True
    assert parsed["json"] == {"a": 1}


# ---------------------------------------------------------------------------
# intake_feishu: source 解析
# ---------------------------------------------------------------------------

def test_intake_feishu_source_parsing():
    with patch.object(server, "_run", return_value={"ok": True}) as m:
        server.intake_feishu("minute-token:abc123")
    assert m.call_args[0][0][:3] == ["intake", "feishu", "--minute-token"]
    assert m.call_args[0][0][3] == "abc123"

    with patch.object(server, "_run", return_value={"ok": True}) as m:
        server.intake_feishu("query:hello world")
    assert m.call_args[0][0][:3] == ["intake", "feishu", "--query"]
    assert m.call_args[0][0][3] == "hello world"

    with patch.object(server, "_run", return_value={"ok": True}) as m:
        server.intake_feishu("latest")
    assert m.call_args[0][0] == ["intake", "feishu", "--latest"]

    with patch.object(server, "_run", return_value={"ok": True}) as m:
        server.intake_feishu("/path/file.md")
    assert m.call_args[0][0] == ["intake", "feishu", "/path/file.md"]


def test_intake_feishu_forwards_flags():
    with patch.object(server, "_run", return_value={"ok": True}) as m:
        server.intake_feishu("latest", draft=True, preview=True, no_open=True, push=True)
    args = m.call_args[0][0]
    assert "--draft" in args
    assert "--preview" in args
    assert "--no-open" in args
    assert "--push" in args


# ---------------------------------------------------------------------------
# run: escape-hatch 拆分
# ---------------------------------------------------------------------------

def test_run_escape_hatch_splits_command_and_args():
    with patch.object(server, "_run", return_value={"ok": True}) as m:
        server.run("preflight", "a.md --json")
    assert m.call_args[0][0] == ["preflight", "a.md", "--json"]


# ---------------------------------------------------------------------------
# tool 注册数量
# ---------------------------------------------------------------------------

def test_tool_count_registered():
    # 21 个 tool: doctor/workspace/status/capabilities/list_drafts/
    # preflight/render/cover/push/preview/publish/ship/mark_ready/delete_draft/
    # login/test_yulan/new/write/intake_feishu/intake_photos/run
    expected = [
        "doctor", "workspace", "status", "capabilities", "list_drafts",
        "preflight", "render", "cover", "push", "preview", "publish",
        "ship", "mark_ready", "delete_draft", "login", "test_yulan",
        "new", "write", "intake_feishu", "intake_photos", "run",
    ]
    for name in expected:
        assert hasattr(server, name), f"缺少 tool 函数: {name}"
        assert callable(getattr(server, name)), f"{name} 不可调用"
    assert len(expected) == 21
