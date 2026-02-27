import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  LineChart,
  Line,
  AreaChart,
  Area,
} from 'recharts'
import {
  Activity,
  TrendingUp,
  AlertTriangle,
  RefreshCw,
  Loader2,
  Calendar,
} from 'lucide-react'
import { usageApi } from '../lib/api'
import { useToast } from '../hooks/useToast'
import { getErrorMessage } from '../lib/api-errors'
import { formatNumber, formatDate } from '../lib/utils'
import type { UsageSummary } from '../types'

export function UsagePage() {
  const [timeRange, setTimeRange] = useState<'7d' | '30d' | '90d'>('30d')
  const { showError } = useToast()

  const {
    data: usage,
    isLoading,
    error,
    isError,
    refetch,
  } = useQuery({
    queryKey: ['usage', timeRange],
    queryFn: async () => {
      try {
        const response = await usageApi.getSummary(timeRange)
        return response.data
      } catch (err) {
        showError(getErrorMessage(err, 'Failed to load usage data'))
        throw err
      }
    },
  })

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-900">Usage Analytics</h1>
        </div>
        <div className="flex items-center justify-center h-96">
          <Loader2 className="w-8 h-8 animate-spin text-primary-600" />
        </div>
      </div>
    )
  }

  if (isError) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-900">Usage Analytics</h1>
        </div>
        <div className="rounded-md bg-red-50 p-4">
          <div className="flex">
            <AlertTriangle className="h-5 w-5 text-red-400" aria-hidden="true" />
            <div className="ml-3">
              <h3 className="text-sm font-medium text-red-800">
                Failed to load usage data
              </h3>
              <p className="mt-2 text-sm text-red-700">
                {getErrorMessage(error, 'Please try again later.')}
              </p>
              <button
                onClick={() => refetch()}
                className="mt-3 inline-flex items-center text-sm font-medium text-red-800 hover:text-red-900"
              >
                <RefreshCw className="w-4 h-4 mr-1" />
                Try again
              </button>
            </div>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Usage Analytics</h1>
          <p className="mt-1 text-sm text-gray-500">
            Monitor your API usage and performance metrics
          </p>
        </div>
        <TimeRangeSelector value={timeRange} onChange={setTimeRange} />
      </div>

      {/* Stats cards */}
      {usage && <StatsCards summary={usage.summary} />}

      {/* Charts */}
      {usage && (
        <>
          <div className="bg-white shadow rounded-lg p-6">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-lg font-medium text-gray-900">API Requests Over Time</h3>
              <Activity className="h-5 w-5 text-gray-400" aria-hidden="true" />
            </div>
            <div className="h-80">
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={usage.dailyUsage}>
                  <defs>
                    <linearGradient id="colorRequests" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#4f46e5" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="#4f46e5" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis
                    dataKey="date"
                    tickFormatter={(date) => formatDate(date)}
                  />
                  <YAxis />
                  <Tooltip
                    labelFormatter={(label) => formatDate(label)}
                    formatter={(value: number) => [formatNumber(value), 'Requests']}
                  />
                  <Area
                    type="monotone"
                    dataKey="requests"
                    stroke="#4f46e5"
                    fillOpacity={1}
                    fill="url(#colorRequests)"
                    strokeWidth={2}
                  />
                </AreaChart>
              </ResponsiveContainer>
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div className="bg-white shadow rounded-lg p-6">
              <div className="flex items-center justify-between mb-6">
                <h3 className="text-lg font-medium text-gray-900">Documents Processed</h3>
                <TrendingUp className="h-5 w-5 text-gray-400" aria-hidden="true" />
              </div>
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={usage.dailyUsage}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis
                      dataKey="date"
                      tickFormatter={(date) => formatDate(date)}
                    />
                    <YAxis />
                    <Tooltip
                      labelFormatter={(label) => formatDate(label)}
                      formatter={(value: number) => [formatNumber(value), 'Documents']}
                    />
                    <Bar dataKey="documents" fill="#4f46e5" radius={[4, 4, 0, 0]} />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </div>

            <div className="bg-white shadow rounded-lg p-6">
              <div className="flex items-center justify-between mb-6">
                <h3 className="text-lg font-medium text-gray-900">Response Time (ms)</h3>
                <Activity className="h-5 w-5 text-gray-400" aria-hidden="true" />
              </div>
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={usage.dailyUsage}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis
                      dataKey="date"
                      tickFormatter={(date) => formatDate(date)}
                    />
                    <YAxis />
                    <Tooltip
                      labelFormatter={(label) => formatDate(label)}
                      formatter={(value: number) => [value.toFixed(0), 'ms']}
                    />
                    <Line
                      type="monotone"
                      dataKey="avgResponseTime"
                      stroke="#10b981"
                      strokeWidth={2}
                      dot={false}
                    />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            </div>
          </div>
        </>
      )}

      {/* Quota Alert */}
      {usage?.summary && usage.summary.quotaUsage > 80 && (
        <div
          className={`rounded-md p-4 ${
            usage.summary.quotaUsage >= 90
              ? 'bg-red-50 border border-red-200'
              : 'bg-yellow-50 border border-yellow-200'
          }`}
          role="alert"
        >
          <div className="flex">
            <AlertTriangle
              className={`h-5 w-5 ${
                usage.summary.quotaUsage >= 90 ? 'text-red-400' : 'text-yellow-400'
              }`}
              aria-hidden="true"
            />
            <div className="ml-3">
              <h3
                className={`text-sm font-medium ${
                  usage.summary.quotaUsage >= 90 ? 'text-red-800' : 'text-yellow-800'
                }`}
              >
                {usage.summary.quotaUsage >= 90
                  ? 'Critical: Quota nearly exhausted'
                  : 'Warning: Approaching quota limit'}
              </h3>
              <p
                className={`mt-2 text-sm ${
                  usage.summary.quotaUsage >= 90 ? 'text-red-700' : 'text-yellow-700'
                }`}
              >
                You have used {usage.summary.quotaUsage.toFixed(1)}% of your monthly
                quota. Consider upgrading your plan to avoid service interruption.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

function TimeRangeSelector({
  value,
  onChange,
}: {
  value: '7d' | '30d' | '90d'
  onChange: (value: '7d' | '30d' | '90d') => void
}) {
  const options: Array<{ value: '7d' | '30d' | '90d'; label: string }> = [
    { value: '7d', label: 'Last 7 days' },
    { value: '30d', label: 'Last 30 days' },
    { value: '90d', label: 'Last 90 days' },
  ]

  return (
    <div className="flex items-center space-x-2">
      <Calendar className="w-5 h-5 text-gray-400" aria-hidden="true" />
      <label htmlFor="time-range" className="sr-only">
        Select time range
      </label>
      <select
        id="time-range"
        value={value}
        onChange={(e) => onChange(e.target.value as typeof value)}
        className="block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-primary-500 focus:border-primary-500 sm:text-sm rounded-md"
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </div>
  )
}

function StatsCards({ summary }: { summary: UsageSummary }) {
  const stats = [
    {
      label: 'Total Requests',
      value: formatNumber(summary.totalRequests),
      change: summary.requestsChange,
      trend: summary.requestsChange >= 0 ? 'up' : 'down',
    },
    {
      label: 'Documents Processed',
      value: formatNumber(summary.totalDocuments),
      change: summary.documentsChange,
      trend: summary.documentsChange >= 0 ? 'up' : 'down',
    },
    {
      label: 'Success Rate',
      value: `${summary.successRate.toFixed(1)}%`,
      change: summary.successRateChange,
      trend: summary.successRateChange >= 0 ? 'up' : 'down',
    },
    {
      label: 'Quota Usage',
      value: `${summary.quotaUsage.toFixed(1)}%`,
      change: summary.quotaUsageChange,
      trend: summary.quotaUsageChange > 0 ? 'up' : 'down',
    },
  ]

  return (
    <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
      {stats.map((stat) => (
        <div key={stat.label} className="bg-white overflow-hidden shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <dt className="text-sm font-medium text-gray-500 truncate">{stat.label}</dt>
            <dd className="mt-1 text-3xl font-semibold text-gray-900">{stat.value}</dd>
            {stat.change !== undefined && (
              <div className="mt-2 flex items-center text-sm">
                <span
                  className={`font-medium ${
                    stat.trend === 'up' ? 'text-green-600' : 'text-red-600'
                  }`}
                >
                  {stat.trend === 'up' ? '+' : ''}
                  {stat.change.toFixed(1)}%
                </span>
                <span className="ml-2 text-gray-500">vs last period</span>
              </div>
            )}
          </div>
        </div>
      ))}
    </div>
  )
}
