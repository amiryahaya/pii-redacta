import { useState, useCallback } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  FlaskConical,
  FileText,
  Upload,
  Loader2,
  Shield,
  Clock,
  Hash,
  CheckCircle2,
  XCircle,
} from 'lucide-react'
import { playgroundApi } from '../lib/api'
import { getErrorMessage } from '../lib/api-errors'
import { useToast } from '../hooks/useToast'
import type { PlaygroundResponse, PlaygroundHistoryEntry } from '../types'

type InputMode = 'text' | 'file'

const ACCEPTED_EXTENSIONS: readonly string[] = ['.txt', '.csv', '.pdf', '.docx']

export function PlaygroundPage() {
  const { showError } = useToast()
  const queryClient = useQueryClient()

  const [mode, setMode] = useState<InputMode>('text')
  const [text, setText] = useState('')
  const [redact, setRedact] = useState(false)
  const [selectedFile, setSelectedFile] = useState<File | null>(null)
  const [result, setResult] = useState<PlaygroundResponse | null>(null)

  // History query
  const { data: history } = useQuery<PlaygroundHistoryEntry[]>({
    queryKey: ['playground-history'],
    queryFn: () => playgroundApi.getHistory().then((r) => r.data),
  })

  // Text submission
  const textMutation = useMutation({
    mutationFn: (data: { text: string; redact?: boolean }) =>
      playgroundApi.submitText(data).then((r) => r.data),
    onSuccess: (data) => {
      setResult(data)
      queryClient.invalidateQueries({ queryKey: ['playground-history'] })
    },
    onError: (err) => {
      showError(getErrorMessage(err, 'Playground analysis failed'))
    },
  })

  // File submission
  const fileMutation = useMutation({
    mutationFn: (file: File) =>
      playgroundApi.uploadFile(file, redact).then((r) => r.data),
    onSuccess: (data) => {
      setResult(data)
      queryClient.invalidateQueries({ queryKey: ['playground-history'] })
    },
    onError: (err) => {
      showError(getErrorMessage(err, 'File analysis failed'))
    },
  })

  const isSubmitting = textMutation.isPending || fileMutation.isPending

  const handleSubmit = useCallback(() => {
    if (mode === 'text') {
      if (!text.trim()) return
      textMutation.mutate({ text, redact: redact || undefined })
    } else {
      if (!selectedFile) return
      fileMutation.mutate(selectedFile)
    }
  }, [mode, text, redact, selectedFile, textMutation, fileMutation])

  const handleFileDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault()
      const file = e.dataTransfer.files[0]
      if (file) {
        const ext = file.name.slice(file.name.lastIndexOf('.')).toLowerCase()
        if (!ACCEPTED_EXTENSIONS.includes(ext)) {
          showError('Unsupported file type. Please use TXT, CSV, PDF, or DOCX.')
          return
        }
        setSelectedFile(file)
        setMode('file')
      }
    },
    [showError]
  )

  const handleFileSelect = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0]
      if (file) {
        const ext = file.name.slice(file.name.lastIndexOf('.')).toLowerCase()
        if (!ACCEPTED_EXTENSIONS.includes(ext)) {
          showError('Unsupported file type. Please use TXT, CSV, PDF, or DOCX.')
          return
        }
        setSelectedFile(file)
      }
    },
    [showError]
  )

  // Group entities by type
  const entityGroups = result
    ? result.entities.reduce(
        (groups, entity) => {
          const type = entity.entity_type
          if (!groups[type]) groups[type] = []
          groups[type].push(entity)
          return groups
        },
        {} as Record<string, typeof result.entities>
      )
    : {}

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 flex items-center gap-2">
            <FlaskConical className="w-7 h-7 text-primary-600" />
            Playground
          </h1>
          <p className="mt-1 text-sm text-gray-500">
            Paste text or upload a file to detect PII instantly
          </p>
        </div>
        {result?.dailyUsage && (
          <div className="flex items-center gap-2 px-3 py-1.5 bg-primary-50 text-primary-700 rounded-full text-sm font-medium">
            <Hash className="w-4 h-4" />
            {result.dailyUsage.usedToday}
            {result.dailyUsage.dailyLimit != null
              ? `/${result.dailyUsage.dailyLimit}`
              : ''}{' '}
            today
          </div>
        )}
      </div>

      {/* Main Content: Input + Results */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Input Panel */}
        <div className="bg-white shadow rounded-lg">
          <div className="border-b border-gray-200">
            <div className="flex" role="tablist" aria-label="Input mode">
              <button
                id="tab-text"
                role="tab"
                aria-selected={mode === 'text'}
                aria-controls="panel-text"
                onClick={() => setMode('text')}
                className={`flex-1 px-4 py-3 text-sm font-medium text-center border-b-2 transition-colors ${
                  mode === 'text'
                    ? 'border-primary-500 text-primary-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700'
                }`}
              >
                <FileText className="w-4 h-4 inline mr-2" aria-hidden="true" />
                Text
              </button>
              <button
                id="tab-file"
                role="tab"
                aria-selected={mode === 'file'}
                aria-controls="panel-file"
                onClick={() => setMode('file')}
                className={`flex-1 px-4 py-3 text-sm font-medium text-center border-b-2 transition-colors ${
                  mode === 'file'
                    ? 'border-primary-500 text-primary-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700'
                }`}
              >
                <Upload className="w-4 h-4 inline mr-2" aria-hidden="true" />
                File
              </button>
            </div>
          </div>

          {/* N2 fix: render both panels so aria-controls always references existing DOM */}
          <div className="p-4" id="panel-text" role="tabpanel" aria-labelledby="tab-text" hidden={mode !== 'text'}>
            <textarea
              aria-label="Text to analyze for PII"
              value={text}
              onChange={(e) => setText(e.target.value)}
              placeholder="Paste text containing PII here... e.g. My email is john@example.com and my phone is +60123456789"
              className="w-full h-64 p-3 text-sm border border-gray-300 rounded-md focus:ring-primary-500 focus:border-primary-500 resize-none"
            />
          </div>
          <div className="p-4" id="panel-file" role="tabpanel" aria-labelledby="tab-file" hidden={mode !== 'file'}>
            <div
              onDragOver={(e) => e.preventDefault()}
              onDrop={handleFileDrop}
              aria-label="File drop zone"
              className="h-64 border-2 border-dashed border-gray-300 rounded-md flex flex-col items-center justify-center text-gray-500 hover:border-primary-400 transition-colors"
            >
              {selectedFile ? (
                <div className="text-center">
                  <FileText className="w-10 h-10 mx-auto mb-2 text-primary-500" />
                  <p className="text-sm font-medium text-gray-900">
                    {selectedFile.name}
                  </p>
                  <p className="text-xs text-gray-500 mt-1">
                    {(selectedFile.size / 1024).toFixed(1)} KB
                  </p>
                  <button
                    onClick={() => setSelectedFile(null)}
                    aria-label={`Remove file ${selectedFile.name}`}
                    className="mt-2 text-xs text-red-600 hover:text-red-700"
                  >
                    Remove
                  </button>
                </div>
              ) : (
                <div className="text-center">
                  <Upload className="w-10 h-10 mx-auto mb-2" />
                  <p className="text-sm">Drag and drop a file here, or</p>
                  <label className="mt-2 inline-block cursor-pointer text-sm font-medium text-primary-600 hover:text-primary-500">
                    browse files
                    <input
                      type="file"
                      className="hidden"
                      accept=".txt,.csv,.pdf,.docx"
                      onChange={handleFileSelect}
                    />
                  </label>
                  <p className="text-xs mt-2">TXT, CSV, PDF, DOCX</p>
                </div>
              )}
            </div>
          </div>

          <div className="px-4 pb-4 flex items-center justify-between">
            <label className="flex items-center gap-2 text-sm text-gray-700">
              <input
                type="checkbox"
                checked={redact}
                onChange={(e) => setRedact(e.target.checked)}
                className="rounded border-gray-300 text-primary-600 focus:ring-primary-500"
              />
              <Shield className="w-4 h-4 text-gray-400" aria-hidden="true" />
              Redact PII
            </label>
            <button
              onClick={handleSubmit}
              disabled={
                isSubmitting ||
                (mode === 'text' && !text.trim()) ||
                (mode === 'file' && !selectedFile)
              }
              className="inline-flex items-center px-4 py-2 bg-primary-600 text-white text-sm font-medium rounded-md hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {isSubmitting ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Analyzing...
                </>
              ) : (
                'Analyze'
              )}
            </button>
          </div>
        </div>

        {/* Results Panel */}
        <div className="bg-white shadow rounded-lg">
          <div className="px-4 py-3 border-b border-gray-200">
            <h3 className="text-sm font-medium text-gray-900">Results</h3>
          </div>

          <div className="p-4">
            {!result ? (
              <div className="h-64 flex items-center justify-center text-gray-400">
                <div className="text-center">
                  <FlaskConical className="w-10 h-10 mx-auto mb-2" />
                  <p className="text-sm">Submit text or a file to see results</p>
                </div>
              </div>
            ) : (
              <div className="space-y-4">
                {/* Stats bar */}
                <div className="flex items-center gap-4 text-xs text-gray-500">
                  <span className="flex items-center gap-1">
                    <Clock className="w-3 h-3" />
                    {result.processingTimeMs.toFixed(1)}ms
                  </span>
                  <span>{result.textLength.toLocaleString()} bytes</span>
                  <span>{result.entities.length} entities found</span>
                </div>

                {/* Entity list */}
                {result.entities.length === 0 ? (
                  <div className="py-8 text-center text-gray-500">
                    <CheckCircle2 className="w-8 h-8 mx-auto mb-2 text-green-500" />
                    <p className="text-sm font-medium">No PII detected</p>
                  </div>
                ) : (
                  <div className="space-y-3 max-h-48 overflow-y-auto">
                    {Object.entries(entityGroups).map(([type, entities]) => (
                      <div key={type}>
                        <h4 className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1">
                          {type.replace(/_/g, ' ')} ({entities.length})
                        </h4>
                        <div className="space-y-1">
                          {entities.map((entity, i) => (
                            <div
                              key={`${type}-${i}`}
                              className="flex items-center justify-between px-2 py-1 bg-red-50 rounded text-sm"
                            >
                              <span className="font-mono text-red-700 truncate max-w-xs">
                                {entity.value}
                              </span>
                              <span className="text-xs text-gray-400 ml-2 flex-shrink-0">
                                {entity.start}-{entity.end}
                              </span>
                            </div>
                          ))}
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {/* Redacted text */}
                {result.redactedText && (
                  <div>
                    <h4 className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1">
                      Redacted Text
                    </h4>
                    <pre className="p-3 bg-gray-50 rounded-md text-xs text-gray-700 overflow-x-auto whitespace-pre-wrap max-h-40 overflow-y-auto">
                      {result.redactedText}
                    </pre>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* History */}
      {history && history.length > 0 && (
        <div className="bg-white shadow rounded-lg">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">Recent Activity</h3>
          </div>
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Type
                  </th>
                  <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    File
                  </th>
                  <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Detections
                  </th>
                  <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Time
                  </th>
                  <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Date
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {history.map((entry) => (
                  <tr key={entry.id}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {entry.requestType === 'playground' ? 'Text' : 'File'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {entry.fileName || '-'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {entry.detectionsCount ?? '-'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {entry.processingTimeMs != null
                        ? `${entry.processingTimeMs}ms`
                        : '-'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      {entry.success ? (
                        <>
                          <CheckCircle2 className="w-4 h-4 text-green-500" aria-hidden="true" />
                          <span className="sr-only">Success</span>
                        </>
                      ) : (
                        <>
                          <XCircle className="w-4 h-4 text-red-500" aria-hidden="true" />
                          <span className="sr-only">Failed</span>
                        </>
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {new Date(entry.createdAt).toLocaleString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  )
}
