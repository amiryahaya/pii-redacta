import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  Key,
  Plus,
  Trash2,
  AlertTriangle,
  Check,
  Loader2,
  Copy,
} from 'lucide-react'
import { apiKeysApi } from '../lib/api'
import { useToast } from '../hooks/useToast'
import { getErrorMessage } from '../lib/api-errors'
import { formatDateTime, copyToClipboard } from '../lib/utils'
import type { ApiKey, GeneratedApiKey } from '../types'

export function ApiKeysPage() {
  const queryClient = useQueryClient()
  const { showSuccess, showError } = useToast()
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [newKey, setNewKey] = useState<GeneratedApiKey | null>(null)
  const [revokingId, setRevokingId] = useState<string | null>(null)

  const { data: apiKeys, isLoading, error, isError } = useQuery({
    queryKey: ['api-keys'],
    queryFn: () => apiKeysApi.list().then((r) => r.data),
  })

  const createMutation = useMutation({
    mutationFn: apiKeysApi.create,
    onSuccess: (response) => {
      setNewKey(response.data)
      queryClient.invalidateQueries({ queryKey: ['api-keys'] })
      showSuccess('API key created successfully')
    },
    onError: (err: unknown) => {
      showError(getErrorMessage(err, 'Failed to create API key'))
    },
  })

  const revokeMutation = useMutation({
    mutationFn: ({ id, reason }: { id: string; reason?: string }) =>
      apiKeysApi.revoke(id, reason),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['api-keys'] })
      showSuccess('API key revoked successfully')
      setRevokingId(null)
    },
    onError: (err: unknown) => {
      showError(getErrorMessage(err, 'Failed to revoke API key'))
      setRevokingId(null)
    },
  })



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
          <AlertTriangle className="h-5 w-5 text-red-400" aria-hidden="true" />
          <div className="ml-3">
            <h3 className="text-sm font-medium text-red-800">
              Failed to load API keys
            </h3>
            <p className="mt-2 text-sm text-red-700">
              {getErrorMessage(error, 'Please try again later.')}
            </p>
            <button
              onClick={() => queryClient.invalidateQueries({ queryKey: ['api-keys'] })}
              className="mt-3 text-sm font-medium text-red-800 hover:text-red-900"
            >
              Try again
            </button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">API Keys</h1>
          <p className="mt-1 text-sm text-gray-500">
            Manage your API keys for programmatic access
          </p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500"
        >
          <Plus className="w-4 h-4 mr-2" aria-hidden="true" />
          Create API Key
        </button>
      </div>

      {/* Security notice */}
      <div className="rounded-md bg-blue-50 p-4" role="alert">
        <div className="flex">
          <AlertTriangle className="h-5 w-5 text-blue-400" aria-hidden="true" />
          <div className="ml-3">
            <h3 className="text-sm font-medium text-blue-800">
              Keep your API keys secure
            </h3>
            <div className="mt-2 text-sm text-blue-700">
              <p>
                API keys provide full access to your account. Never share them
                or commit them to version control. Use environment variables
                instead.
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* API Keys list */}
      <div className="bg-white shadow overflow-hidden border-b border-gray-200 sm:rounded-lg">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Name
              </th>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Key
              </th>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Last Used
              </th>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Created
              </th>
              <th scope="col" className="relative px-6 py-3">
                <span className="sr-only">Actions</span>
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {apiKeys?.length === 0 ? (
              <tr>
                <td colSpan={5} className="px-6 py-12 text-center">
                  <Key className="mx-auto h-12 w-12 text-gray-300" aria-hidden="true" />
                  <h3 className="mt-2 text-sm font-medium text-gray-900">
                    No API keys
                  </h3>
                  <p className="mt-1 text-sm text-gray-500">
                    Get started by creating a new API key.
                  </p>
                </td>
              </tr>
            ) : (
              apiKeys?.map((key: ApiKey) => (
                <tr key={key.id}>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm font-medium text-gray-900">
                      {key.name}
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <code className="text-sm text-gray-600 bg-gray-100 px-2 py-1 rounded">
                      {key.keyPrefix}••••••••
                    </code>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {formatDateTime(key.lastUsedAt)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {formatDateTime(key.createdAt)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                    <button
                      onClick={() => setRevokingId(key.id)}
                      className="text-red-600 hover:text-red-900"
                      aria-label={`Revoke API key ${key.name}`}
                    >
                      <Trash2 className="w-4 h-4" aria-hidden="true" />
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Create API Key Modal */}
      {showCreateModal && (
        <CreateKeyModal
          onClose={() => {
            setShowCreateModal(false)
            setNewKey(null)
          }}
          onCreate={(data) => createMutation.mutate(data)}
          newKey={newKey}
          isLoading={createMutation.isPending}
        />
      )}

      {/* Revoke Confirmation Modal */}
      {revokingId && (
        <RevokeModal
          onClose={() => setRevokingId(null)}
          onConfirm={(reason) =>
            revokeMutation.mutate({ id: revokingId, reason })
          }
          isLoading={revokeMutation.isPending}
        />
      )}
    </div>
  )
}

interface CreateKeyModalProps {
  onClose: () => void
  onCreate: (data: { name: string; environment: 'live' | 'test' }) => void
  newKey: GeneratedApiKey | null
  isLoading: boolean
}

function CreateKeyModal({ onClose, onCreate, newKey, isLoading }: CreateKeyModalProps) {
  const [name, setName] = useState('')
  const [environment, setEnvironment] = useState<'live' | 'test'>('live')
  const [copied, setCopied] = useState(false)
  const { showError, showSuccess } = useToast()

  const handleCopy = async () => {
    if (newKey?.fullKey) {
      try {
        await copyToClipboard(newKey.fullKey)
        setCopied(true)
        showSuccess('API key copied to clipboard')
        setTimeout(() => setCopied(false), 2000)
      } catch {
        showError('Failed to copy to clipboard')
      }
    }
  }

  // Close on escape key
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose()
    }
  }

  return (
    <div 
      className="fixed inset-0 z-50 overflow-y-auto"
      onKeyDown={handleKeyDown}
      role="dialog"
      aria-modal="true"
      aria-labelledby="create-key-title"
    >
      <div className="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0">
        <div
          className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
          onClick={onClose}
          aria-hidden="true"
        />
        <span className="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">
          &#8203;
        </span>
        <div className="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
          <div className="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
            <div className="sm:flex sm:items-start">
              <div className="mt-3 text-center sm:mt-0 sm:text-left w-full">
                <h3 
                  className="text-lg leading-6 font-medium text-gray-900" 
                  id="create-key-title"
                >
                  {newKey ? 'API Key Created' : 'Create API Key'}
                </h3>
                <div className="mt-4">
                  {newKey ? (
                    <div className="space-y-4">
                      <div className="rounded-md bg-yellow-50 p-4">
                        <div className="flex">
                          <AlertTriangle className="h-5 w-5 text-yellow-400" aria-hidden="true" />
                          <div className="ml-3">
                            <h4 className="text-sm font-medium text-yellow-800">
                              Copy this key now
                            </h4>
                            <p className="text-sm text-yellow-700 mt-1">
                              This is the only time you&apos;ll see the full key. Store it
                              securely.
                            </p>
                          </div>
                        </div>
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-gray-700">
                          Your API Key
                        </label>
                        <div className="mt-1 flex rounded-md shadow-sm">
                          <input
                            type="text"
                            readOnly
                            value={newKey.fullKey}
                            className="focus:ring-primary-500 focus:border-primary-500 flex-1 block w-full rounded-none rounded-l-md sm:text-sm font-mono bg-gray-50 border-gray-300"
                            aria-label="API Key"
                          />
                          <button
                            type="button"
                            onClick={handleCopy}
                            className="inline-flex items-center px-3 py-2 border border-l-0 border-gray-300 rounded-r-md bg-gray-50 text-gray-500 hover:bg-gray-100 focus:outline-none focus:ring-1 focus:ring-primary-500"
                            aria-label={copied ? 'Copied' : 'Copy to clipboard'}
                          >
                            {copied ? (
                              <Check className="w-4 h-4 text-green-600" aria-hidden="true" />
                            ) : (
                              <Copy className="w-4 h-4" aria-hidden="true" />
                            )}
                          </button>
                        </div>
                      </div>

                      <button
                        onClick={onClose}
                        className="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-primary-600 text-base font-medium text-white hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 sm:text-sm"
                      >
                        I&apos;ve copied my key
                      </button>
                    </div>
                  ) : (
                    <form
                      onSubmit={(e) => {
                        e.preventDefault()
                        onCreate({ name, environment })
                      }}
                      className="space-y-4"
                    >
                      <div>
                        <label htmlFor="key-name" className="block text-sm font-medium text-gray-700">
                          Key Name
                        </label>
                        <input
                          id="key-name"
                          name="name"
                          type="text"
                          required
                          placeholder="e.g., Production API"
                          value={name}
                          onChange={(e) => setName(e.target.value)}
                          className="mt-1 focus:ring-primary-500 focus:border-primary-500 block w-full shadow-sm sm:text-sm border-gray-300 rounded-md"
                        />
                      </div>

                      <fieldset>
                        <legend className="block text-sm font-medium text-gray-700">
                          Environment
                        </legend>
                        <div className="mt-2 space-y-2">
                          <div className="flex items-center">
                            <input
                              id="env-live"
                              name="environment"
                              type="radio"
                              value="live"
                              checked={environment === 'live'}
                              onChange={() => setEnvironment('live')}
                              className="focus:ring-primary-500 h-4 w-4 text-primary-600 border-gray-300"
                            />
                            <label htmlFor="env-live" className="ml-3 block text-sm text-gray-700">
                              Live - Production use
                            </label>
                          </div>
                          <div className="flex items-center">
                            <input
                              id="env-test"
                              name="environment"
                              type="radio"
                              value="test"
                              checked={environment === 'test'}
                              onChange={() => setEnvironment('test')}
                              className="focus:ring-primary-500 h-4 w-4 text-primary-600 border-gray-300"
                            />
                            <label htmlFor="env-test" className="ml-3 block text-sm text-gray-700">
                              Test - Development and testing
                            </label>
                          </div>
                        </div>
                      </fieldset>

                      <div className="mt-5 sm:mt-4 sm:flex sm:flex-row-reverse">
                        <button
                          type="submit"
                          disabled={isLoading}
                          className="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-primary-600 text-base font-medium text-white hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50"
                        >
                          {isLoading ? (
                            <>
                              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                              Creating...
                            </>
                          ) : (
                            'Create Key'
                          )}
                        </button>
                        <button
                          type="button"
                          onClick={onClose}
                          className="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 sm:mt-0 sm:w-auto sm:text-sm"
                        >
                          Cancel
                        </button>
                      </div>
                    </form>
                  )}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

interface RevokeModalProps {
  onClose: () => void
  onConfirm: (reason?: string) => void
  isLoading: boolean
}

function RevokeModal({ onClose, onConfirm, isLoading }: RevokeModalProps) {
  const [reason, setReason] = useState('')

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose()
    }
  }

  return (
    <div 
      className="fixed inset-0 z-50 overflow-y-auto"
      onKeyDown={handleKeyDown}
      role="dialog"
      aria-modal="true"
      aria-labelledby="revoke-title"
    >
      <div className="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0">
        <div
          className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
          onClick={onClose}
          aria-hidden="true"
        />
        <span className="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">
          &#8203;
        </span>
        <div className="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
          <div className="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
            <div className="sm:flex sm:items-start">
              <div className="mx-auto flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full bg-red-100 sm:mx-0 sm:h-10 sm:w-10">
                <AlertTriangle className="h-6 w-6 text-red-600" aria-hidden="true" />
              </div>
              <div className="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left">
                <h3 className="text-lg leading-6 font-medium text-gray-900" id="revoke-title">
                  Revoke API Key
                </h3>
                <div className="mt-2">
                  <p className="text-sm text-gray-500">
                    This will immediately disable the API key. Any applications
                    using this key will stop working.
                  </p>
                </div>
                <div className="mt-4">
                  <label htmlFor="revoke-reason" className="block text-sm font-medium text-gray-700">
                    Reason (optional)
                  </label>
                  <input
                    id="revoke-reason"
                    type="text"
                    placeholder="e.g., Key compromised"
                    value={reason}
                    onChange={(e) => setReason(e.target.value)}
                    className="mt-1 focus:ring-red-500 focus:border-red-500 block w-full shadow-sm sm:text-sm border-gray-300 rounded-md"
                  />
                </div>
              </div>
            </div>
          </div>
          <div className="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
            <button
              type="button"
              onClick={() => onConfirm(reason)}
              disabled={isLoading}
              className="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50"
            >
              {isLoading ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Revoking...
                </>
              ) : (
                'Revoke Key'
              )}
            </button>
            <button
              type="button"
              onClick={onClose}
              className="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm"
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
