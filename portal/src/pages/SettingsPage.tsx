import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  User,
  Lock,
  Bell,
  Shield,
  Loader2,
  Check,
  Eye,
  EyeOff,
  AlertCircle,
} from 'lucide-react'
import { useAuthStore } from '../stores/authStore'
import { userApi, authApi } from '../lib/api'
import { useToast } from '../hooks/useToast'
import { getErrorMessage } from '../lib/api-errors'
import type { UserPreferences } from '../types'
import { z } from 'zod'

// Validation schemas
const profileSchema = z.object({
  displayName: z.string().max(100, 'Name must be less than 100 characters').optional(),
  companyName: z.string().max(100, 'Company name must be less than 100 characters').optional(),
})

const passwordSchema = z
  .object({
    currentPassword: z.string().min(1, 'Current password is required'),
    newPassword: z
      .string()
      .min(8, 'New password must be at least 8 characters')
      .regex(/[a-zA-Z]/, 'Password must contain at least one letter')
      .regex(/\d/, 'Password must contain at least one number'),
    confirmPassword: z.string().min(1, 'Please confirm your new password'),
  })
  .refine((data) => data.newPassword === data.confirmPassword, {
    message: 'Passwords do not match',
    path: ['confirmPassword'],
  })

export function SettingsPage() {
  const [activeTab, setActiveTab] = useState<'profile' | 'security' | 'notifications'>('profile')

  const tabs = [
    { id: 'profile', label: 'Profile', icon: User },
    { id: 'security', label: 'Security', icon: Lock },
    { id: 'notifications', label: 'Notifications', icon: Bell },
  ]

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Settings</h1>
        <p className="mt-1 text-sm text-gray-500">
          Manage your account settings and preferences
        </p>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8" aria-label="Settings tabs">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as typeof activeTab)}
              className={`group inline-flex items-center py-4 px-1 border-b-2 font-medium text-sm ${
                activeTab === tab.id
                  ? 'border-primary-500 text-primary-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
              aria-current={activeTab === tab.id ? 'page' : undefined}
            >
              <tab.icon
                className={`-ml-0.5 mr-2 h-5 w-5 ${
                  activeTab === tab.id
                    ? 'text-primary-500'
                    : 'text-gray-400 group-hover:text-gray-500'
                }`}
                aria-hidden="true"
              />
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {/* Content */}
      <div className="bg-white shadow sm:rounded-lg">
        {activeTab === 'profile' && <ProfileSettings />}
        {activeTab === 'security' && <SecuritySettings />}
        {activeTab === 'notifications' && <NotificationSettings />}
      </div>
    </div>
  )
}

function ProfileSettings() {
  const { user, setUser } = useAuthStore()
  const { showSuccess, showError } = useToast()
  const [errors, setErrors] = useState<Record<string, string>>({})
  const [formData, setFormData] = useState({
    displayName: user?.displayName || '',
    companyName: user?.companyName || '',
  })

  const mutation = useMutation({
    mutationFn: userApi.updateProfile,
    onSuccess: (response) => {
      setUser(response.data)
      showSuccess('Profile updated successfully')
      setErrors({})
    },
    onError: (err: unknown) => {
      showError(getErrorMessage(err, 'Failed to update profile'))
    },
  })

  const handleChange = (field: string, value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }))
    if (errors[field]) {
      setErrors((prev) => ({ ...prev, [field]: '' }))
    }
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()

    const result = profileSchema.safeParse(formData)
    if (!result.success) {
      const fieldErrors: Record<string, string> = {}
      result.error.errors.forEach((err) => {
        const field = err.path[0] as string
        fieldErrors[field] = err.message
      })
      setErrors(fieldErrors)
      return
    }

    mutation.mutate(formData)
  }

  return (
    <div className="px-4 py-5 sm:p-6">
      <div className="flex items-center space-x-5">
        <div className="flex-shrink-0">
          <div className="relative">
            <div className="h-20 w-20 rounded-full bg-primary-100 flex items-center justify-center">
              <span className="text-3xl font-medium text-primary-600">
                {user?.displayName?.[0] || user?.email?.[0] || '?'}
              </span>
            </div>
          </div>
        </div>
        <div>
          <h3 className="text-lg leading-6 font-medium text-gray-900">Profile Information</h3>
          <p className="mt-1 text-sm text-gray-500">
            Update your personal information
          </p>
        </div>
      </div>

      <form onSubmit={handleSubmit} className="mt-6 space-y-6">
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
          <div>
            <label htmlFor="email" className="block text-sm font-medium text-gray-700">
              Email
            </label>
            <input
              type="email"
              id="email"
              name="email"
              value={user?.email || ''}
              disabled
              className="mt-1 block w-full border-gray-300 rounded-md shadow-sm bg-gray-50 text-gray-500 sm:text-sm"
              aria-label="Email address (cannot be changed)"
            />
            <p className="mt-1 text-xs text-gray-500">
              Email cannot be changed. Contact support for assistance.
            </p>
          </div>

          <div>
            <label htmlFor="displayName" className="block text-sm font-medium text-gray-700">
              Full Name
            </label>
            <input
              type="text"
              id="displayName"
              name="displayName"
              value={formData.displayName}
              onChange={(e) => handleChange('displayName', e.target.value)}
              aria-invalid={!!errors.displayName}
              aria-describedby={errors.displayName ? 'displayName-error' : undefined}
              className={`mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-primary-500 focus:border-primary-500 sm:text-sm ${
                errors.displayName ? 'border-red-300' : ''
              }`}
            />
            {errors.displayName && (
              <p id="displayName-error" className="mt-1 text-sm text-red-600" role="alert">
                {errors.displayName}
              </p>
            )}
          </div>

          <div>
            <label htmlFor="companyName" className="block text-sm font-medium text-gray-700">
              Company Name
            </label>
            <input
              type="text"
              id="companyName"
              name="companyName"
              value={formData.companyName}
              onChange={(e) => handleChange('companyName', e.target.value)}
              aria-invalid={!!errors.companyName}
              aria-describedby={errors.companyName ? 'companyName-error' : undefined}
              className={`mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-primary-500 focus:border-primary-500 sm:text-sm ${
                errors.companyName ? 'border-red-300' : ''
              }`}
            />
            {errors.companyName && (
              <p id="companyName-error" className="mt-1 text-sm text-red-600" role="alert">
                {errors.companyName}
              </p>
            )}
          </div>
        </div>

        <div className="flex justify-end">
          <button
            type="submit"
            disabled={mutation.isPending}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50"
          >
            {mutation.isPending ? (
              <>
                <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <Check className="w-4 h-4 mr-2" />
                Save Changes
              </>
            )}
          </button>
        </div>
      </form>
    </div>
  )
}

function SecuritySettings() {
  const { showSuccess, showError } = useToast()
  const [errors, setErrors] = useState<Record<string, string>>({})
  const [showPasswords, setShowPasswords] = useState({
    current: false,
    new: false,
    confirm: false,
  })
  const [formData, setFormData] = useState({
    currentPassword: '',
    newPassword: '',
    confirmPassword: '',
  })

  const passwordChecks = [
    { label: 'At least 8 characters', met: formData.newPassword.length >= 8 },
    { label: 'Contains a number', met: /\d/.test(formData.newPassword) },
    { label: 'Contains a letter', met: /[a-zA-Z]/.test(formData.newPassword) },
  ]

  const mutation = useMutation({
    mutationFn: authApi.changePassword,
    onSuccess: () => {
      showSuccess('Password changed successfully')
      setFormData({ currentPassword: '', newPassword: '', confirmPassword: '' })
      setErrors({})
    },
    onError: (err: unknown) => {
      showError(getErrorMessage(err, 'Failed to change password'))
    },
  })

  const handleChange = (field: string, value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }))
    if (errors[field]) {
      setErrors((prev) => ({ ...prev, [field]: '' }))
    }
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()

    const result = passwordSchema.safeParse(formData)
    if (!result.success) {
      const fieldErrors: Record<string, string> = {}
      result.error.errors.forEach((err) => {
        const field = err.path[0] as string
        fieldErrors[field] = err.message
      })
      setErrors(fieldErrors)
      return
    }

    mutation.mutate({
      currentPassword: formData.currentPassword,
      newPassword: formData.newPassword,
    })
  }

  const togglePasswordVisibility = (field: 'current' | 'new' | 'confirm') => {
    setShowPasswords((prev) => ({ ...prev, [field]: !prev[field] }))
  }

  return (
    <div className="px-4 py-5 sm:p-6">
      <div className="flex items-center space-x-3">
        <Shield className="h-6 w-6 text-gray-400" aria-hidden="true" />
        <div>
          <h3 className="text-lg leading-6 font-medium text-gray-900">Security</h3>
          <p className="mt-1 text-sm text-gray-500">
            Update your password and security settings
          </p>
        </div>
      </div>

      <form onSubmit={handleSubmit} className="mt-6 space-y-6">
        <div className="max-w-md space-y-4">
          <div>
            <label htmlFor="currentPassword" className="block text-sm font-medium text-gray-700">
              Current Password
            </label>
            <div className="mt-1 relative">
              <input
                type={showPasswords.current ? 'text' : 'password'}
                id="currentPassword"
                name="currentPassword"
                value={formData.currentPassword}
                onChange={(e) => handleChange('currentPassword', e.target.value)}
                aria-invalid={!!errors.currentPassword}
                aria-describedby={errors.currentPassword ? 'currentPassword-error' : undefined}
                className={`block w-full border-gray-300 rounded-md shadow-sm focus:ring-primary-500 focus:border-primary-500 sm:text-sm pr-10 ${
                  errors.currentPassword ? 'border-red-300' : ''
                }`}
              />
              <button
                type="button"
                onClick={() => togglePasswordVisibility('current')}
                className="absolute inset-y-0 right-0 pr-3 flex items-center"
                aria-label={showPasswords.current ? 'Hide current password' : 'Show current password'}
                aria-pressed={showPasswords.current}
              >
                {showPasswords.current ? (
                  <EyeOff className="h-5 w-5 text-gray-400" aria-hidden="true" />
                ) : (
                  <Eye className="h-5 w-5 text-gray-400" aria-hidden="true" />
                )}
              </button>
            </div>
            {errors.currentPassword && (
              <p id="currentPassword-error" className="mt-1 text-sm text-red-600" role="alert">
                {errors.currentPassword}
              </p>
            )}
          </div>

          <div>
            <label htmlFor="newPassword" className="block text-sm font-medium text-gray-700">
              New Password
            </label>
            <div className="mt-1 relative">
              <input
                type={showPasswords.new ? 'text' : 'password'}
                id="newPassword"
                name="newPassword"
                value={formData.newPassword}
                onChange={(e) => handleChange('newPassword', e.target.value)}
                aria-invalid={!!errors.newPassword}
                aria-describedby={errors.newPassword ? 'newPassword-error' : 'password-requirements'}
                className={`block w-full border-gray-300 rounded-md shadow-sm focus:ring-primary-500 focus:border-primary-500 sm:text-sm pr-10 ${
                  errors.newPassword ? 'border-red-300' : ''
                }`}
              />
              <button
                type="button"
                onClick={() => togglePasswordVisibility('new')}
                className="absolute inset-y-0 right-0 pr-3 flex items-center"
                aria-label={showPasswords.new ? 'Hide new password' : 'Show new password'}
                aria-pressed={showPasswords.new}
              >
                {showPasswords.new ? (
                  <EyeOff className="h-5 w-5 text-gray-400" aria-hidden="true" />
                ) : (
                  <Eye className="h-5 w-5 text-gray-400" aria-hidden="true" />
                )}
              </button>
            </div>
            {errors.newPassword && (
              <p id="newPassword-error" className="mt-1 text-sm text-red-600" role="alert">
                {errors.newPassword}
              </p>
            )}
            <ul id="password-requirements" className="mt-2 space-y-1">
              {passwordChecks.map((req) => (
                <li
                  key={req.label}
                  className={`text-xs flex items-center ${
                    req.met ? 'text-green-600' : 'text-gray-500'
                  }`}
                >
                  <Check
                    className={`w-3 h-3 mr-1 ${req.met ? 'opacity-100' : 'opacity-0'}`}
                    aria-hidden="true"
                  />
                  {req.label}
                </li>
              ))}
            </ul>
          </div>

          <div>
            <label htmlFor="confirmPassword" className="block text-sm font-medium text-gray-700">
              Confirm New Password
            </label>
            <div className="mt-1 relative">
              <input
                type={showPasswords.confirm ? 'text' : 'password'}
                id="confirmPassword"
                name="confirmPassword"
                value={formData.confirmPassword}
                onChange={(e) => handleChange('confirmPassword', e.target.value)}
                aria-invalid={!!errors.confirmPassword}
                aria-describedby={errors.confirmPassword ? 'confirmPassword-error' : undefined}
                className={`block w-full border-gray-300 rounded-md shadow-sm focus:ring-primary-500 focus:border-primary-500 sm:text-sm pr-10 ${
                  errors.confirmPassword ? 'border-red-300' : ''
                }`}
              />
              <button
                type="button"
                onClick={() => togglePasswordVisibility('confirm')}
                className="absolute inset-y-0 right-0 pr-3 flex items-center"
                aria-label={showPasswords.confirm ? 'Hide confirmation password' : 'Show confirmation password'}
                aria-pressed={showPasswords.confirm}
              >
                {showPasswords.confirm ? (
                  <EyeOff className="h-5 w-5 text-gray-400" aria-hidden="true" />
                ) : (
                  <Eye className="h-5 w-5 text-gray-400" aria-hidden="true" />
                )}
              </button>
            </div>
            {errors.confirmPassword && (
              <p id="confirmPassword-error" className="mt-1 text-sm text-red-600" role="alert">
                {errors.confirmPassword}
              </p>
            )}
          </div>
        </div>

        <div className="flex justify-end">
          <button
            type="submit"
            disabled={mutation.isPending}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50"
          >
            {mutation.isPending ? (
              <>
                <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                Updating...
              </>
            ) : (
              'Change Password'
            )}
          </button>
        </div>
      </form>

      {/* Security tips */}
      <div className="mt-8 rounded-md bg-blue-50 p-4">
        <div className="flex">
          <AlertCircle className="h-5 w-5 text-blue-400" aria-hidden="true" />
          <div className="ml-3">
            <h4 className="text-sm font-medium text-blue-800">
              Security Tips
            </h4>
            <ul className="mt-2 text-sm text-blue-700 list-disc list-inside space-y-1">
              <li>Use a unique password that you don&apos;t use elsewhere</li>
              <li>Enable two-factor authentication when available</li>
              <li>Never share your password with anyone</li>
              <li>Change your password periodically</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}

function NotificationSettings() {
  const queryClient = useQueryClient()
  const { showSuccess, showError } = useToast()
  const [isSaving, setIsSaving] = useState(false)

  const { data: preferences, isLoading } = useQuery<UserPreferences>({
    queryKey: ['notification-preferences'],
    queryFn: () => userApi.getPreferences().then((r) => r.data),
  })

  const [formData, setFormData] = useState<UserPreferences | null>(null)

  // Initialize form data when preferences load
  if (preferences && !formData) {
    setFormData(preferences)
  }

  const handleToggle = async (key: keyof UserPreferences) => {
    if (!formData) return

    const newData = { ...formData, [key]: !formData[key] }
    setFormData(newData)
    setIsSaving(true)

    try {
      await userApi.updatePreferences(newData)
      queryClient.invalidateQueries({ queryKey: ['notification-preferences'] })
      showSuccess('Preferences updated')
    } catch (err: unknown) {
      showError(getErrorMessage(err, 'Failed to update preferences'))
      // Revert on error
      setFormData(formData)
    } finally {
      setIsSaving(false)
    }
  }

  if (isLoading) {
    return (
      <div className="px-4 py-5 sm:p-6 flex items-center justify-center h-48">
        <Loader2 className="w-8 h-8 animate-spin text-primary-600" />
      </div>
    )
  }

  const notificationOptions: Array<{ key: keyof UserPreferences; label: string; description: string }> = [
    {
      key: 'emailQuotaAlert',
      label: 'Usage Alerts',
      description: 'Get notified when you approach your API quota limit',
    },
    {
      key: 'emailSecurityAlert',
      label: 'Security Alerts',
      description: 'Receive notifications about security events on your account',
    },
    {
      key: 'emailMarketing',
      label: 'Product Updates',
      description: 'Stay informed about new features and improvements',
    },
    {
      key: 'emailMonthlyReport',
      label: 'Monthly Reports',
      description: 'Receive a monthly summary of your API usage',
    },
  ]

  return (
    <div className="px-4 py-5 sm:p-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg leading-6 font-medium text-gray-900">
            Notification Preferences
          </h3>
          <p className="mt-1 text-sm text-gray-500">
            Choose how you want to be notified
          </p>
        </div>
        {isSaving && (
          <Loader2 className="w-5 h-5 animate-spin text-primary-600" aria-label="Saving..." />
        )}
      </div>

      <div className="mt-6 space-y-4">
        {notificationOptions.map(({ key, label, description }) => (
          <div
            key={String(key)}
            className="flex items-start"
          >
            <div className="flex items-center h-5">
              <input
                id={String(key)}
                type="checkbox"
                checked={formData?.[key] ?? false}
                onChange={() => handleToggle(key)}
                disabled={isSaving}
                className="focus:ring-primary-500 h-4 w-4 text-primary-600 border-gray-300 rounded disabled:opacity-50"
              />
            </div>
            <div className="ml-3 text-sm">
              <label htmlFor={String(key)} className="font-medium text-gray-700">
                {label}
              </label>
              <p className="text-gray-500">{description}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
