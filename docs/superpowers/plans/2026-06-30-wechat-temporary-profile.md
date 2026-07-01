# WeChat Temporary Profile Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 MoonPub 的微信公众号浏览器自动化增加显式 `--temporary-profile` 隔离模式，同时保留默认稳定的持久 profile 链路。

**Architecture:** 在 CLI 层为浏览器自动化命令新增 `--temporary-profile`，在 CDP 层引入 profile 模式与带清理职责的浏览器会话对象，令临时模式只使用一次性 profile 且不读写持久 session。`publish.rs` 仅负责把模式继续传递给现有自动化流程。

**Tech Stack:** Rust, chromiumoxide, cargo nextest

---

### Task 1: 写失败测试，锁定 CLI 行为

**Files:**
- Modify: `src/cli.rs`

- [ ] **Step 1: 为浏览器命令补 CLI 解析测试**

```rust
    #[test]
    fn parses_login_with_temporary_profile() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "login".to_owned(),
            "--temporary-profile".to_owned(),
        ])?;
        assert_eq!(
            options.command,
            Command::Login {
                temporary_profile: true,
            }
        );
        Ok(())
    }
```

- [ ] **Step 2: 跑 CLI 定向测试，确认先失败**

Run: `cargo nextest run --all-features parses_login_with_temporary_profile`

Expected: FAIL，提示 `Command::Login` 结构不匹配或不存在 `temporary_profile`

- [ ] **Step 3: 为 configure / test-yulan 增补同类失败测试**

```rust
    #[test]
    fn parses_configure_with_temporary_profile() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "configure".to_owned(),
            "--temporary-profile".to_owned(),
            "--headed".to_owned(),
        ])?;
        assert_eq!(
            options.command,
            Command::Configure {
                steps: vec![],
                headed: true,
                temporary_profile: true,
            }
        );
        Ok(())
    }
```

- [ ] **Step 4: 再跑定向测试，确认仍按预期失败**

Run: `cargo nextest run --all-features temporary_profile`

Expected: FAIL，因为 CLI 和命令结构尚未实现

### Task 2: 写失败测试，锁定 CDP profile 语义

**Files:**
- Modify: `src/cdp.rs`

- [ ] **Step 1: 为 profile/session 选择逻辑补失败测试**

```rust
    #[test]
    fn persistent_profile_uses_config_directory() {
        let session = BrowserProfileMode::Persistent;
        assert!(profile_dir_for(&session).to_string_lossy().contains(".config/moonpub"));
        assert!(session_file_for(&session).is_some());
    }
```

- [ ] **Step 2: 为 temporary 模式补失败测试**

```rust
    #[test]
    fn temporary_profile_uses_temp_directory_and_no_session_file() {
        let mode = BrowserProfileMode::temporary();
        let profile = profile_dir_for(&mode);
        assert!(profile.starts_with(std::env::temp_dir()));
        assert!(session_file_for(&mode).is_none());
    }
```

- [ ] **Step 3: 跑 CDP 定向测试，确认先失败**

Run: `cargo nextest run --all-features temporary_profile_uses_temp_directory_and_no_session_file`

Expected: FAIL，因为 mode/helper 还不存在

### Task 3: 实现最小 CLI 改动并让测试转绿

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/app.rs`
- Modify: `src/error.rs`

- [ ] **Step 1: 给浏览器命令增加 `temporary_profile` 字段并解析 flag**

```rust
    Login {
        temporary_profile: bool,
    },
```

- [ ] **Step 2: 在 app 路由中把 flag 传给 publish 层**

```rust
        Command::Login { temporary_profile } => {
            crate::publish::login(*temporary_profile)
```

- [ ] **Step 3: 更新 help text**

```text
moonpub ... login [--temporary-profile]
moonpub ... configure [<step>..] [--headed] [--temporary-profile]
```

- [ ] **Step 4: 跑 CLI 定向测试，确认通过**

Run: `cargo nextest run --all-features temporary_profile`

Expected: PASS，CLI 相关测试通过

### Task 4: 实现 CDP profile mode 与会话对象

**Files:**
- Modify: `src/cdp.rs`

- [ ] **Step 1: 引入 profile mode 和临时 profile guard**

```rust
pub enum BrowserProfileMode {
    Persistent,
    Temporary { dir: PathBuf },
}
```

- [ ] **Step 2: 用会话对象替代裸元组返回**

```rust
pub struct BrowserSession {
    pub browser: Browser,
    pub page: Page,
    _temporary_profile: Option<TemporaryProfileGuard>,
}
```

- [ ] **Step 3: 让 save/restore 在 temporary 模式下跳过 session**

```rust
if let Some(path) = session_file_for(mode) {
    std::fs::write(path, json).ok();
}
```

- [ ] **Step 4: 跑 CDP 定向测试，确认通过**

Run: `cargo nextest run --all-features profile_`

Expected: PASS，新增 profile 语义测试通过

### Task 5: 接入 publish 自动化链路

**Files:**
- Modify: `src/publish.rs`

- [ ] **Step 1: 让 login/configure/test 系列函数接收 `temporary_profile: bool`**

```rust
pub fn login(temporary_profile: bool) -> Result<String, String> {
```

- [ ] **Step 2: 在入口处构造 mode 并传给 `open_browser` / `setup_editor`**

```rust
let mode = BrowserProfileMode::from_temporary_flag(temporary_profile);
let session = open_browser(!headed, &mode).await?;
```

- [ ] **Step 3: 确保临时模式仍保活 Browser 直到登录完成**

```rust
with_retained_resource(session.browser, |browser| { ... })
```

- [ ] **Step 4: 跑浏览器相关单元测试**

Run: `cargo nextest run --all-features cdp::tests:: cli::tests::`

Expected: PASS

### Task 6: 同步文档与项目约束

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `docs/USER_GUIDE.md`
- Modify: `PROGRESS.md`
- Modify: `AGENTS.md`

- [ ] **Step 1: 在 README / 用户指南里加入 `--temporary-profile` 用法**

```text
moonpub configure --temporary-profile --headed
```

- [ ] **Step 2: 在 AGENTS / PROGRESS 里记录新的非显而易见约束**

```text
`--temporary-profile` 只用于浏览器自动化链路，且不会复用持久 session。
```

- [ ] **Step 3: 检查文档口径一致**

Run: `rg -n "temporary-profile|临时 profile|temporary profile" README.md README_zh.md docs/USER_GUIDE.md PROGRESS.md AGENTS.md`

Expected: 每份相关文档都有同步说明

### Task 7: 全量验证

**Files:**
- Modify: none

- [ ] **Step 1: 格式检查**

Run: `cargo fmt --all -- --check`

Expected: PASS

- [ ] **Step 2: Clippy**

Run: `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`

Expected: PASS

- [ ] **Step 3: 全量测试**

Run: `cargo nextest run --all-features`

Expected: PASS
