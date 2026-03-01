import { useState } from 'react'
import { useQuery, useMutation } from '@tanstack/react-query'
import {
  Play,
  Loader2,
  Lock,
  CheckCircle,
  XCircle,
  Clock,
  ChevronDown,
  ChevronUp,
} from 'lucide-react'
import { batchApi, subscriptionApi } from '../lib/api'
import { useToast } from '../hooks/useToast'
import { getErrorMessage } from '../lib/api-errors'
import type { BatchJob, BatchResultItem } from '../types'

export function BatchPage() {
  const { showSuccess, showError } = useToast()
  const [inputText, setInputText] = useState('')
  const [redact, setRedact] = useState(false)
  const [useCustomRules, setUseCustomRules] = useState(false)
  const [activeBatchId, setActiveBatchId] = useState<string | null>(null)
  const [expandedItems, setExpandedItems] = useState<Set<number>>(new Set())

  const { data: subscription } = useQuery({
    queryKey: ['subscription'],
    queryFn: () => subscriptionApi.get().then((r) => r.data),
  })

  const hasFeature = subscription?.tier?.features?.batchProcessing ?? false

  const submitMutation = useMutation({
    mutationFn: batchApi.submit,
    onSuccess: (response) => {
      setActiveBatchId(response.data.id)
      showSuccess('Batch job submitted')
    },
    onError: (err: unknown) => {
      showError(getErrorMessage(err, 'Failed to submit batch'))
    },
  })

  // Poll batch status while active
  const { data: batchStatus } = useQuery<BatchJob>({
    queryKey: ['batch-status', activeBatchId],
    queryFn: () => batchApi.getStatus(activeBatchId!).then((r) => r.data),
    enabled: !!activeBatchId,
    refetchInterval: (query) => {
      const status = query.state.data?.status
      if (status === 'completed' || status === 'failed' || status === 'partial') {
        return false
      }
      return 1000
    },
  })

  // Load results when batch completes
  const isComplete =
    batchStatus?.status === 'completed' ||
    batchStatus?.status === 'failed' ||
    batchStatus?.status === 'partial'

  const { data: batchResults } = useQuery<BatchResultItem[]>({
    queryKey: ['batch-results', activeBatchId],
    queryFn: () => batchApi.getResults(activeBatchId!).then((r) => r.data),
    enabled: !!activeBatchId && isComplete,
  })

  const handleSubmit = () => {
    const items = inputText
      .split('\n')
      .map((line) => line.trim())
      .filter((line) => line.length > 0)

    if (items.length === 0) {
      showError('Please enter at least one text item')
      return
    }

    submitMutation.mutate({ items, redact, useCustomRules })
  }

  const toggleExpanded = (index: number) => {
    setExpandedItems((prev) => {
      const next = new Set(prev)
      if (next.has(index)) {
        next.delete(index)
      } else {
        next.add(index)
      }
      return next
    })
  }

  const statusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircle className="w-4 h-4 text-green-500" />
      case 'failed':
        return <XCircle className="w-4 h-4 text-red-500" />
      case 'processing':
        return <Loader2 className="w-4 h-4 text-blue-500 animate-spin" />
      default:
        return <Clock className="w-4 h-4 text-gray-400" />
    }
  }

  // Tier gate
  if (!hasFeature) {
    return (
      <div className="text-center py-16">
        <Lock className="w-12 h-12 text-gray-400 mx-auto mb-4" />
        <h2 className="text-xl font-semibold text-gray-900 mb-2">
          Batch Processing
        </h2>
        <p className="text-gray-500 max-w-md mx-auto">
          Batch processing is available on Pro and Enterprise plans. Upgrade to
          process multiple text items in a single job.
        </p>
      </div>
    )
  }

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Batch Processing</h1>
        <p className="mt-1 text-sm text-gray-500">
          Submit multiple text items for PII detection in a single batch
        </p>
      </div>

      {/* Input Section */}
      <div className="bg-white rounded-lg border border-gray-200 p-6 mb-6">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Text Items (one per line)
        </label>
        <textarea
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
          placeholder="Enter text items, one per line...&#10;e.g., My email is test@example.com&#10;Call me at +60123456789"
          className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm h-40 font-mono"
        />

        <div className="flex items-center gap-6 mt-4">
          <label className="flex items-center text-sm">
            <input
              type="checkbox"
              checked={redact}
              onChange={(e) => setRedact(e.target.checked)}
              className="rounded border-gray-300 text-primary-600 mr-2"
            />
            Redact detected PII
          </label>
          <label className="flex items-center text-sm">
            <input
              type="checkbox"
              checked={useCustomRules}
              onChange={(e) => setUseCustomRules(e.target.checked)}
              className="rounded border-gray-300 text-primary-600 mr-2"
            />
            Use custom rules
          </label>
        </div>

        <button
          onClick={handleSubmit}
          disabled={!inputText.trim() || submitMutation.isPending}
          className="mt-4 inline-flex items-center px-4 py-2 bg-primary-600 text-white text-sm font-medium rounded-md hover:bg-primary-700 disabled:opacity-50"
        >
          {submitMutation.isPending ? (
            <Loader2 className="w-4 h-4 mr-2 animate-spin" />
          ) : (
            <Play className="w-4 h-4 mr-2" />
          )}
          Submit Batch
        </button>
      </div>

      {/* Status Section */}
      {batchStatus && (
        <div className="bg-white rounded-lg border border-gray-200 p-6 mb-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-medium text-gray-900">
              Batch Status
            </h2>
            <div className="flex items-center gap-2">
              {statusIcon(batchStatus.status)}
              <span className="text-sm font-medium capitalize">
                {batchStatus.status}
              </span>
            </div>
          </div>

          {/* Progress bar */}
          <div className="w-full bg-gray-200 rounded-full h-2 mb-2">
            <div
              className={`h-2 rounded-full transition-all ${
                batchStatus.status === 'failed'
                  ? 'bg-red-500'
                  : batchStatus.status === 'partial'
                  ? 'bg-yellow-500'
                  : 'bg-primary-600'
              }`}
              style={{
                width: `${
                  batchStatus.totalItems > 0
                    ? ((batchStatus.completedItems + batchStatus.failedItems) /
                        batchStatus.totalItems) *
                      100
                    : 0
                }%`,
              }}
            />
          </div>
          <p className="text-sm text-gray-500">
            {batchStatus.completedItems + batchStatus.failedItems} /{' '}
            {batchStatus.totalItems} items processed
            {batchStatus.failedItems > 0 &&
              ` (${batchStatus.failedItems} failed)`}
          </p>
        </div>
      )}

      {/* Results Section */}
      {batchResults && batchResults.length > 0 && (
        <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200">
            <h2 className="text-lg font-medium text-gray-900">Results</h2>
          </div>
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  #
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Entities
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Time
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">
                  Details
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {batchResults.map((item) => (
                <>
                  <tr key={item.itemIndex}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {item.itemIndex + 1}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center gap-1">
                        {statusIcon(item.status)}
                        <span className="text-sm capitalize">{item.status}</span>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {item.entities
                        ? Array.isArray(item.entities)
                          ? item.entities.length
                          : 0
                        : '-'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {item.processingTimeMs != null
                        ? `${item.processingTimeMs}ms`
                        : '-'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-right">
                      <button
                        onClick={() => toggleExpanded(item.itemIndex)}
                        className="text-primary-600 hover:text-primary-800"
                      >
                        {expandedItems.has(item.itemIndex) ? (
                          <ChevronUp className="w-4 h-4 inline" />
                        ) : (
                          <ChevronDown className="w-4 h-4 inline" />
                        )}
                      </button>
                    </td>
                  </tr>
                  {expandedItems.has(item.itemIndex) && (
                    <tr key={`${item.itemIndex}-detail`}>
                      <td colSpan={5} className="px-6 py-4 bg-gray-50">
                        {item.errorMessage && (
                          <p className="text-sm text-red-600 mb-2">
                            Error: {item.errorMessage}
                          </p>
                        )}
                        {item.redactedText && (
                          <div className="mb-2">
                            <p className="text-xs font-medium text-gray-500 mb-1">
                              Redacted Text:
                            </p>
                            <pre className="text-sm bg-white p-2 rounded border text-gray-800">
                              {item.redactedText}
                            </pre>
                          </div>
                        )}
                        {item.entities && Array.isArray(item.entities) && item.entities.length > 0 && (
                          <div>
                            <p className="text-xs font-medium text-gray-500 mb-1">
                              Entities:
                            </p>
                            <div className="space-y-1">
                              {item.entities.map((e, i) => (
                                <div
                                  key={i}
                                  className="inline-flex items-center bg-yellow-50 border border-yellow-200 rounded px-2 py-1 text-xs mr-2"
                                >
                                  <span className="font-medium">
                                    {e.entity_type}
                                  </span>
                                  <span className="text-gray-500 ml-1">
                                    {e.value}
                                  </span>
                                </div>
                              ))}
                            </div>
                          </div>
                        )}
                      </td>
                    </tr>
                  )}
                </>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
