import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  FileCode2,
  Plus,
  Trash2,
  Pencil,
  Play,
  AlertTriangle,
  Loader2,
  Lock,
  X,
} from 'lucide-react'
import { rulesApi, subscriptionApi } from '../lib/api'
import { useToast } from '../hooks/useToast'
import { getErrorMessage } from '../lib/api-errors'
import type { CustomRule } from '../types'

export function CustomRulesPage() {
  const queryClient = useQueryClient()
  const { showSuccess, showError } = useToast()
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [editingRule, setEditingRule] = useState<CustomRule | null>(null)
  const [testingRuleId, setTestingRuleId] = useState<string | null>(null)
  const [testText, setTestText] = useState('')
  const [testResults, setTestResults] = useState<{
    matches: Array<{ value: string; start: number; end: number; entityLabel: string; confidence: number }>
    processingTimeMs: number
  } | null>(null)
  const [deletingId, setDeletingId] = useState<string | null>(null)

  // Form state
  const [formName, setFormName] = useState('')
  const [formDescription, setFormDescription] = useState('')
  const [formPattern, setFormPattern] = useState('')
  const [formEntityLabel, setFormEntityLabel] = useState('')
  const [formConfidence, setFormConfidence] = useState(0.9)

  const { data: subscription } = useQuery({
    queryKey: ['subscription'],
    queryFn: () => subscriptionApi.get().then((r) => r.data),
  })

  const hasFeature = subscription?.tier?.features?.customRules ?? false

  const {
    data: rules,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: ['custom-rules'],
    queryFn: () => rulesApi.list().then((r) => r.data),
    enabled: hasFeature,
  })

  const createMutation = useMutation({
    mutationFn: rulesApi.create,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['custom-rules'] })
      showSuccess('Rule created successfully')
      resetForm()
      setShowCreateModal(false)
    },
    onError: (err: unknown) => {
      showError(getErrorMessage(err, 'Failed to create rule'))
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Parameters<typeof rulesApi.update>[1] }) =>
      rulesApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['custom-rules'] })
      showSuccess('Rule updated successfully')
      setEditingRule(null)
      resetForm()
    },
    onError: (err: unknown) => {
      showError(getErrorMessage(err, 'Failed to update rule'))
    },
  })

  const deleteMutation = useMutation({
    mutationFn: rulesApi.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['custom-rules'] })
      showSuccess('Rule deleted')
      setDeletingId(null)
    },
    onError: (err: unknown) => {
      showError(getErrorMessage(err, 'Failed to delete rule'))
      setDeletingId(null)
    },
  })

  const testMutation = useMutation({
    mutationFn: ({ id, text }: { id: string; text: string }) =>
      rulesApi.test(id, { text }),
    onSuccess: (response) => {
      setTestResults(response.data)
    },
    onError: (err: unknown) => {
      showError(getErrorMessage(err, 'Failed to test rule'))
    },
  })

  const resetForm = () => {
    setFormName('')
    setFormDescription('')
    setFormPattern('')
    setFormEntityLabel('')
    setFormConfidence(0.9)
  }

  const openEditModal = (rule: CustomRule) => {
    setEditingRule(rule)
    setFormName(rule.name)
    setFormDescription(rule.description || '')
    setFormPattern(rule.pattern)
    setFormEntityLabel(rule.entityLabel)
    setFormConfidence(rule.confidence)
  }

  const handleSubmit = () => {
    if (editingRule) {
      updateMutation.mutate({
        id: editingRule.id,
        data: {
          name: formName,
          description: formDescription || undefined,
          pattern: formPattern,
          entityLabel: formEntityLabel,
          confidence: formConfidence,
        },
      })
    } else {
      createMutation.mutate({
        name: formName,
        description: formDescription || undefined,
        pattern: formPattern,
        entityLabel: formEntityLabel,
        confidence: formConfidence,
      })
    }
  }

  // Tier gate
  if (!hasFeature) {
    return (
      <div className="text-center py-16">
        <Lock className="w-12 h-12 text-gray-400 mx-auto mb-4" />
        <h2 className="text-xl font-semibold text-gray-900 mb-2">
          Custom Rules
        </h2>
        <p className="text-gray-500 max-w-md mx-auto">
          Custom detection rules are available on Pro and Enterprise plans.
          Upgrade your plan to define custom regex patterns for PII detection.
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
            <h3 className="text-sm font-medium text-red-800">Failed to load rules</h3>
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
          <h1 className="text-2xl font-bold text-gray-900">Custom Rules</h1>
          <p className="mt-1 text-sm text-gray-500">
            Define custom regex patterns for PII detection
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
          New Rule
        </button>
      </div>

      {/* Rules table */}
      {rules && rules.length === 0 ? (
        <div className="text-center py-12 bg-white rounded-lg border border-gray-200">
          <FileCode2 className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900">No custom rules yet</h3>
          <p className="mt-2 text-sm text-gray-500">
            Create your first custom detection rule to get started.
          </p>
        </div>
      ) : (
        <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Name</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Pattern</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Label</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Active</th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {rules?.map((rule) => (
                <tr key={rule.id}>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm font-medium text-gray-900">{rule.name}</div>
                    {rule.description && (
                      <div className="text-sm text-gray-500">{rule.description}</div>
                    )}
                  </td>
                  <td className="px-6 py-4">
                    <code className="text-xs bg-gray-100 px-2 py-1 rounded">{rule.pattern}</code>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                      {rule.entityLabel}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span
                      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        rule.isActive
                          ? 'bg-green-100 text-green-800'
                          : 'bg-gray-100 text-gray-800'
                      }`}
                    >
                      {rule.isActive ? 'Active' : 'Inactive'}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm">
                    <button
                      onClick={() => {
                        setTestingRuleId(rule.id)
                        setTestText('')
                        setTestResults(null)
                      }}
                      className="text-primary-600 hover:text-primary-800 mr-3"
                      title="Test"
                    >
                      <Play className="w-4 h-4 inline" />
                    </button>
                    <button
                      onClick={() => openEditModal(rule)}
                      className="text-gray-600 hover:text-gray-800 mr-3"
                      title="Edit"
                    >
                      <Pencil className="w-4 h-4 inline" />
                    </button>
                    <button
                      onClick={() => setDeletingId(rule.id)}
                      className="text-red-600 hover:text-red-800"
                      title="Delete"
                    >
                      <Trash2 className="w-4 h-4 inline" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Test Panel */}
      {testingRuleId && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white rounded-lg shadow-xl w-full max-w-lg mx-4 p-6">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-medium">Test Rule</h3>
              <button onClick={() => setTestingRuleId(null)}>
                <X className="w-5 h-5 text-gray-400" />
              </button>
            </div>
            <textarea
              value={testText}
              onChange={(e) => setTestText(e.target.value)}
              placeholder="Enter sample text to test against this rule..."
              className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm h-32 mb-4"
            />
            <button
              onClick={() =>
                testMutation.mutate({ id: testingRuleId, text: testText })
              }
              disabled={!testText || testMutation.isPending}
              className="w-full py-2 bg-primary-600 text-white rounded-md text-sm font-medium hover:bg-primary-700 disabled:opacity-50"
            >
              {testMutation.isPending ? 'Testing...' : 'Run Test'}
            </button>
            {testResults && (
              <div className="mt-4">
                <p className="text-sm text-gray-600 mb-2">
                  {testResults.matches.length} match(es) found in{' '}
                  {testResults.processingTimeMs.toFixed(1)}ms
                </p>
                {testResults.matches.map((m, i) => (
                  <div
                    key={i}
                    className="bg-yellow-50 border border-yellow-200 rounded px-3 py-2 text-sm mb-2"
                  >
                    <span className="font-mono">{m.value}</span>
                    <span className="text-gray-500 ml-2">
                      [{m.start}:{m.end}]
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Create/Edit Modal */}
      {(showCreateModal || editingRule) && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white rounded-lg shadow-xl w-full max-w-lg mx-4 p-6">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-medium">
                {editingRule ? 'Edit Rule' : 'Create Rule'}
              </h3>
              <button
                onClick={() => {
                  setShowCreateModal(false)
                  setEditingRule(null)
                }}
              >
                <X className="w-5 h-5 text-gray-400" />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Name
                </label>
                <input
                  type="text"
                  value={formName}
                  onChange={(e) => setFormName(e.target.value)}
                  className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm"
                  placeholder="e.g., Employee ID"
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
                  placeholder="What does this rule detect?"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Regex Pattern
                </label>
                <input
                  type="text"
                  value={formPattern}
                  onChange={(e) => setFormPattern(e.target.value)}
                  className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm font-mono"
                  placeholder="e.g., EMP-\d{6}"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Entity Label
                </label>
                <input
                  type="text"
                  value={formEntityLabel}
                  onChange={(e) => setFormEntityLabel(e.target.value)}
                  className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm"
                  placeholder="e.g., EMPLOYEE_ID"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Confidence ({(formConfidence * 100).toFixed(0)}%)
                </label>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.05"
                  value={formConfidence}
                  onChange={(e) => setFormConfidence(parseFloat(e.target.value))}
                  className="w-full"
                />
              </div>
            </div>
            <div className="mt-6 flex justify-end space-x-3">
              <button
                onClick={() => {
                  setShowCreateModal(false)
                  setEditingRule(null)
                }}
                className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200"
              >
                Cancel
              </button>
              <button
                onClick={handleSubmit}
                disabled={
                  !formName ||
                  !formPattern ||
                  !formEntityLabel ||
                  createMutation.isPending ||
                  updateMutation.isPending
                }
                className="px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-md hover:bg-primary-700 disabled:opacity-50"
              >
                {createMutation.isPending || updateMutation.isPending
                  ? 'Saving...'
                  : editingRule
                  ? 'Update'
                  : 'Create'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete confirmation */}
      {deletingId && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white rounded-lg shadow-xl w-full max-w-sm mx-4 p-6">
            <h3 className="text-lg font-medium text-gray-900 mb-2">Delete Rule</h3>
            <p className="text-sm text-gray-500 mb-4">
              Are you sure you want to delete this rule? This action cannot be undone.
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
