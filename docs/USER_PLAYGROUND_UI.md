# User Testing Interface (Playground)

**Purpose:** Simple web UI for testing PII detection without API integration  
**Target Users:** Evaluators, developers testing the service  
**Future:** Foundation for enterprise user portal with usage controls

---

## UI Concept

```
┌─────────────────────────────────────────────────────────────────┐
│  🔒 PII Redacta              [Login] [Sign Up]         v0.1.0   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                                                          │  │
│  │         📄 Test Your Document                            │  │
│  │                                                          │  │
│  │    ┌────────────────────────────────────────────────┐   │  │
│  │    │                                                  │   │  │
│  │    │    📤 Drop file here or click to browse         │   │  │
│  │    │                                                  │   │  │
│  │    │    Supports: TXT, PDF, DOCX (Max 5MB)           │   │  │
│  │    │                                                  │   │  │
│  │    └────────────────────────────────────────────────┘   │  │
│  │                                                          │  │
│  │    ┌────────────────────────────────────────────────┐   │  │
│  │    │ [x] Redact PII (replace with tokens)           │   │  │
│  │    │                                                  │   │  │
│  │    │ [  🔍 Process Document  ]                       │   │  │
│  │    └────────────────────────────────────────────────┘   │  │
│  │                                                          │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  📊 Results                                              │  │
│  │                                                          │  │
│  │  Tab: [Detected PII] [Redacted Text] [Download]         │  │
│  │                                                          │  │
│  │  ┌─────────────────┐  ┌──────────────────────────────┐  │  │
│  │  │ Type   │ Value  │  │ Email: <<PII_EMAIL_...>>    │  │  │
│  │  │────────│────────│  │ IC: <<PII_MY_NRIC_...>>     │  │  │
│  │  │ Email  │ a@b... │  │ Phone: <<PII_PHONE_...>>    │  │  │
│  │  │ NRIC   │ 850... │  │                               │  │  │
│  │  │ Phone  │ 012... │  │ [📋 Copy]  [⬇️ Download]    │  │  │
│  │  └─────────────────┘  └──────────────────────────────┘  │  │
│  │                                                          │  │
│  │  Processing time: 45ms | 3 entities found               │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    USER PLAYGROUND ARCHITECTURE                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐       ┌─────────────┐       ┌─────────────┐  │
│  │   Browser   │──────▶│  Leptos UI  │──────▶│  Axum API   │  │
│  │  (User)     │◀──────│  (WASM)     │◀──────│  (Backend)  │  │
│  └─────────────┘       └─────────────┘       └──────┬──────┘  │
│                                                     │          │
│                           ┌────────────────────────┼──────┐   │
│                           │                        │      │   │
│                           ▼                        ▼      ▼   │
│                    ┌────────────┐          ┌──────────┐      │
│                    │ PostgreSQL │          │  Redis   │      │
│                    │ (Users,    │          │ (Session,│      │
│                    │  Usage)    │          │  Cache)  │      │
│                    └────────────┘          └──────────┘      │
│                                                               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phased Implementation

### Phase 1: Basic Playground (Anonymous)

**Features:**
- File upload (drag & drop)
- Text paste input
- PII detection display
- Optional redaction
- No login required

**Limits (hardcoded):**
- Max file size: 1MB
- Max 5 files per day per IP
- No storage (process and discard)

**Sprint:** 1 week

---

### Phase 2: User Accounts

**Features:**
- Simple registration (email/password)
- Login/logout
- History of processed files (last 10)
- Higher limits for registered users

**Limits per account:**
- Max file size: 5MB
- Max 50 files per month
- 30-day history retention

**Database:**
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    plan VARCHAR(20) DEFAULT 'free'  -- 'free', 'starter', 'pro'
);

CREATE TABLE processing_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    filename VARCHAR(255),
    file_size INTEGER,
    entities_found INTEGER,
    processing_time_ms INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Sprint:** 2 weeks

---

### Phase 3: Enterprise Controls

**Features:**
- Organization accounts
- Admin dashboard
- Usage quotas per user/org
- Custom file size limits
- Priority processing
- Audit logs

**Plans:**
| Plan | Users | Files/Month | Max Size | Features |
|------|-------|-------------|----------|----------|
| Free | 1 | 50 | 5MB | Basic |
| Starter | 5 | 500 | 10MB | API access |
| Pro | 20 | 5000 | 50MB | Webhooks |
| Enterprise | ∞ | ∞ | Custom | SLA, SSO |

**Sprint:** 3 weeks

---

## Tech Stack Recommendation

### Frontend: Leptos (Rust WASM)

```rust
// Leptos component for file upload
#[component]
fn FileUpload() -> impl IntoView {
    let (file, set_file) = create_signal(None::<File>);
    let (result, set_result) = create_signal(None::<ProcessingResult>);
    let (loading, set_loading) = create_signal(false);
    
    let on_drop = move |ev: DragEvent| {
        ev.prevent_default();
        if let Some(files) = ev.data_transfer() {
            if let Some(file) = files.files().get(0) {
                set_file.set(Some(file));
            }
        }
    };
    
    let process = move |_| {
        if let Some(file) = file.get() {
            set_loading.set(true);
            
            spawn_local(async move {
                let form_data = FormData::new().unwrap();
                form_data.append_with_blob("file", &file).unwrap();
                
                let resp = Request::post("/api/v1/upload")
                    .body(form_data)
                    .send()
                    .await
                    .unwrap();
                
                let result: ProcessingResult = resp.json().await.unwrap();
                set_result.set(Some(result));
                set_loading.set(false);
            });
        }
    };
    
    view! {
        <div class="upload-zone" on:drop=on_drop>
            {move || file.get().map(|f| view! {
                <p>"Selected: " {f.name()}</p>
            })}
            
            <button on:click=process disabled=loading>
                {move || if loading.get() { "Processing..." } else { "Process" }}
            </button>
            
            {move || result.get().map(|r| view! {
                <ResultsDisplay result=r />
            })}
        </div>
    }
}
```

### Alternative: HTMX + Tera (Simpler)

If you want simpler server-side rendering:

```html
<!-- templates/upload.html -->
<div hx-post="/upload" hx-target="#results" enctype="multipart/form-data">
    <input type="file" name="file" accept=".txt,.pdf,.docx">
    <button type="submit">Upload</button>
