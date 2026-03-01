import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  Webhook,
  Plus,
  Trash2,
  Play,
  Loader2,
  Lock,
  X,
  CheckCircle,
  XCircle,
  Clock,
  ChevronDown,
  ChevronUp,
  AlertTriangle,
  Copy,
} from 'lucide-react'
import { webhooksApi, subscriptionApi } from '../lib/api'
import { useToast } from '../hooks/useToast'
import { getErrorMessage } from '../lib/api-errors'
import type { WebhookEndpoint, WebhookDelivery } from '../types'

const EVENT_TYPES = [
  { value: 'detection.completed', label: 'Detection Completed' },
  { value: 'batch.completed', label: 'Batch Completed' },
  { value: 'batch.failed', label: 'Batch Failed' },
]

export function WebhooksPage() {
  const queryClient = useQueryClient()
  const { showSuccess, showError } = useToast()
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [deletingId, setDeletingId] = useState<string | null>(null)
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const [createdSecret, setCreatedSecret] = useState<string | null>(null)

  // Form state
  const [formUrl, setFormUrl] = useState('')
  const [formDescription, setFormDescription] = useState('')
  const [formEvents, setFormEvents] = useState<Set<string>>(new Set())

  const { data: subscription } = useQuery({
    queryKey: ['subscription'],
    queryFn: () => subscriptionApi.get().then((r) => r.data),
  })

  const hasFeature = subscription?.tier?.features?.webhooks ?? false

  const {
    data: webhooks,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: ['webhooks'],
    queryFn: () => webhooksApi.list().then((r) => r.data),
    enabled: hasFeature,
  })

  const { data: deliveries } = useQuery<WebhookDelivery[]>({
    queryKey: ['webhook-deliveries', expandedId],
    queryFn: () => webhooksApi.deliveries(expandedId!).then((r) => r.data),
    enabled: !!expandedId,
  })

  const createMutation = useMutation({
    mutationFn: webhooksApi.create,
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: ['webhooks'] })
      setCreatedSecret(response.data.secret)
      showSuccess('Webhook created')
      setShowCreateModal(false)
      resetForm()
    },
    onError: (err: unknown) => {
      showError(getErrorMessage(err, 'Failed to create webhook'))
    },
  })

  const deleteMutation = useMutation({
    mutationFn: webhooksApi.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['webhooks'] })
      showSuccess('Webhook deleted')
      setDeletingId(null)
    },
    onError: (err: unknown) => {
      showError(getErrorMessage(err, 'Failed to delete webhook'))
      setDeletingId(null)
    },
  })

  const testMutation = useMutation({
    mutationFn: webhooksApi.test,
    onSuccess: () => {
      showSuccess('Test event sent')
    },
    onError: (err: unknown) => {
      showError(getErrorMessage(err, 'Failed to send test event'))
    },
  })

  const resetForm = () => {
    setFormUrl('')
    setFormDescription('')
    setFormEvents(new Set())
  }

  const toggleEvent = (event: string) => {
    setFormEvents((prev) => {
      const next = new Set(prev)
      if (next.has(event)) {
        next.delete(event)
      } else {
        next.add(event)
      }
      return next
    })
  }

  const handleCreate = () => {
    if (!formUrl || formEvents.size === 0) return
    createMutation.mutate({
      url: formUrl,
      description: formDescription || undefined,
      events: Array.from(formEvents),
    })
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
    showSuccess('Copied to clipboard')
  }

  const statusBadge = (wh: WebhookEndpoint) => {
    if (!wh.isActive) {
      return (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
          Disabled
        </span>
      )
    }
    if (wh.failureCount > 0) {
      return (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
          {wh.failureCount} failures
        </span>
      )
    }
    return (
      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
        Active
      </span>
    )
  }

  const deliveryStatusIcon = (status: string) => {
    switch (status) {
      case 'delivered':
        return <CheckCircle className="w-4 h-4 text-green-500" />
      case 'failed':
        return <XCircle className="w-4 h-4 text-red-500" />
      default:
        return <Clock className="w-4 h-4 text-gray-400" />
    }
  }

  // Tier gate
  if (!hasFeature) {
    return (
      <div className="text-center py-16">
        <Lock className="w-12 h-12 text-gray-400 mx-auto mb-4" />
        <h2 className="text-xl font-semibold text-gray-900 mb-2">Webhooks</h2>
        <p className="text-gray-500 max-w-md mx-auto">
          Webhooks are available on Pro and Enterprise plans. Upgrade to receive
          real-time notifications when PII detection events occur.
        </p>
      </div>
    )
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader2 className="w-8 h-8 animate-spin text-primary-600" />
      </div>
    )
  }

  if (isError) {
    return (
      <div className="rounded-md bg-red-50 p-4">
        <div className="flex">
          <AlertTriangle className="h-5 w-5 text-red-400" />
          <div className="ml-3">
            <h3 className="text-sm font-medium text-red-800">
              Failed to load webhooks
            </h3>
            <p className="mt-2 text-sm text-red-700">
              {getErrorMessage(error, 'Please try again later.')}
            </p>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Webhooks</h1>
          <p className="mt-1 text-sm text-gray-500">
            Receive real-time notifications for PII detection events
          </p>
        </div>
        <button
          onClick={() => {
            resetForm()
            setShowCreateModal(true)
          }}
          className="inline-flex items-center px-4 py-2 bg-primary-600 text-white text-sm font-medium rounded-md hover:bg-primary-700"
        >
          <Plus className="w-4 h-4 mr-2" />
          New Webhook
        </button>
      </div>

      {/* Secret display after creation */}
      {createdSecret && (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
          <div className="flex items-start justify-between">
            <div>
              <h3 className="text-sm font-medium text-yellow-800">
                Webhook Secret
              </h3>
              <p className="text-sm text-yellow-700 mt-1">
                Save this secret now. It won't be shown again.
              </p>
              <div className="mt-2 flex items-center gap-2">
                <code className="text-sm bg-white px-3 py-1 rounded border border-yellow-300 font-mono">
                  {createdSecret}
                </code>
                <button
                  onClick={() => copyToClipboard(createdSecret)}
                  className="text-yellow-700 hover:text-yellow-900"
                >
                  <Copy className="w-4 h-4" />
                </button>
              </div>
            </div>
            <button onClick={() => setCreatedSecret(null)}>
              <X className="w-5 h-5 text-yellow-500" />
            </button>
          </div>
        </div>
      )}

      {/* Webhooks list */}
      {webhooks && webhooks.length === 0 ? (
        <div className="text-center py-12 bg-white rounded-lg border border-gray-200">
          <Webhook className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900">
            No webhooks configured
          </h3>
          <p className="mt-2 text-sm text-gray-500">
            Create your first webhook endpoint to receive event notifications.
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {webhooks?.map((wh) => (
            <div
              key={wh.id}
              className="bg-white rounded-lg border border-gray-200 overflow-hidden"
            >
              <div className="px-6 py-4">
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3">
                      <p className="text-sm font-medium text-gray-900 truncate">
                        {wh.url}
                      </p>
                      {statusBadge(wh)}
                    </div>
                    {wh.description && (
                      <p className="text-sm text-gray-500 mt-1">
                        {wh.description}
                      </p>
                    )}
                    <div className="flex items-center gap-2 mt-2">
                      {wh.events.map((event) => (
                        <span
                          key={event}
                          className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-700"
                        >
                          {event}
                        </span>
                      ))}
                    </div>
                  </div>
                  <div className="flex items-center gap-2 ml-4">
                    <button
                      onClick={() => testMutation.mutate(wh.id)}
                      disabled={testMutation.isPending}
                      className="inline-flex items-center px-3 py-1.5 text-sm text-primary-600 hover:text-primary-800 border border-gray-200 rounded-md hover:bg-gray-50"
                      title="Send test event"
                    >
                      <Play className="w-4 h-4 mr-1" />
                      Test
                    </button>
                    <button
                      onClick={() =>
                        setExpandedId(expandedId === wh.id ? null : wh.id)
                      }
                      className="inline-flex items-center px-3 py-1.5 text-sm text-gray-600 hover:text-gray-800 border border-gray-200 rounded-md hover:bg-gray-50"
                    >
                      {expandedId === wh.id ? (
                        <ChevronUp className="w-4 h-4" />
                      ) : (
                        <ChevronDown className="w-4 h-4" />
                      )}
                    </button>
                    <button
                      onClick={() => setDeletingId(wh.id)}
                      className="inline-flex items-center px-3 py-1.5 text-sm text-red-600 hover:text-red-800 border border-gray-200 rounded-md hover:bg-red-50"
                      title="Delete"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              </div>

              {/* Delivery log */}
              {expandedId === wh.id && (
                <div className="border-t border-gray-200 bg-gray-50 px-6 py-4">
                  <h4 className="text-sm font-medium text-gray-700 mb-3">
                    Recent Deliveries
                  </h4>
                  {deliveries && deliveries.length > 0 ? (
                    <table className="min-w-full divide-y divide-gray-200">
                      <thead>
                        <tr>
                          <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                            Status
                          </th>
                          <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                            Event
                          </th>
                          <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                            HTTP
                          </th>
                          <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                            Attempts
                          </th>
                          <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                            Time
                          </th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-gray-200">
                        {deliveries.map((d) => (
                          <tr key={d.id}>
                            <td className="px-3 py-2 whitespace-nowrap">
                              <div className="flex items-center gap-1">
                                {deliveryStatusIcon(d.status)}
                                <span className="text-xs capitalize">
                                  {d.status}
                                </span>
                              </div>
                            </td>
                            <td className="px-3 py-2 whitespace-nowrap text-xs text-gray-700">
                              {d.eventType}
                            </td>
                            <td className="px-3 py-2 whitespace-nowrap text-xs text-gray-500">
                              {d.httpStatus ?? '-'}
                            </td>
                            <td className="px-3 py-2 whitespace-nowrap text-xs text-gray-500">
                              {d.attempts}
                            </td>
                            <td className="px-3 py-2 whitespace-nowrap text-xs text-gray-500">
                              {new Date(d.createdAt).toLocaleString()}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  ) : (
                    <p className="text-sm text-gray-500">
                      No deliveries yet for this endpoint.
                    </p>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Create Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white rounded-lg shadow-xl w-full max-w-lg mx-4 p-6">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-medium">Create Webhook</h3>
              <button onClick={() => setShowCreateModal(false)}>
                <X className="w-5 h-5 text-gray-400" />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  URL (HTTPS required)
                </label>
                <input
                  type="url"
                  value={formUrl}
                  onChange={(e) => setFormUrl(e.target.value)}
                  className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm"
                  placeholder="https://example.com/webhook"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Description (optional)
                </label>
                <input
                  type="text"
                  value={formDescription}
                  onChange={(e) => setFormDescription(e.target.value)}
                  className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm"
                  placeholder="What is this webhook for?"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Events
                </label>
                <div className="space-y-2">
                  {EVENT_TYPES.map((event) => (
                    <label key={event.value} className="flex items-center">
                      <input
                        type="checkbox"
                        checked={formEvents.has(event.value)}
                        onChange={() => toggleEvent(event.value)}
                        className="rounded border-gray-300 text-primary-600 mr-2"
                      />
                      <span className="text-sm text-gray-700">
                        {event.label}
                      </span>
                      <span className="text-xs text-gray-400 ml-2">
                        {event.value}
                      </span>
                    </label>
                  ))}
                </div>
              </div>
            </div>
            <div className="mt-6 flex justify-end space-x-3">
              <button
                onClick={() => setShowCreateModal(false)}
                className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200"
              >
                Cancel
              </button>
              <button
                onClick={handleCreate}
                disabled={
                  !formUrl || formEvents.size === 0 || createMutation.isPending
                }
                className="px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-md hover:bg-primary-700 disabled:opacity-50"
              >
                {createMutation.isPending ? 'Creating...' : 'Create'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete confirmation */}
      {deletingId && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white rounded-lg shadow-xl w-full max-w-sm mx-4 p-6">
            <h3 className="text-lg font-medium text-gray-900 mb-2">
              Delete Webhook
            </h3>
            <p className="text-sm text-gray-500 mb-4">
              Are you sure you want to delete this webhook endpoint? All
              delivery history will be lost.
            </p>
            <div className="flex justify-end space-x-3">
              <button
                onClick={() => setDeletingId(null)}
                className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-md"
              >
                Cancel
              </button>
              <button
                onClick={() => deleteMutation.mutate(deletingId)}
                className="px-4 py-2 text-sm font-medium text-white bg-red-600 rounded-md hover:bg-red-700"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
