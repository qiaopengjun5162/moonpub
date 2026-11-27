//! Unix 时间戳 → `YYYY-MM-DD`（UTC）。
//!
//! 依赖里没有日期库（不用 chrono / time），而 `intake` 和 `export` 都需要同一个
//! 「自 epoch 起的时间 → 日期字符串」换算。算法只留这一份，避免两处各写一遍民用历、
//! 一处闰年算错另一处看不出来。
//!
//! 换算用 Howard Hinnant 的 `civil_from_days`（以 400 年为一个 era 的整除技巧），
//! 对 1970 前后都成立，不需要查表。

use std::time::{SystemTime, UNIX_EPOCH};

/// 自 Unix epoch 起的天数 → `YYYY-MM-DD`（UTC）。
pub(crate) fn civil_from_days(days_since_epoch: i64) -> String {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + i64::from(m <= 2);
    format!("{year:04}-{m:02}-{d:02}")
}

/// 自 Unix epoch 起的秒数 → `YYYY-MM-DD`（UTC）。
///
/// 用 `div_euclid` 而不是 `/`：epoch 之前的负秒数要往下取整，`/` 会朝零截断，
/// 1970-01-01 前几小时会被算成第 0 天（也就是 1970-01-01 本身）。
pub(crate) fn civil_from_secs(secs: i64) -> String {
    civil_from_days(secs.div_euclid(86_400))
}

/// 把一个 `SystemTime` 格式化成 `YYYY-MM-DD`（UTC）。
///
/// 早于 epoch 的时间（`duration_since` 报错）按第 0 天处理，与传 0 秒一致。
pub(crate) fn date_utc(when: SystemTime) -> String {
    let secs = when
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    civil_from_secs(secs)
}

/// 当前 UTC 日期。
pub(crate) fn today_utc() -> String {
    date_utc(SystemTime::now())
}

#[cfg(test)]
mod tests {
    use super::{civil_from_days, civil_from_secs, date_utc};
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn civil_from_days_formats_the_epoch() {
        assert_eq!(civil_from_days(0), "1970-01-01");
    }

    #[test]
    fn civil_from_days_handles_leap_years_and_month_ends() {
        // 2024-02-29 是闰日；2023-03-01 是平年的同一天。
        assert_eq!(civil_from_days(19_782), "2024-02-29");
        assert_eq!(civil_from_days(19_783), "2024-03-01");
        assert_eq!(civil_from_days(19_417), "2023-03-01");
        assert_eq!(civil_from_days(20_000), "2024-10-04");
    }

    #[test]
    fn civil_from_days_handles_dates_before_the_epoch() {
        assert_eq!(civil_from_days(-1), "1969-12-31");
        assert_eq!(civil_from_days(-365), "1969-01-01");
    }

    #[test]
    fn civil_from_secs_truncates_toward_the_day_start() {
        // 2026-06-10T00:00:00Z 与当天最后一秒必须落在同一天。
        let start = 1_781_049_600;
        assert_eq!(civil_from_secs(start), "2026-06-10");
        assert_eq!(civil_from_secs(start + 86_399), "2026-06-10");
        assert_eq!(civil_from_secs(start + 86_400), "2026-06-11");
    }

    #[test]
    fn civil_from_secs_floors_negative_seconds_into_the_previous_day() {
        // -1 秒是 1969-12-31T23:59:59Z，必须留在 12-31，不能被截断成 1 月 1 日。
        assert_eq!(civil_from_secs(-1), "1969-12-31");
        assert_eq!(civil_from_secs(-86_400), "1969-12-31");
        assert_eq!(civil_from_secs(-86_401), "1969-12-30");
    }

    #[test]
    fn date_utc_formats_a_system_time() {
        let when = UNIX_EPOCH + Duration::from_secs(1_781_049_600);
        assert_eq!(date_utc(when), "2026-06-10");
        assert_eq!(date_utc(UNIX_EPOCH), "1970-01-01");
    }
}
