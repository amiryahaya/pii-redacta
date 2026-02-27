import { AxiosError } from 'axios'

export interface ApiErrorResponse {
  error?: {
    code?: number
    message?: string
  }
}

export interface ApiError extends AxiosError<ApiErrorResponse> {}

/**
 * Type guard to check if error is an API error
 */
export function isApiError(error: unknown): error is ApiError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'response' in error &&
    typeof (error as ApiError).response?.data?.error?.message === 'string'
  )
}

/**
 * Extract error message from unknown error
 */
export function getErrorMessage(
  error: unknown,
  defaultMessage: string = 'An unexpected error occurred'
): string {
  if (isApiError(error)) {
    return error.response?.data?.error?.message || defaultMessage
  }
  
  if (error instanceof Error) {
    return error.message
  }
  
  if (typeof error === 'string') {
    return error
  }
  
  return defaultMessage
}

/**
 * Map HTTP status codes to user-friendly messages
 */
export function getStatusCodeMessage(status: number): string {
  const messages: Record<number, string> = {
    400: 'Invalid request. Please check your input.',
    401: 'Your session has expired. Please sign in again.',
    403: 'You do not have permission to perform this action.',
    404: 'The requested resource was not found.',
    409: 'This action conflicts with existing data.',
    422: 'Validation failed. Please check your input.',
    429: 'Too many requests. Please try again later.',
    500: 'Server error. Please try again later.',
    502: 'Service temporarily unavailable. Please try again later.',
    503: 'Service temporarily unavailable. Please try again later.',
  }
  
  return messages[status] || 'An unexpected error occurred. Please try again.'
}

/**
 * Handle API error with appropriate messaging
 */
export function handleApiError(
  error: unknown,
  showError: (message: string) => void,
  defaultMessage?: string
): void {
  if (isApiError(error) && error.response?.status) {
    const status = error.response.status
    
    // Handle 401 - redirect to login
    if (status === 401) {
      window.location.href = '/login'
      return
    }
    
    const message = error.response.data?.error?.message 
      || getStatusCodeMessage(status)
    showError(message)
  } else {
    showError(getErrorMessage(error, defaultMessage))
  }
}
