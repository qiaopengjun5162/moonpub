# Radar Structure Cleanup Design

## Goal

把 `src/radar.rs` 从“命令入口 + CSV 导入 + 标题建议实现细节”的混合文件，收回为一个薄的路由与导出层；把已经自然成团的职责继续下沉到 `src/radar/` 子模块中，同时保持 CLI 行为、输出文案和测试结果不变。

## Why Now

`radar.rs` 目前仍有 800+ 行，虽然已经拆出了 `cli`、`store`、`analyze`、`scrape`，但 `CSV import` 与 `title suggestion` 还留在壳文件里，导致：

- `radar.rs` 同时承担类型定义、命令分发、CSV 解析、文章标题建议等多种职责。
- 后续继续扩展 Radar 时，维护者仍需要先读一个偏大的聚合文件。
- 现有子模块边界已经形成，再拖下去只会让“半拆分”状态固化。

这次只做结构清理，不顺手扩功能，也不改变用户看到的行为。

## Scope

### In Scope

- 把 `import_csv` 和 `parse_csv_row` 从 `src/radar.rs` 下沉到新的 `src/radar/import.rs`
- 把 `suggest_titles` 及其只服务于标题建议的辅助逻辑，从 `src/radar.rs` 下沉到新的 `src/radar/suggest.rs`
- 让 `src/radar.rs` 保留：
  - `RadarCommand` 定义
  - `run_radar` 命令分发
  - `mod` / `pub use` / `pub(crate) use` 聚合导出
- 保持现有调用方、CLI 参数、输出文案、错误行为和测试覆盖不变
- 在 `PROGRESS.md` 记录本次真实拆分和验证结果

### Out of Scope

- 不修改 `radar` CLI 语法
- 不改变 CSV 支持的列名或导入规则
- 不修改标题建议算法、评分逻辑或输出模板
- 不顺手拆 `cli.rs`、`markdown.rs` 或其他模块
- 不新增新功能或新命令

## Target File Structure

本次完成后，Radar 目录结构应变为：

```text
src/
  radar.rs
  radar/
    analyze.rs
    cli.rs
    import.rs
    scrape.rs
    store.rs
    suggest.rs
```

各文件职责如下：

- `src/radar.rs`
  - 只放 `RadarCommand`
  - 只放 `run_radar`
  - 只做子模块装配与必要 re-export
- `src/radar/import.rs`
  - 负责 CSV 文件读取
  - 负责列头识别与单行解析
  - 负责把 CSV 行转换成 `TrendSample` 并写入 store
- `src/radar/suggest.rs`
  - 负责标题建议主流程
  - 负责只被标题建议使用的辅助函数
- `src/radar/analyze.rs`
  - 保持文章分析职责不变
- `src/radar/store.rs`
  - 保持趋势样本存储职责不变

## Design Decisions

### 1. 保留 `RadarCommand` 在 `src/radar.rs`

`RadarCommand` 是 Radar 的公共入口类型，CLI 和 app 层都会消费它。把它继续留在 `src/radar.rs`，能让外部调用保持稳定，也让 `radar.rs` 继续作为 Radar 子系统的总入口。

### 2. `import_csv` 单独成模块

CSV 导入已经形成明确职责链：读文件、识别列、解析行、构造 `TrendSample`、调用 `add_trend_sample`。它与 `run_radar` 没有必要耦合，单独拆到 `import.rs` 后：

- `radar.rs` 会明显变薄
- CSV 规则更容易单独测试
- 后续若扩展 TSV 或更强导入逻辑，也有自然落点

### 3. `suggest_titles` 单独成模块

标题建议逻辑本身已经像一个小用例：读文章、取 frontmatter、做 token 分析、读取趋势样本、套标题模板。它与 `analyze.rs` 有关联，但输出目标不同：

- `analyze.rs` 更像“分析解释”
- `suggest.rs` 更像“生成建议”

因此不把它塞进 `analyze.rs`，避免再次形成大而杂的分析模块。

### 4. 不追求一次性把 Radar 拆到极致

这次不继续拆 `RadarCommand`、`run_radar`，也不动 `scrape` / `store` / `analyze` 的接口。目标是把最明显的混合职责移走，而不是为了“更纯粹”引入额外重构风险。

## Behavior Invariants

本次重构后，以下行为必须保持不变：

- `moonpub radar import ...` 的 CLI 参数与输出文案不变
- CSV 识别列名集合不变
- 空 CSV、缺失必要列、I/O 失败时的错误行为不变
- `moonpub radar suggest ...` 的输出结构、标题模板和趋势引用逻辑不变
- `moonpub radar analyze ...`、`add`、`list`、`scrape` 行为不受影响

如果实现过程中发现必须改用户可见行为，需先停下来单独确认，而不是顺手带进去。

## Testing Strategy

本次是结构重构，验证重点是“行为没有变”：

- 跑 `cargo fmt --all -- --check`
- 跑 `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`
- 跑 `cargo nextest run --all-features radar::`

若现有 Radar 测试不足以覆盖新边界，可补最小必要测试，但只允许补“守住现有行为”的测试，不允许借机扩展功能。

## Documentation Impact

这次不需要改 README / README_zh 的用户文档，因为用户可见命令不变。

需要更新：

- `PROGRESS.md`
  - 记录 `radar import` / `suggest` 模块拆分已完成
  - 记录本次实际跑过的验证命令

若实现中发现新的结构约束明显且长期有效，再补进 `AGENTS.md`；否则不为了“同步”而添加噪音。

## Risks And Mitigations

### 风险 1：辅助函数可见性调整出错

`suggest_titles` 依赖若干内部辅助函数，拆模块时容易出现 `pub(crate)` 范围过大或过小的问题。

缓解方式：

- 先保持最小可见性
- 只对确实跨模块复用的符号做 `pub(crate)` 导出

### 风险 2：测试移动后行为没变但导出路径变了

`#[cfg(test)]` 下的 re-export 目前可能被其他测试依赖。拆分时要保留现有测试入口，避免只因为模块路径变化导致回归。

缓解方式：

- 优先维持 `radar.rs` 现有测试导出面
- 重构后先跑 `radar::` 相关 nextest，再跑全量检查

### 风险 3：顺手继续拆太多

这是典型的结构清理任务，最容易失控。

缓解方式：

- 只改 `radar.rs` 与 `src/radar/` 相关文件
- 不触碰 `cli.rs`、`markdown.rs`、`app.rs`

## Acceptance Criteria

满足以下条件即可视为这轮设计达成：

1. `src/radar.rs` 只保留总入口职责，不再包含 CSV 导入和标题建议实现细节。
2. 新增 `src/radar/import.rs` 与 `src/radar/suggest.rs`，职责边界清晰。
3. Radar 相关测试通过，且全量格式化 / lint / nextest 验证通过。
4. `PROGRESS.md` 记录这次结构清理和真实验证结果。
