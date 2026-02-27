import { useState, useCallback } from 'react'

interface FormState<T> {
  values: T
  errors: Partial<Record<keyof T, string>>
  touched: Partial<Record<keyof T, boolean>>
  isSubmitting: boolean
  submitError: string | null
}

interface UseFormOptions<T> {
  initialValues: T
  onSubmit: (values: T) => Promise<void>
  validate?: (values: T) => Partial<Record<keyof T, string>>
}

export function useForm<T extends Record<string, unknown>>({
  initialValues,
  onSubmit,
  validate,
}: UseFormOptions<T>) {
  const [state, setState] = useState<FormState<T>>({
    values: initialValues,
    errors: {},
    touched: {},
    isSubmitting: false,
    submitError: null,
  })

  const setValue = useCallback(<K extends keyof T>(
    field: K,
    value: T[K]
  ) => {
    setState((prev) => ({
      ...prev,
      values: { ...prev.values, [field]: value },
      // Clear error when user starts typing
      errors: prev.errors[field] ? { ...prev.errors, [field]: undefined } : prev.errors,
      submitError: null,
    }))
  }, [])

  const setTouched = useCallback(<K extends keyof T>(field: K) => {
    setState((prev) => ({
      ...prev,
      touched: { ...prev.touched, [field]: true },
    }))
  }, [])

  const validateField = useCallback(<K extends keyof T>(field: K) => {
    if (!validate) return
    
    const errors = validate(state.values)
    const fieldError = errors[field]
    
    setState((prev) => ({
      ...prev,
      errors: { ...prev.errors, [field]: fieldError },
    }))
  }, [validate, state.values])

  const handleSubmit = useCallback(async (e: React.FormEvent) => {
    e.preventDefault()

    // Validate all fields
    if (validate) {
      const errors = validate(state.values)
      setState((prev) => ({ ...prev, errors, touched: {} }))
      
      if (Object.keys(errors).length > 0) {
        return
      }
    }

    setState((prev) => ({ ...prev, isSubmitting: true, submitError: null }))

    try {
      await onSubmit(state.values)
      setState((prev) => ({ ...prev, isSubmitting: false }))
    } catch (err) {
      const message = err instanceof Error ? err.message : 'An error occurred'
      setState((prev) => ({
        ...prev,
        isSubmitting: false,
        submitError: message,
      }))
    }
  }, [state.values, validate, onSubmit])

  const resetForm = useCallback(() => {
    setState({
      values: initialValues,
      errors: {},
      touched: {},
      isSubmitting: false,
      submitError: null,
    })
  }, [initialValues])

  return {
    values: state.values,
    errors: state.errors,
    touched: state.touched,
    isSubmitting: state.isSubmitting,
    submitError: state.submitError,
    setValue,
    setTouched,
    validateField,
    handleSubmit,
    resetForm,
  }
}
