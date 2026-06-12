default:
    @just --list

# 代码格式化
fmt:
    cargo fmt

# TOML 格式化
taplo-fmt:
    taplo fmt --option reorder_keys=true

# 静态分析 (零容忍)
clippy:
    cargo clippy --all-targets --all-features --tests --benches -- -D warnings

# 运行测试 (使用 cargo-nextest)
test:
    cargo nextest run --all-features

# 依赖审计
deny:
    cargo deny check

# 拼写检查
typos:
    typos

# 安装到本地
install:
    cargo install --path . --quiet

# CI 完整检查
check: fmt taplo-fmt clippy test deny typos

# 打 tag 并推送 (触发 GitHub Actions release)
release version:
    git tag -a "v{{version}}" -m "v{{version}}"
    git push origin "v{{version}}"
