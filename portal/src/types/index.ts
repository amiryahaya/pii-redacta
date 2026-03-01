export interface User {
  id: string
  email: string
  displayName: string | null
  companyName: string | null
  emailNotificationsEnabled: boolean
  isAdmin: boolean
  createdAt: string
}

export interface ApiKey {
  id: string
  name: string
  keyPrefix: string
  environment: 'live' | 'test'
  lastUsedAt: string | null
  expiresAt: string | null
  isActive: boolean
  createdAt: string
}

export interface GeneratedApiKey extends ApiKey {
  fullKey: string
}

export interface Tier {
  id: string
  name: string
  displayName: string
  description: string | null
  limits: TierLimits
  features: TierFeatures
  monthlyPriceCents: number | null
  yearlyPriceCents: number | null
}

export interface TierLimits {
  apiEnabled: boolean
  maxApiKeys: number | null
  maxFileSize: number | null
  maxFilesPerMonth: number | null
  maxPagesPerFile: number | null
  maxTotalSize: number | null
  playgroundMaxDaily: number | null
  playgroundMaxFileSize: number | null
  retentionDays: number | null
  maxBatchItems: number | null
  maxWebhookEndpoints: number | null
  maxCustomRules: number | null
}

export interface TierFeatures {
  batchProcessing: boolean
  customRules: boolean
  emailSupport: boolean
  playground: boolean
  rateLimitPerMinute: number | null
  sla: string | null
  webhooks: boolean
}

export interface Subscription {
  id: string
  status: 'trial' | 'active' | 'past_due' | 'cancelled' | 'expired'
  tier: Tier
  currentPeriodStart: string | null
  currentPeriodEnd: string | null
  cancelAtPeriodEnd: boolean
}

export interface UsageStats {
  totalRequests: number
  totalFiles: number
  totalPages: number
  storageUsed: number
  monthlyFiles: number
  monthlyLimit: number | null
}

export interface DailyUsage {
  date: string
  requests: number
  files: number
  pages: number
}

export interface UsageSummary {
  monthlyRequests: number
  monthlyDocuments: number
  quotaUsage: number
  quotaUsageChange: number
  requestsChange: number
  documentsChange: number
}

export interface LoginCredentials {
  email: string
  password: string
}

export interface RegisterData {
  email: string
  password: string
  displayName?: string
  companyName?: string
}

export interface CreateApiKeyData {
  name: string
  environment: 'live' | 'test'
  expiresAt?: string | null
}

export interface UserPreferences {
  emailQuotaAlert: boolean
  emailSecurityAlert: boolean
  emailMarketing: boolean
  emailMonthlyReport: boolean
}

export interface PlaygroundEntity {
  entity_type: string
  value: string
  start: number
  end: number
  confidence?: number
}

export interface PlaygroundResponse {
  entities: PlaygroundEntity[]
  processingTimeMs: number
  redactedText?: string
  textLength: number
  dailyUsage: {
    usedToday: number
    dailyLimit: number | null
  }
}

export interface PlaygroundHistoryEntry {
  id: string
  requestType: string
  fileName: string | null
  fileType: string | null
  detectionsCount: number | null
  processingTimeMs: number | null
  success: boolean
  createdAt: string
}

export interface DashboardStats {
  stats: {
    monthlyRequests: number
    monthlyDocuments: number
    quotaUsage: number
    documentsChange: number
    requestsChange: number
  }
  charts: {
    dailyRequests: Array<{ date: string; value: number }>
    dailyDocuments: Array<{ date: string; value: number }>
  }
  recentActivity: Array<{
    id: string
    type: 'document' | 'api'
    description: string
    timestamp: string
  }>
}

// Custom Rules
export interface CustomRule {
  id: string
  name: string
  description: string | null
  pattern: string
  entityLabel: string
  confidence: number
  isActive: boolean
  createdAt: string
  updatedAt: string
}

export interface RuleTestResult {
  matches: Array<{
    value: string
    start: number
    end: number
    entityLabel: string
    confidence: number
  }>
  processingTimeMs: number
}

// Batch Processing
export interface BatchJob {
  id: string
  status: string
  totalItems: number
  completedItems: number
  failedItems: number
  redact: boolean
  useCustomRules: boolean
  createdAt: string
  completedAt: string | null
}

export interface BatchResultItem {
  itemIndex: number
  status: string
  entities: PlaygroundEntity[] | null
  redactedText: string | null
  processingTimeMs: number | null
  errorMessage: string | null
}

// Webhooks
export interface WebhookEndpoint {
  id: string
  url: string
  description: string | null
  secret: string
  events: string[]
  isActive: boolean
  failureCount: number
  lastTriggeredAt: string | null
  createdAt: string
}

export interface WebhookDelivery {
  id: string
  eventType: string
  status: string
  httpStatus: number | null
  attempts: number
  createdAt: string
  deliveredAt: string | null
}
