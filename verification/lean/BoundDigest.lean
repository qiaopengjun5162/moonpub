/-
  MoonPub `wechat::bound_digest` 的核心不变量，用 Lean 4 建模并证明。

  被证的命题（Rust 里没有任何地方写下来过，只能靠读代码相信）：
    对任意输入，截断结果的 UTF-16 码元数 ≤ 120。

  注意这是**模型**：`takeUnder` 是照着 `bound_digest` 的 `take_while` 手抄的，
  Lean 不知道 Rust 那边有没有改。模型与代码的一致性由 Rust 侧属性测试兜住。
-/

namespace Moonpub

/-- 一个字符在 UTF-16 里占几个码元：星平面（≥ U+10000）占 2，其余 1。 -/
def u16 (c : Char) : Nat := if c.val.toNat < 0x10000 then 1 else 2

/-- 等价于 Rust 的 `s.encode_utf16().count()`。 -/
def units : List Char → Nat
  | [] => 0
  | c :: cs => u16 c + units cs

/-- `bound_digest` 里的 `take_while` 循环：累加到 120 就停。 -/
def takeUnder : Nat → Nat → List Char → List Char
  | 0, _, _ => []
  | _ + 1, _, [] => []
  | n + 1, u, c :: cs =>
      if u + u16 c < 120 then c :: takeUnder n (u + u16 c) cs else []

/-- 完整模型：取前缀 + 省略号。 -/
def boundDigest (s : List Char) : List Char := takeUnder s.length 0 s ++ ['…']

theorem units_append : ∀ (a b : List Char), units (a ++ b) = units a + units b
  | [], _ => by simp [units]
  | c :: cs, b => by
    simp only [List.cons_append, units]
    rw [units_append cs b]
    omega

theorem dots_units : u16 '…' = 1 := by decide

/-- 循环不变量：起点累加值 < 120，则结束时累加值仍 < 120。 -/
theorem takeUnder_bound :
    ∀ (n u : Nat) (s : List Char), u < 120 → u + units (takeUnder n u s) < 120
  | 0, u, _, h => by simp only [takeUnder, units]; omega
  | _ + 1, u, [], h => by simp only [takeUnder, units]; omega
  | n + 1, u, c :: cs, h => by
    by_cases hlt : u + u16 c < 120
    · simp only [takeUnder, hlt, if_true, units]
      have ih := takeUnder_bound n (u + u16 c) cs hlt
      omega
    · simp only [takeUnder, hlt, if_false, units]
      omega

/-- **主定理**：截断后的摘要永远不超过微信的 120 码元上限。 -/
theorem boundDigest_units_le (s : List Char) : units (boundDigest s) ≤ 120 := by
  have h := takeUnder_bound s.length 0 s (by decide)
  simp only [boundDigest, units_append, units, dots_units]
  omega

/-- 推论：截断结果去掉末尾省略号后，仍是原文的前缀（不会凭空插字）。 -/
theorem boundDigest_is_prefix (s : List Char) :
    ∃ t, boundDigest s = t ++ ['…'] ∧ t <+: s := by
  refine ⟨takeUnder s.length 0 s, rfl, ?_⟩
  exact takeUnder_prefix s.length 0 s
where
  takeUnder_prefix : ∀ (n u : Nat) (s : List Char), takeUnder n u s <+: s
    | 0, _, _ => by simp only [takeUnder]; exact List.nil_prefix
    | _ + 1, _, [] => by simp only [takeUnder]; exact List.nil_prefix
    | n + 1, u, c :: cs => by
      by_cases hlt : u + u16 c < 120
      · simp only [takeUnder, hlt, if_true]
        exact List.cons_prefix_cons.mpr ⟨rfl, takeUnder_prefix n (u + u16 c) cs⟩
      · simp only [takeUnder, hlt, if_false]
        exact List.nil_prefix

end Moonpub

#print axioms Moonpub.boundDigest_units_le
#print axioms Moonpub.boundDigest_is_prefix
