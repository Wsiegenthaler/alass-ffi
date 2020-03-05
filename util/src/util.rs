use alass_core::*;

use encoding_rs::Encoding;

use alass_core::TimeDelta;

use subparse::timetypes::TimeSpan as SubTimeSpan;
use subparse::timetypes::TimePoint as SubTimePoint;
use subparse::timetypes::TimeDelta as SubTimeDelta;

///
/// Converts optional `TimeDelta` from `alass_core` to human readable representation
/// 
pub fn delta_str(d: Option<&TimeDelta>, interval: i64) -> String {
    d.map(|d| *d * interval)
        .map(|d| format!("{}", d))
        .unwrap_or_else(|| String::from("???"))
}

///
/// Scales the start and end of each span by a given factor
/// 
pub fn scaled_timespan(ts: SubTimeSpan, factor: f64) -> SubTimeSpan {
    SubTimeSpan::new(
        SubTimePoint::from_msecs((ts.start.msecs() as f64 * factor) as i64),
        SubTimePoint::from_msecs((ts.end.msecs() as f64 * factor) as i64),
    )
}

///
/// Convert `TimePoint` from `subparse` to `alass_core` representation
/// 
pub fn to_alass_timepoint(t: SubTimePoint, interval: i64) -> TimePoint {
    TimePoint::from(t.msecs() / interval)
}

///
/// Convert `TimeDelta` from `alass_core` to `subparse` representation
/// 
pub fn to_subparse_delta(t: TimeDelta, interval: i64) -> SubTimeDelta {
    SubTimeDelta::from_msecs(i64::from(t) * interval)
}

///
/// Convert `TimeDelta`s from `alass_core` to `subparse` representation
/// 
pub fn to_subparse_deltas(v: &[TimeDelta], interval: i64) -> Vec<SubTimeDelta> {
    v.iter().cloned().map(|x| to_subparse_delta(x, interval)).collect()
}

///
/// Retrieves `Encoding` given it's label
/// 
pub fn lookup_encoding(label: Option<String>) -> Option<&'static Encoding> {
    label.map(|l| Encoding::for_label(l.as_bytes())).flatten()
}

///
/// Attempts to detect character encoding by inspecting raw bytes
/// 
pub fn detect_encoding(data: &[u8]) -> Option<&'static Encoding> {
    let (charset, confidence, _) = chardet::detect(data);
    let charset = chardet::charset2encoding(&charset);
    let encoding = Encoding::for_label(charset.as_bytes());
    if encoding.is_some() {
        log::info!("subtitle encoding '{}' detected (confidence='{}')", charset, confidence);
    }
    encoding
}

///
/// Attempts to detect character encoding by inspecting raw bytes. Fall back to
/// default value if detection fails
/// 
pub fn detect_encoding_or(data: &[u8], fallback: &'static Encoding) -> &'static Encoding {
    detect_encoding(data).unwrap_or(fallback)
}
