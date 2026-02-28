use super::*;
use axum::response::IntoResponse;

#[test]
fn test_subscription_error_not_found_status() {
    let err = SubscriptionError::NotFound;
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_subscription_error_database_status() {
    let err = SubscriptionError::Database(sqlx::Error::RowNotFound);
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_tier_response_serialization() {
    let tier = TierResponse {
        id: "tier-1".to_string(),
        name: "pro".to_string(),
        display_name: "Professional".to_string(),
        description: Some("Pro tier".to_string()),
        limits: serde_json::json!({"max_files_per_month": 1000}),
        features: serde_json::json!({"pdf_support": true}),
        monthly_price_cents: Some(2999),
        yearly_price_cents: Some(29990),
    };

    let json = serde_json::to_value(&tier).expect("Should serialize");
    // Verify camelCase field names
    assert_eq!(json["displayName"], "Professional");
    assert_eq!(json["monthlyPriceCents"], 2999);
    assert_eq!(json["yearlyPriceCents"], 29990);
    assert_eq!(json["name"], "pro");
}
