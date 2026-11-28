# Lean 4 样例：`bound_digest` 的截断不变量

**这不是 CI 的一部分，也不是质量门禁。** 它是 [`docs/FORMAL_VERIFICATION_ZH.md`](../../docs/FORMAL_VERIFICATION_ZH.md) 那份决策记录的配套样例——留着是为了让「Lean 4 证一遍到底长什么样、成本多少」有个可复现的实物，而不是一句口头结论。

## 里面证了什么

| 定理 | 命题 |
| --- | --- |
| `boundDigest_units_le` | 对任意字符列表，截断结果的 UTF-16 码元数 ≤ 120 |
| `boundDigest_is_prefix` | 截断只删尾巴，结果是原文前缀，不会凭空插字 |

两条都只依赖 `propext` / `Quot.sound`（Lean 内核标准公理），无 `sorry`、无 `admit`、无自定义 `axiom`。文件末尾的 `#print axioms` 就是用来验证这一点的。

## ⚠️ 它证的不是 Rust 代码

`takeUnder` 是照着 `src/wechat.rs` 里 `bound_digest` 的 `take_while` **手抄**的模型。Rust 那边改一行，这里完全不知道，证明照样「成立」。

真正守住线上代码的是 `src/property_tests.rs` 里那批属性测试——它们跑在编译出来的真实二进制上，代码一改就红。两边的关系是互补：

- Lean：对**任意输入**有保证，但对**代码是否改过**一无所知；
- 属性测试：对**代码改动**立刻反应，但只覆盖生成器造得出来的输入。

## 怎么跑

需要本机有 Lean 4（elan 管理）。这个文件不依赖 Mathlib，纯内核 + `omega`，秒级完成：

```bash
lean BoundDigest.lean
```

期望输出只有两条 `depends on axioms: [propext, Quot.sound]`，没有任何 error/warning。

## 什么时候该删掉这个目录

代码大改之后如果没人回来同步，这个模型就会悄悄失真，而失真的证明比没有证明更糟（它给人虚假的安全感）。判断标准很简单：**如果 `src/wechat.rs` 的 `bound_digest` 改了而这里没跟着改，就删掉这个目录**，别留着当装饰。