</div>

<div id="results"></div>
```

---

## File Processing Flow

```
User Upload
    │
    ▼
┌─────────────────┐
│ Validate File   │──Error──▶ 400 Bad Request
│ - Size check    │
│ - Type check    │
│ - Virus scan?   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Extract Text    │
│ - TXT: direct   │
│ - PDF: pdf-extract
│ - DOCX: unzip   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Detect PII      │
│ - Run patterns  │
│ - Get entities  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────┐
│ Optional:       │────▶│ Tokenize    │
│ Redact PII      │     │ & Replace   │
└────────┬────────┘     └──────┬──────┘
         │                      │
         └──────────┬───────────┘
                    ▼
         ┌─────────────────┐
         │ Return Results  │
         │ - Entity list   │
         │ - Redacted text │
         │ - Download link │
         └─────────────────┘
```

---

## Rate Limiting Strategy

### Anonymous Users
```rust
// Rate limit by IP address
pub async fn anonymous_rate_limit(
    ip: IpAddr,
    redis: &mut redis::aio::Connection,
) -> Result<bool> {
    let key = format!("anon_limit:{}", ip);
    let count: i32 = redis.incr(&key, 1).await?;
    
    if count == 1 {
        redis.expire(&key, 86400).await?; // 24 hours
    }
    
    Ok(count <= 5) // 5 uploads per day
}
```

### Authenticated Users
```rust
// Rate limit by user ID
pub async fn user_rate_limit(
    user_id: Uuid,
    plan: &str,
    db: &PgPool,
) -> Result<bool> {
    let limit = match plan {
        "free" => 50,
        "starter" => 500,
        "pro" => 5000,
        _ => 50,
    };
    
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM processing_history 
         WHERE user_id = $1 AND created_at > NOW() - INTERVAL '30 days'"
    )
    .bind(user_id)
    .fetch_one(db)
    .await?;
    
    Ok(count < limit as i64)
}
```

---

## UI Components Needed

### 1. File Upload Zone
```rust
#[component]
fn UploadZone(
    on_file: Callback<File>,
    max_size: usize,
) -> impl IntoView {
    view! {
        <div 
            class="upload-zone"
            on:drop=move |ev| {
                ev.prevent_default();
                // Handle drop
            }
            on:dragover=|ev| ev.prevent_default()
        >
            <input type="file" accept=".txt,.pdf,.docx" />
            <p>"Drop file or click to browse"</p>
            <p class="hint">"Max " {max_size / 1024 / 1024} "MB"</p>
        </div>
    }
}
```

### 2. Results Display
```rust
#[component]
fn ResultsDisplay(result: ProcessingResult) -> impl IntoView {
    view! {
        <div class="results">
            <div class="stats">
                <span>"Found: " {result.entities.len()} " entities"</span>
                <span>"Time: " {result.processing_time_ms} "ms"</span>
            </div>
            
            <div class="tabs">
                <Tab label="Detected PII">
                    <EntityTable entities=result.entities />
                </Tab>
                <Tab label="Redacted Text">
                    <RedactedText text=result.redacted_text />
                </Tab>
            </div>
        </div>
    }
}
```

### 3. Usage Meter (for logged-in users)
```rust
#[component]
fn UsageMeter(used: i32, limit: i32) -> impl IntoView {
    let percentage = (used as f32 / limit as f32) * 100.0;
    
    view! {
        <div class="usage-meter">
            <div class="bar">
                <div class="fill" style:width=move || format!("{}%", percentage)></div>
            </div>
            <span>{used} " / " {limit} " files this month"</span>
            <a href="/upgrade">"Upgrade"</a>
        </div>
    }
}
```

---

## Implementation Roadmap

### Week 1: Basic Playground
- [ ] Set up Leptos project
- [ ] File upload component
- [ ] Connect to existing `/api/v1/upload`
- [ ] Display results
- [ ] IP-based rate limiting

### Week 2-3: User Accounts
- [ ] Registration/login forms
- [ ] User database
- [ ] Session management
- [ ] Processing history
- [ ] Usage tracking

### Week 4-6: Enterprise Features
- [ ] Organization support
- [ ] Admin dashboard
- [ ] Plan management
- [ ] Billing integration (Stripe)
- [ ] Advanced analytics

---

## Recommended Starting Point

**For immediate user testing, I recommend:**

1. **Build the anonymous playground first** (Week 1)
   - Users can test immediately without signup
   - Validates the core UX
   - Low commitment for evaluators

2. **Add simple user accounts** (Week 2-3)
   - Email + password
   - Track usage for upgrade prompts
   - Build foundation for enterprise

3. **Enterprise features** (Later)
   - Based on actual usage patterns
   - Customer feedback driven

---

## Questions for You

1. **Priority:** Do you want the playground first, or full user management?
2. **Auth:** Email/password or social login (Google, GitHub)?
3. **UI Framework:** Leptos (Rust) or React (JavaScript)?
4. **Self-hosted:** Will customers deploy this themselves?

---

**Ready to start with the anonymous playground?**
