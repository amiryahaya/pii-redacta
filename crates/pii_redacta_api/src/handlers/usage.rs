//! Usage analytics handlers for Portal
//!
//! Provides endpoints for users to view their usage statistics.

use crate::extractors::AuthUser;
use crate::AppState;
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Datelike, Duration, Months, Utc};
use serde::{Deserialize, Serialize};

/// Error type for usage handlers
#[derive(Debug, thiserror::Error)]
pub enum UsageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for UsageError {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::json!({
            "error": {
                "code": 500,
                "message": "An unexpected error occurred",
            }
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}

/// Usage statistics response
#[derive(Debug, Serialize)]
pub struct UsageStatsResponse {
    pub total_requests: i64,
    pub total_files: i64,
    pub total_pages: i64,
    pub storage_used: i64,
    pub monthly_files: i64,
    pub monthly_limit: Option<i32>,
}

/// Daily usage entry
#[derive(Debug, Serialize)]
pub struct DailyUsageResponse {
    pub date: String,
    pub requests: i64,
    pub files: i64,
    pub pages: i64,
}

/// Query parameters for daily usage
#[derive(Debug, Deserialize)]
pub struct DailyUsageQuery {
    #[serde(default = "default_days")]
    pub days: i32,
}

fn default_days() -> i32 {
    30
}

/// Get usage statistics for the authenticated user
pub async fn get_usage_stats(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<UsageStatsResponse>, UsageError> {
    // Get total stats
    let totals = sqlx::query_as::<_, (i64, i64, i64)>(
        r#"
        SELECT 
            COALESCE(SUM(CASE WHEN request_type LIKE 'api_%' THEN 1 ELSE 0 END), 0) as requests,
            COALESCE(SUM(CASE WHEN file_name IS NOT NULL THEN 1 ELSE 0 END), 0) as files,
            COALESCE(SUM(page_count), 0) as pages
        FROM usage_logs 
        WHERE user_id = $1
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_one(state.db.pool())
    .await?;

    // Get current month file count
    let now = Utc::now();
    let month_start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
    let monthly_files = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM usage_logs 
        WHERE user_id = $1 
        AND file_name IS NOT NULL
        AND created_at >= $2
        "#,
    )
    .bind(auth_user.user_id)
    .bind(month_start)
    .fetch_one(state.db.pool())
    .await?;

    // Get user's tier limit
    let monthly_limit = sqlx::query_scalar::<_, Option<i32>>(
        r#"
        SELECT (t.limits->>'max_files_per_month')::int
        FROM subscriptions s
        JOIN tiers t ON s.tier_id = t.id
        WHERE s.user_id = $1 
        AND s.status IN ('trial', 'active', 'past_due')
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?
    .flatten();

    // Calculate storage (sum of file sizes for last 30 days as approximation)
    let storage_used = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(SUM(file_size_bytes), 0) 
        FROM usage_logs 
        WHERE user_id = $1 
        AND created_at >= NOW() - INTERVAL '30 days'
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_one(state.db.pool())
    .await?;

    Ok(Json(UsageStatsResponse {
        total_requests: totals.0,
        total_files: totals.1,
        total_pages: totals.2,
        storage_used,
        monthly_files,
        monthly_limit,
    }))
}

/// Get daily usage breakdown
pub async fn get_daily_usage(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<DailyUsageQuery>,
) -> Result<Json<Vec<DailyUsageResponse>>, UsageError> {
    let days = query.days.clamp(1, 90);
    let start_date = Utc::now() - Duration::days(days as i64);

    let rows = sqlx::query_as::<_, (chrono::NaiveDate, i64, i64, i64)>(
        r#"
        SELECT 
            DATE(created_at) as date,
            COALESCE(SUM(CASE WHEN request_type LIKE 'api_%' THEN 1 ELSE 0 END), 0) as requests,
            COALESCE(SUM(CASE WHEN file_name IS NOT NULL THEN 1 ELSE 0 END), 0) as files,
            COALESCE(SUM(page_count), 0) as pages
        FROM usage_logs 
        WHERE user_id = $1 
        AND created_at >= $2
        GROUP BY DATE(created_at)
        ORDER BY date DESC
        "#,
    )
    .bind(auth_user.user_id)
    .bind(start_date)
    .fetch_all(state.db.pool())
    .await?;

    let daily: Vec<DailyUsageResponse> = rows
        .into_iter()
        .map(|(date, requests, files, pages)| DailyUsageResponse {
            date: date.to_string(),
            requests,
            files,
            pages,
        })
        .collect();

    Ok(Json(daily))
}

/// Dashboard statistics response
#[derive(Debug, Serialize)]
pub struct DashboardStatsResponse {
    pub stats: DashboardStatValues,
    pub charts: DashboardCharts,
    #[serde(rename = "recentActivity")]
    pub recent_activity: Vec<DashboardActivity>,
}

#[derive(Debug, Serialize)]
pub struct DashboardStatValues {
    #[serde(rename = "totalRequests")]
    pub total_requests: i64,
    #[serde(rename = "totalDocuments")]
    pub total_documents: i64,
    #[serde(rename = "quotaUsage")]
    pub quota_usage: f64,
    #[serde(rename = "documentsChange")]
    pub documents_change: f64,
    #[serde(rename = "requestsChange")]
    pub requests_change: f64,
}

#[derive(Debug, Serialize)]
pub struct DashboardCharts {
    #[serde(rename = "dailyRequests")]
    pub daily_requests: Vec<DailyDataPoint>,
    #[serde(rename = "dailyDocuments")]
    pub daily_documents: Vec<DailyDataPoint>,
}

#[derive(Debug, Serialize)]
pub struct DailyDataPoint {
    pub date: String,
    pub value: i64,
}

#[derive(Debug, Serialize)]
pub struct DashboardActivity {
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub description: String,
    pub timestamp: String,
}

/// Get dashboard statistics for the authenticated user
pub async fn get_dashboard_stats(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<DashboardStatsResponse>, UsageError> {
    let now = Utc::now();
    let current_month_start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
    let prev_month_start = current_month_start - Months::new(1);

    // Current month totals
    let current = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT
            COUNT(*) as requests,
            COALESCE(SUM(CASE WHEN file_name IS NOT NULL THEN 1 ELSE 0 END), 0) as documents
        FROM usage_logs
        WHERE user_id = $1 AND created_at >= $2
        "#,
    )
    .bind(auth_user.user_id)
    .bind(current_month_start)
    .fetch_one(state.db.pool())
    .await?;

    // Previous month totals for change %
    let previous = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT
            COUNT(*) as requests,
            COALESCE(SUM(CASE WHEN file_name IS NOT NULL THEN 1 ELSE 0 END), 0) as documents
        FROM usage_logs
        WHERE user_id = $1 AND created_at >= $2 AND created_at < $3
        "#,
    )
    .bind(auth_user.user_id)
    .bind(prev_month_start)
    .bind(current_month_start)
    .fetch_one(state.db.pool())
    .await?;

    // Quota from tier (max_files_per_month is the JSONB key in seed data)
    let monthly_limit = sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT (t.limits->>'max_files_per_month')::bigint
        FROM subscriptions s
        JOIN tiers t ON s.tier_id = t.id
        WHERE s.user_id = $1 AND s.status IN ('trial', 'active', 'past_due')
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?
    .flatten();

    let quota_usage = match monthly_limit {
        Some(limit) if limit > 0 => (current.1 as f64 / limit as f64) * 100.0,
        _ => 0.0,
    };

    let requests_change = if previous.0 > 0 {
        ((current.0 - previous.0) as f64 / previous.0 as f64) * 100.0
    } else {
        0.0
    };
    let documents_change = if previous.1 > 0 {
        ((current.1 - previous.1) as f64 / previous.1 as f64) * 100.0
    } else {
        0.0
    };

    // Last 30 days for charts
    let chart_start = now - Duration::days(30);
    let daily_rows = sqlx::query_as::<_, (chrono::NaiveDate, i64, i64)>(
        r#"
        SELECT
            DATE(created_at) as date,
            COUNT(*) as requests,
            COALESCE(SUM(CASE WHEN file_name IS NOT NULL THEN 1 ELSE 0 END), 0) as documents
        FROM usage_logs
        WHERE user_id = $1 AND created_at >= $2
        GROUP BY DATE(created_at)
        ORDER BY date ASC
        "#,
    )
    .bind(auth_user.user_id)
    .bind(chart_start)
    .fetch_all(state.db.pool())
    .await?;

    let daily_requests: Vec<DailyDataPoint> = daily_rows
        .iter()
        .map(|(date, requests, _)| DailyDataPoint {
            date: date.to_string(),
            value: *requests,
        })
        .collect();

    let daily_documents: Vec<DailyDataPoint> = daily_rows
        .iter()
        .map(|(date, _, documents)| DailyDataPoint {
            date: date.to_string(),
            value: *documents,
        })
        .collect();

    // Recent activity (last 10 entries)
    let activities =
        sqlx::query_as::<_, (uuid::Uuid, String, Option<String>, chrono::DateTime<Utc>)>(
            r#"
        SELECT id, request_type, file_name, created_at
        FROM usage_logs
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 10
        "#,
        )
        .bind(auth_user.user_id)
        .fetch_all(state.db.pool())
        .await?;

    let recent_activity: Vec<DashboardActivity> = activities
        .into_iter()
        .map(|(id, req_type, file_name, created_at)| {
            let description = match file_name {
                Some(name) => format!("Processed file: {}", name),
                None => format!("API request: {}", req_type),
            };
            DashboardActivity {
                id: id.to_string(),
                activity_type: req_type,
                description,
                timestamp: created_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(Json(DashboardStatsResponse {
        stats: DashboardStatValues {
            total_requests: current.0,
            total_documents: current.1,
            quota_usage,
            requests_change,
            documents_change,
        },
        charts: DashboardCharts {
            daily_requests,
            daily_documents,
        },
        recent_activity,
    }))
}

/// Usage summary response (lightweight)
#[derive(Debug, Serialize)]
pub struct UsageSummaryResponse {
    pub total_requests: i64,
    pub total_documents: i64,
    pub quota_usage: f64,
}

/// Get a lightweight usage summary for the authenticated user
pub async fn get_usage_summary(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<UsageSummaryResponse>, UsageError> {
    let now = Utc::now();
    let month_start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();

    let current = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT
            COUNT(*) as requests,
            COALESCE(SUM(CASE WHEN file_name IS NOT NULL THEN 1 ELSE 0 END), 0) as documents
        FROM usage_logs
        WHERE user_id = $1 AND created_at >= $2
        "#,
    )
    .bind(auth_user.user_id)
    .bind(month_start)
    .fetch_one(state.db.pool())
    .await?;

    let monthly_limit = sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT (t.limits->>'max_files_per_month')::bigint
        FROM subscriptions s
        JOIN tiers t ON s.tier_id = t.id
        WHERE s.user_id = $1 AND s.status IN ('trial', 'active', 'past_due')
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?
    .flatten();

    let quota_usage = match monthly_limit {
        Some(limit) if limit > 0 => (current.1 as f64 / limit as f64) * 100.0,
        _ => 0.0,
    };

    Ok(Json(UsageSummaryResponse {
        total_requests: current.0,
        total_documents: current.1,
        quota_usage,
    }))
}
