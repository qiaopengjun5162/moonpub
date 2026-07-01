# WeChat Temporary Profile Design

## Goal

为 MoonPub 的微信公众号浏览器自动化增加一个显式开启的“临时隔离 profile”模式，在不影响默认稳定链路的前提下，提供更接近一次性/隔离浏览器环境的登录与预览路径。

## Scope

本次只覆盖依赖 Chrome/CDP 的微信公众号后台命令：

- `moonpub login`
- `moonpub configure`
- `moonpub step-test`
- `moonpub test-zanshang`
- `moonpub test-chuangzuo`
- `moonpub test-yulan`

不修改微信 API 草稿推送逻辑，不修改 `push` / `ship` 的 API 部分，不改变“创作来源 = 个人观点，仅供参考”的现有行为。

## User-Facing Behavior

默认行为保持不变：

- 继续使用持久 profile：`~/.config/moonpub/chrome-profile`
- 继续复用持久 session：`~/.config/moonpub/session.json`

新增显式 flag：

- `--temporary-profile`

当用户显式传入 `--temporary-profile` 时：

- 使用独立的临时浏览器 profile 目录
- 不读取持久 `session.json`
- 不写入持久 `session.json`
- 不污染默认稳定 profile
- 通常需要重新扫码登录

## Approach Options

### Option A: 默认持久 profile + 显式临时 profile

优点：

- 最符合当前用户诉求
- 不破坏现有稳定工作流
- CLI 语义清晰，风险最小

缺点：

- 临时模式下扫码频率更高
- 需要在多个浏览器命令中显式传递 mode

### Option B: 全局配置切换默认 profile 模式

优点：

- 可集中控制默认行为

缺点：

- 容易误切换，影响已有稳定链路
- 会让“这次只是想临时隔离一下”的场景变重

### Option C: 直接把现有 profile 改成一次性

优点：

- 隔离最强

缺点：

- 直接破坏当前已稳定的复用登录态体验
- 与当前产品方向相反

结论：采用 Option A。

## Design

### CLI

给上述 6 个浏览器自动化命令新增 `--temporary-profile`。

示例：

```bash
moonpub login --temporary-profile
moonpub configure --temporary-profile --headed
moonpub test-yulan --temporary-profile --headed
```

### Runtime Model

新增一个浏览器 profile 模式枚举：

- `Persistent`
- `Temporary`

CDP 层根据模式决定：

- profile 目录路径
- session 文件路径是否可用
- 是否执行 session restore/save

### Temporary Profile Semantics

临时模式应当“隔离优先”，而不是“尽量复用现有状态”：

- profile 目录位于系统临时目录下的 MoonPub 专用目录
- 每次运行使用唯一子目录
- 运行结束后尽力清理该目录
- session restore/save 在临时模式下直接跳过

这保证临时模式不会复用之前的持久浏览器状态，也不会把这次扫码得到的状态写回默认持久链路。

### Browser Lifetime and Cleanup

浏览器启动结果不再只返回裸 `(Browser, Page)`，而是返回带 profile 生命周期的会话对象。

原因：

- 临时 profile 目录需要在浏览器运行期间保持存在
- 浏览器关闭后需要尽力清理临时目录
- 这类资源生命周期应由 CDP 层统一管理，不应散落在 `publish.rs`

### Error Handling

- 临时模式与默认模式共享原有登录/编辑器/预览错误语义
- 临时 profile 目录清理失败只记录，不中断主流程
- session restore/save 在临时模式下属于预期跳过，不应报错

## Testing

需要补三类测试：

1. CLI 解析测试
   - `login --temporary-profile`
   - `configure --temporary-profile --headed`
   - `test-yulan --temporary-profile`

2. CDP 路径选择测试
   - 持久模式返回固定 profile/session 路径
   - 临时模式 profile 路径唯一且位于临时目录
   - 临时模式没有 session 文件路径

3. 生命周期测试
   - 临时 profile guard drop 后会尝试清理目录

## Docs Impact

同步更新：

- `README.md`
- `README_zh.md`
- `docs/USER_GUIDE.md`
- `PROGRESS.md`
- `AGENTS.md`

文档中要明确：

- 默认仍是稳定持久 profile
- `--temporary-profile` 是显式隔离模式
- 临时模式通常需要重新扫码
