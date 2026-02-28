import { useQuery } from '@tanstack/react-query'
import {
  FileText,
  Key,
  BarChart3,
  AlertTriangle,
  ArrowRight,
  Loader2,
  RefreshCw,
} from 'lucide-react'
import { Link } from 'react-router-dom'
import { LineChart, Line, ResponsiveContainer, Tooltip } from 'recharts'
import { dashboardApi, apiKeysApi } from '../lib/api'
import { formatNumber, formatDate } from '../lib/utils'
import { useToast } from '../hooks/useToast'
import { getErrorMessage } from '../lib/api-errors'
import type { ApiKey, DashboardStats } from '../types'

export function DashboardPage() {
  const { showError } = useToast()

  const {
    data: dashboard,
    isLoading: isDashboardLoading,
    error: dashboardError,
    isError: isDashboardError,
    refetch: refetchDashboard,
  } = useQuery<DashboardStats>({
    queryKey: ['dashboard'],
    queryFn: async () => {
      try {
        const response = await dashboardApi.getStats()
        return response.data
      } catch (err) {
        showError(getErrorMessage(err, 'Failed to load dashboard'))
        throw err
      }
    },
  })

  const {
    data: apiKeys,
    isLoading: isKeysLoading,
    error: keysError,
    isError: isKeysError,
    refetch: refetchKeys,
  } = useQuery<ApiKey[]>({
    queryKey: ['api-keys-dashboard'],
    queryFn: () => apiKeysApi.list().then((r) => r.data.slice(0, 3)),
  })

  const isLoading = isDashboardLoading || isKeysLoading
  const isError = isDashboardError || isKeysError

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-96">
        <Loader2 className="w-8 h-8 animate-spin text-primary-600" />
      </div>
    )
  }

  if (isError) {
    return (
      <div className="rounded-md bg-red-50 p-4">
        <div className="flex">
          <AlertTriangle className="h-5 w-5 text-red-400" aria-hidden="true" />
          <div className="ml-3">
            <h3 className="text-sm font-medium text-red-800">
              Failed to load dashboard
            </h3>
            <p className="mt-2 text-sm text-red-700">
              {getErrorMessage(dashboardError || keysError, 'Please try again.')}
            </p>
            <button
              onClick={() => {
                refetchDashboard()
                refetchKeys()
              }}
              className="mt-3 inline-flex items-center text-sm font-medium text-red-800 hover:text-red-900"
            >
              <RefreshCw className="w-4 h-4 mr-1" />
              Try again
            </button>
          </div>
        </div>
      </div>
    )
  }

  const stats = [
    {
      name: 'Documents Processed',
      value: dashboard ? formatNumber(dashboard.stats.monthlyDocuments) : '0',
      icon: FileText,
      change: dashboard?.stats.documentsChange ?? 0,
      trend: (dashboard?.stats.documentsChange ?? 0) >= 0 ? 'up' : 'down',
      href: '/usage',
    },
    {
      name: 'API Requests',
      value: dashboard ? formatNumber(dashboard.stats.monthlyRequests) : '0',
      icon: BarChart3,
      change: dashboard?.stats.requestsChange ?? 0,
      trend: (dashboard?.stats.requestsChange ?? 0) >= 0 ? 'up' : 'down',
      href: '/usage',
    },
    {
      name: 'Active API Keys',
      value: apiKeys?.length ?? 0,
      icon: Key,
      href: '/api-keys',
    },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
        <p className="mt-1 text-sm text-gray-500">
          Overview of your PII Redaction API usage
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-3">
        {stats.map((stat) => (
          <div
            key={stat.name}
            className="bg-white overflow-hidden shadow rounded-lg hover:shadow-md transition-shadow"
          >
            <div className="p-5">
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  <stat.icon
                    className="h-6 w-6 text-gray-400"
                    aria-hidden="true"
                  />
                </div>
                <div className="ml-5 w-0 flex-1">
                  <dl>
                    <dt className="text-sm font-medium text-gray-500 truncate">
                      {stat.name}
                    </dt>
                    <dd className="flex items-baseline">
                      <div className="text-2xl font-semibold text-gray-900">
                        {stat.value}
                      </div>
                      {stat.change !== undefined && stat.change !== 0 && (
                        <div
                          className={`ml-2 flex items-baseline text-sm font-semibold ${
                            stat.trend === 'up'
                              ? 'text-green-600'
                              : 'text-red-600'
                          }`}
                        >
                          {stat.trend === 'up' ? '↑' : '↓'}
                          <span className="sr-only">
                            {stat.trend === 'up' ? 'Increased' : 'Decreased'} by
                          </span>
                          {Math.abs(stat.change)}%
                        </div>
                      )}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
            <div className="bg-gray-50 px-5 py-3">
              <div className="text-sm">
                <Link
                  to={stat.href}
                  className="font-medium text-primary-600 hover:text-primary-500 inline-flex items-center"
                >
                  View details
                  <ArrowRight className="ml-1 w-4 h-4" aria-hidden="true" />
                </Link>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Charts */}
      {dashboard?.charts && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="bg-white shadow rounded-lg p-6">
            <h3 className="text-lg font-medium text-gray-900 mb-4">
              Daily Requests
            </h3>
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={dashboard.charts.dailyRequests}>
                  <Tooltip
                    contentStyle={{
                      backgroundColor: 'white',
                      border: '1px solid #e5e7eb',
                      borderRadius: '6px',
                    }}
                    labelFormatter={(label) => formatDate(label)}
                  />
                  <Line
                    type="monotone"
                    dataKey="value"
                    stroke="#4f46e5"
                    strokeWidth={2}
                    dot={false}
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
          </div>

          <div className="bg-white shadow rounded-lg p-6">
            <h3 className="text-lg font-medium text-gray-900 mb-4">
              Daily Documents
            </h3>
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={dashboard.charts.dailyDocuments}>
                  <Tooltip
                    contentStyle={{
                      backgroundColor: 'white',
                      border: '1px solid #e5e7eb',
                      borderRadius: '6px',
                    }}
                    labelFormatter={(label) => formatDate(label)}
                  />
                  <Line
                    type="monotone"
                    dataKey="value"
                    stroke="#10b981"
                    strokeWidth={2}
                    dot={false}
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
          </div>
        </div>
      )}

      {/* Recent Activity and API Keys */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Recent Activity */}
        <div className="bg-white shadow rounded-lg">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">Recent Activity</h3>
          </div>
          <div className="divide-y divide-gray-200">
            {dashboard?.recentActivity?.length === 0 ? (
              <div className="px-6 py-8 text-center">
                <p className="text-sm text-gray-500">No recent activity</p>
              </div>
            ) : (
              dashboard?.recentActivity?.map((activity) => (
                <div key={activity.id} className="px-6 py-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-sm font-medium text-gray-900">
                        {activity.type === 'document' ? 'Document processed' : 'API request'}
                      </p>
                      <p className="text-sm text-gray-500">
                        {activity.description}
                      </p>
                    </div>
                    <p className="text-xs text-gray-500">
                      {formatDate(activity.timestamp)}
                    </p>
                  </div>
                </div>
              ))
            )}
          </div>
          {dashboard?.recentActivity && dashboard.recentActivity.length > 0 && (
            <div className="px-6 py-3 bg-gray-50 text-center">
              <Link
                to="/usage"
                className="text-sm font-medium text-primary-600 hover:text-primary-500"
              >
                View all activity
              </Link>
            </div>
          )}
        </div>

        {/* API Keys */}
        <div className="bg-white shadow rounded-lg">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">Active API Keys</h3>
          </div>
          <div className="divide-y divide-gray-200">
            {apiKeys?.length === 0 ? (
              <div className="px-6 py-8 text-center">
                <p className="text-sm text-gray-500">No API keys created yet</p>
              </div>
            ) : (
              apiKeys?.map((key) => (
                <div key={key.id} className="px-6 py-4 flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-900">{key.name}</p>
                    <p className="text-xs text-gray-500">
                      Last used: {formatDate(key.lastUsedAt)}
                    </p>
                  </div>
                  <span
                    className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      key.environment === 'live'
                        ? 'bg-green-100 text-green-800'
                        : 'bg-yellow-100 text-yellow-800'
                    }`}
                  >
                    {key.environment}
                  </span>
                </div>
              ))
            )}
          </div>
          <div className="px-6 py-3 bg-gray-50 text-center">
            <Link
              to="/api-keys"
              className="text-sm font-medium text-primary-600 hover:text-primary-500"
            >
              Manage API keys
            </Link>
          </div>
        </div>
      </div>

      {/* Quota warning */}
      {dashboard?.stats.quotaUsage && dashboard.stats.quotaUsage > 80 && (
        <div
          className={`rounded-md p-4 ${
            dashboard.stats.quotaUsage >= 90
              ? 'bg-red-50 border border-red-200'
              : 'bg-yellow-50 border border-yellow-200'
          }`}
          role="alert"
        >
          <div className="flex">
            <AlertTriangle
              className={`h-5 w-5 ${
                dashboard.stats.quotaUsage >= 90
                  ? 'text-red-400'
                  : 'text-yellow-400'
              }`}
              aria-hidden="true"
            />
            <div className="ml-3">
              <h3
                className={`text-sm font-medium ${
                  dashboard.stats.quotaUsage >= 90
                    ? 'text-red-800'
                    : 'text-yellow-800'
                }`}
              >
                {dashboard.stats.quotaUsage >= 90
                  ? 'Critical: Quota nearly exhausted'
                  : 'Warning: Approaching quota limit'}
              </h3>
              <p
                className={`mt-2 text-sm ${
                  dashboard.stats.quotaUsage >= 90
                    ? 'text-red-700'
                    : 'text-yellow-700'
                }`}
              >
                You have used {dashboard.stats.quotaUsage.toFixed(1)}% of your
                monthly quota. Consider upgrading your plan to avoid service
                interruption.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
