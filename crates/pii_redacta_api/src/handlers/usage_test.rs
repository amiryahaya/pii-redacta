use super::*;

#[test]
fn test_parse_range_days_7d() {
    assert_eq!(parse_range_days("7d"), 7);
}

#[test]
fn test_parse_range_days_30d() {
    assert_eq!(parse_range_days("30d"), 30);
}

#[test]
fn test_parse_range_days_90d() {
    assert_eq!(parse_range_days("90d"), 90);
}

#[test]
fn test_parse_range_days_invalid() {
    assert_eq!(parse_range_days("invalid"), 30);
    assert_eq!(parse_range_days(""), 30);
    assert_eq!(parse_range_days("1d"), 30);
    assert_eq!(parse_range_days("365d"), 30);
}

#[test]
fn test_default_days() {
    assert_eq!(default_days(), 30);
}

#[test]
fn test_default_range() {
    assert_eq!(default_range(), "30d");
}
