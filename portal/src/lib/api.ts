import axios from 'axios'
import { useAuthStore } from '../stores/authStore'
import type {
  User,
  ApiKey,
  GeneratedApiKey,
  Subscription,
  UsageStats,
  DailyUsage,
  LoginCredentials,
  RegisterData,
  CreateApiKeyData,
  Tier,
  UserPreferences,
  DashboardStats,
  UsageSummary,
} from '../types'

const api = axios.create({
  baseURL: '/api/v1',
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor to add auth token
api.interceptors.request.use((config) => {
  const token = useAuthStore.getState().token
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

// Response interceptor to handle auth errors
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      useAuthStore.getState().clearAuth()
      window.location.href = '/login'
    }
    return Promise.reject(error)
  }
)

// Auth API
export const authApi = {
  login: (credentials: LoginCredentials) =>
    api.post<{ user: User; token: string }>('/auth/login', credentials),

  register: (data: RegisterData) =>
    api.post<{ user: User; token: string }>('/auth/register', data),

  logout: () => api.post('/auth/logout'),

  me: () => api.get<User>('/auth/me'),

  changePassword: (data: { currentPassword: string; newPassword: string }) =>
    api.post('/auth/change-password', data),
}

// API Keys API
export const apiKeysApi = {
  list: () => api.get<ApiKey[]>('/api-keys'),

  create: (data: CreateApiKeyData) =>
    api.post<GeneratedApiKey>('/api-keys', data),

  revoke: (id: string, reason?: string) =>
    api.post(`/api-keys/${id}/revoke`, { reason }),
}

// Subscription API
export const subscriptionApi = {
  get: () => api.get<Subscription>('/subscription'),

  listTiers: () => api.get<Tier[]>('/tiers'),
}

// Usage API
export const usageApi = {
  getStats: () => api.get<UsageStats>('/usage/stats'),

  getDaily: (days: number = 30) =>
    api.get<DailyUsage[]>('/usage/daily', { params: { days } }),

  getSummary: (range: string) =>
    api.get<{
      summary: UsageSummary
      dailyUsage: DailyUsage[]
    }>('/usage/summary', { params: { range } }),
}

// User API
export const userApi = {
  updateProfile: (data: { displayName?: string; companyName?: string }) =>
    api.patch<User>('/users/profile', data),

  updatePreferences: (data: UserPreferences) =>
    api.patch<UserPreferences>('/users/preferences', data),

  getPreferences: () => api.get<UserPreferences>('/users/preferences'),
}

// Dashboard API
export const dashboardApi = {
  getStats: () => api.get<DashboardStats>('/dashboard/stats'),
}

export default api
