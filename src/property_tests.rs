//! 不变量测试：把「本该形式化证明的命题」写成对真实代码的属性测试。
//!
//! 为什么不用 proptest：项目零 dev-dependency，为三个不变量引入 proptest
//! （带 quick-error / rand / regex-syntax 等一串传递依赖）不划算。这里的
//! 生成器只有一个 xorshift64，种子写死，失败可复现。
//!
//! 这层测试和 Lean 证明是互补的，不是替代。
//!
//! Lean 证明（`verification/lean/BoundDigest.lean`）对**模型**给出任意输入的保证，
//! 但模型是手抄的，代码改了它不知道。这里的测试跑在**编译出来的真实代码**上，
//! 代码一改就红，但只覆盖到生成器能造出来的输入。两者叠起来才既「对所有输入
//! 成立」又「对真实代码成立」。

use crate::html::escape;
use crate::timefmt::{civil_from_days, civil_from_secs};
use crate::wechat::{DIGEST_MAX_UNITS, bound_digest, digest_units};

/// 确定性 xorshift64：同样的种子跑出同样的用例，失败可原样复现。
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// 字符池偏挑「会出事的」：星平面字符占 2 个 UTF-16 码元、空白会被折叠、
/// HTML 元字符要走转义、省略号本身是合法输入。
const POOL: [char; 15] = [
    'a', 'Z', '0', ' ', '\n', '\t', '中', '文', '，', '😀', '🚀', '&', '<', '>', '…',
];

fn random_text(rng: &mut Rng, max_chars: usize) -> String {
    let len = rng.below(max_chars + 1);
    (0..len).map(|_| POOL[rng.below(POOL.len())]).collect()
}

