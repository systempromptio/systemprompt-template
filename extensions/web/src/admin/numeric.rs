#[must_use]
#[inline]
#[expect(
    clippy::cast_precision_loss,
    reason = "DB counts/durations fit i32 range in practice; precision loss beyond 2^53 is acceptable for display math"
)]
pub fn to_f64(v: i64) -> f64 {
    v as f64
}

#[must_use]
#[inline]
#[expect(
    clippy::cast_possible_truncation,
    reason = "f64→f32 narrowing for display values where full f64 precision is unnecessary"
)]
pub fn to_f32(v: f64) -> f32 {
    v as f32
}

#[must_use]
#[inline]
pub fn saturating_i32(v: i64) -> i32 {
    i32::try_from(v).unwrap_or(if v > 0 { i32::MAX } else { i32::MIN })
}

#[must_use]
#[inline]
#[expect(
    clippy::cast_precision_loss,
    reason = "DB counts fit i32 range; precision loss beyond 2^53 is acceptable"
)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "f64→f32 narrowing for display values"
)]
pub fn to_f32_from_i64(v: i64) -> f32 {
    v as f64 as f32
}

#[must_use]
#[inline]
#[expect(
    clippy::cast_precision_loss,
    reason = "cast_sign_loss: duration seconds from chrono are always non-negative in this context"
)]
pub fn seconds_to_f64(v: i64) -> f64 {
    v as f64
}

#[must_use]
#[inline]
#[expect(
    clippy::cast_possible_truncation,
    reason = "f64→usize for small non-negative indices (e.g. hour 0–23); values are bounds-checked after conversion"
)]
#[expect(
    clippy::cast_sign_loss,
    reason = "negative values are clamped to 0.0 before conversion"
)]
pub fn to_usize(v: f64) -> usize {
    v.max(0.0) as usize
}

#[must_use]
#[inline]
#[expect(
    clippy::cast_possible_truncation,
    reason = "f64→i64 for computed byte rates; fractional part is intentionally discarded"
)]
pub fn to_i64(v: f64) -> i64 {
    v as i64
}

#[must_use]
#[inline]
#[expect(
    clippy::cast_possible_truncation,
    reason = "f64→i64 after rounding; values are display counters well within i64 range"
)]
pub fn round_to_i64(v: f64) -> i64 {
    v.round() as i64
}

#[must_use]
#[inline]
#[expect(
    clippy::cast_sign_loss,
    reason = "callers guarantee non-negative values (percentages, counts)"
)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "i64→usize safe for display counters on 64-bit platforms"
)]
pub fn i64_to_usize(v: i64) -> usize {
    v as usize
}

#[must_use]
#[inline]
pub fn saturating_i16(v: i64) -> i16 {
    i16::try_from(v).unwrap_or(if v > 0 { i16::MAX } else { i16::MIN })
}

#[must_use]
#[inline]
#[expect(
    clippy::cast_possible_truncation,
    reason = "f64→i16 narrowing for clamped score values (1-5 range)"
)]
pub fn f64_rounded_to_i16(v: f64) -> i16 {
    v.round() as i16
}

#[must_use]
#[inline]
#[expect(
    clippy::cast_possible_truncation,
    reason = "f64→i64 for percentage values (0-100 range)"
)]
pub fn pct_i64(numerator: i64, denominator: i64) -> i64 {
    (to_f64(numerator) / to_f64(denominator) * 100.0) as i64
}

#[must_use]
#[inline]
#[expect(
    clippy::cast_possible_wrap,
    reason = "usize→i64 for collection lengths that fit in practice"
)]
pub fn usize_to_i64(v: usize) -> i64 {
    v as i64
}

#[must_use]
#[inline]
#[expect(
    clippy::cast_precision_loss,
    reason = "usize→f64 for small collection lengths; precision loss beyond 2^53 is acceptable"
)]
pub fn usize_to_f64(v: usize) -> f64 {
    v as f64
}