/// `bound_digest` 内部做的空白折叠，测试要复刻同一套口径来断言「前缀」。
fn normalized(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// 不变量 1：摘要截断后永远不超过微信的 120 码元上限
// ---------------------------------------------------------------------------

/// 对应 Lean 定理 `boundDigest_units_le`，但跑在真实代码上。
///
/// 这条不变量以前只写在 `bound_digest` 的实现里靠读代码相信：超限的摘要
/// 会让 `push` 直接失败，而失败信息是微信返回的一句「摘要过长」，
/// 看不出是哪个环节截断错了。
#[test]
fn digest_never_exceeds_wechat_utf16_limit_under_random_input() {
    let mut rng = Rng::new(0x5EED_1234);
    for _ in 0..20_000 {
        // 上限拉到 400 字符，确保越过 120 这条线，同时混入大量 2 码元字符。
        let text = random_text(&mut rng, 400);
        let out = bound_digest(&text);
        let units = digest_units(&out);
        assert!(
            units <= DIGEST_MAX_UNITS,
            "bound_digest 超限: {units} > {DIGEST_MAX_UNITS}, input={text:?}, out={out:?}"
        );
    }
}

/// 边界值：正好 119 / 120 / 121 码元，以及用一个 2 码元字符跨越界线。
#[test]
fn digest_boundary_is_exact_at_120_units() {
    for units in [0, 1, 118, 119, 120, 121, 122, 240] {
        let text = "a".repeat(units);
        let out = bound_digest(&text);
        assert!(digest_units(&out) <= DIGEST_MAX_UNITS, "units={units}");
        if units <= DIGEST_MAX_UNITS {
            assert_eq!(out, text, "未超限的摘要必须原样保留，units={units}");
        } else {
            assert!(out.ends_with('…'), "超限必须加省略号，units={units}");
        }
    }

    // 118 个 ASCII 之后来一个星平面字符：119 装得下，再下一个就装不下。
    let astral = format!("{}😀", "a".repeat(118));
    assert_eq!(digest_units(&astral), 120, "前提：这个输入正好 120 码元");
    assert_eq!(bound_digest(&astral), astral, "120 是允许的，不该截断");

    let over = format!("{}😀b", "a".repeat(118));
    let out = bound_digest(&over);
    assert!(digest_units(&out) <= DIGEST_MAX_UNITS);
    assert!(out.ends_with('…'));
    // 截断不能把星平面字符劈成半个代理对。
    assert!(out.chars().all(|c| c.len_utf16() <= 2));
}

/// 截断只删尾巴，不插字、不重排：去掉末尾省略号后必须是原文（折叠空白后）的前缀。
#[test]
fn digest_truncation_only_cuts_the_tail() {
    let mut rng = Rng::new(0xC0FF_EE01);
    for _ in 0..20_000 {
        let text = random_text(&mut rng, 400);
        let out = bound_digest(&text);
        let body = out.strip_suffix('…').unwrap_or(&out);
        let base = normalized(&text);
        assert!(
            base.starts_with(body),
            "截断结果不是原文前缀: body={body:?}, normalized={base:?}"
        );
    }
}

/// 幂等：对已经处理过的摘要再处理一次，结果不变（重推草稿时会走两遍）。
#[test]
fn digest_bounding_is_idempotent() {
    let mut rng = Rng::new(0x1DE_A100);
    for _ in 0..5_000 {
        let text = random_text(&mut rng, 400);
        let once = bound_digest(&text);
        let twice = bound_digest(&once);
        assert_eq!(once, twice, "二次截断改变了结果, input={text:?}");
    }
}

// ---------------------------------------------------------------------------
// 不变量 2：日期换算与民用历互逆
// ---------------------------------------------------------------------------

/// Hinnant `days_from_civil`，`civil_from_days` 的逆运算。
///
/// 这里重写一遍而不是调库里现成的，是为了让「互逆」这条性质真的被检验：
/// 如果两边共用同一份实现，写错的地方会一起错，round-trip 测试就成了自证。
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn parse_civil(formatted: &str) -> (i64, i64, i64) {
    assert_eq!(formatted.len(), 10, "格式必须是 YYYY-MM-DD: {formatted:?}");
    let bytes = formatted.as_bytes();
    assert_eq!(bytes[4], b'-', "年与月之间必须是横线: {formatted:?}");
    assert_eq!(bytes[7], b'-', "月与日之间必须是横线: {formatted:?}");
    let year: i64 = formatted[0..4].parse().expect("年是四位数字");
    let month: i64 = formatted[5..7].parse().expect("月是两位数字");
    let day: i64 = formatted[8..10].parse().expect("日是两位数字");
    (year, month, day)
}

/// 完整 400 年格里历周期逐日 round-trip。
///
/// 146097 天正好覆盖闰年规则的全部组合：4 年一闰、100 年不闰、400 年又闰。
/// 从 1970 起算的这一轮里，2000 年是「400 年闰」、2100 年是「100 年不闰」，
/// 两个最容易写错的都被覆盖到。
#[test]
fn civil_days_round_trips_over_a_full_gregorian_cycle() {
    for days in 0..146_097i64 {
        let (y, m, d) = parse_civil(&civil_from_days(days));
        assert_eq!(
            days_from_civil(y, m, d),
            days,
            "round-trip 失败于 day={days}"
        );
    }
}

/// epoch 之前的负天数也要逐日 round-trip：`div_euclid` 写错成 `/` 就死在这里。
#[test]
fn civil_days_round_trips_before_the_epoch() {
    for days in -146_097..0i64 {
        let (y, m, d) = parse_civil(&civil_from_days(days));
        assert_eq!(
            days_from_civil(y, m, d),
            days,
            "round-trip 失败于 day={days}"
        );
    }
}

/// 每一年（0..9999）的首日与末日都要 round-trip，顺带守「补零」：
/// 1999-01-01 不能输出成 1999-1-1。
#[test]
fn civil_days_round_trips_on_every_year_boundary() {
    for year in 0..10_000i64 {
        for (month, day) in [(1, 1), (12, 31)] {
            let days = days_from_civil(year, month, day);
            let want = format!("{year:04}-{month:02}-{day:02}");
            assert_eq!(civil_from_days(days), want);
        }
    }
}

/// 严格单调：日期字符串的字典序必须与天数序一致。
///
/// 这条能抓到「格式宽度不齐」那类 bug —— 一旦某年输出成 5 位或带符号，
/// 字典序就断了，而 round-trip 测试发现不了（因为它只看单个值）。
#[test]
fn civil_days_string_order_matches_day_order() {
    let mut previous = civil_from_days(0);
    for days in 1..73_050i64 {
        let current = civil_from_days(days);
        assert!(
            current > previous,
            "字典序断裂: day={days} -> {current:?} 不大于 {previous:?}"
        );
        previous = current;
    }
}

/// 一天之内的任何一秒都必须落在同一天上，跨过零点才换日。
///
/// 这条专门守 `civil_from_secs` 的取整方向：写成 `/` 而不是 `div_euclid`
/// 时，epoch 之前的秒数会朝零截断，`1969-12-31T23:59:59Z` 被算成 1970-01-01。
/// 命题写成「秒 → 天」而不是照抄 `secs.div_euclid(86_400)`，否则测试和实现
/// 用同一份除法，写错的地方会一起错。
#[test]
fn civil_secs_keeps_every_second_of_a_day_on_that_day() {
    let mut rng = Rng::new(0x5EC5_0001);
    for _ in 0..20_000 {
        let day = rng.below(2_000_001) as i64 - 1_000_000;
        let offset = rng.below(86_400) as i64;
        let secs = day * 86_400 + offset;
        assert_eq!(
            civil_from_secs(secs),
            civil_from_days(day),
            "秒落在错的一天: secs={secs}, offset={offset}"
        );
    }
    // 边界：当天第一秒、最后一秒、下一秒。
    assert_eq!(civil_from_secs(-1), civil_from_days(-1));
    assert_eq!(civil_from_secs(-86_400), civil_from_days(-1));
    assert_eq!(civil_from_secs(-86_401), civil_from_days(-2));
    assert_eq!(civil_from_secs(86_399), civil_from_days(0));
    assert_eq!(civil_from_secs(86_400), civil_from_days(1));
}

// ---------------------------------------------------------------------------
// 不变量 3：HTML 转义不残留裸元字符，且可无损还原
// ---------------------------------------------------------------------------

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(index) = rest.find('&') {
        out.push_str(&rest[..index]);
        rest = &rest[index..];
        let (decoded, len) = if rest.starts_with("&amp;") {
            ('&', 5)
        } else if rest.starts_with("&lt;") {
            ('<', 4)
        } else if rest.starts_with("&gt;") {
            ('>', 4)
        } else if rest.starts_with("&quot;") {
            ('"', 6)
        } else if rest.starts_with("&#39;") {
            ('\'', 5)
        } else {
            ('&', 1)
        };
        out.push(decoded);
        rest = &rest[len..];
    }
    out.push_str(rest);
    out
}

/// 转义后的文本里不能再出现裸 `<` 或 `>`。
///
/// 这条是「正文里的尖括号不会变成标签」的形式化版本：公众号后台会原样接受
/// 我们推上去的 HTML，一旦有裸标签漏进去，轻则排版错乱，重则整段被吞。
#[test]
fn escape_leaves_no_bare_angle_bracket() {
    let mut rng = Rng::new(0xB00D_0001);
    for _ in 0..20_000 {
        let text = random_text(&mut rng, 80);
        let escaped = escape(&text);
        assert!(
            !escaped.contains('<') && !escaped.contains('>'),
            "转义后仍有裸尖括号: input={text:?}, out={escaped:?}"
        );
    }
}

/// 转义不丢信息：还原后与原串逐字节相等。
#[test]
fn escape_round_trips_through_unescape() {
    let mut rng = Rng::new(0xB00D_0002);
    for _ in 0..20_000 {
        let text = random_text(&mut rng, 80);
        assert_eq!(unescape(&escape(&text)), text, "还原失败: input={text:?}");
    }
}
