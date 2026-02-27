import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'
import type { User } from '../types'

interface AuthState {
  user: User | null
  token: string | null
  isAuthenticated: boolean
  rememberMe: boolean
  setAuth: (user: User, token: string) => void
  setUser: (user: User) => void
  clearAuth: () => void
  updateUser: (user: Partial<User>) => void
  setRememberMe: (value: boolean) => void
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      token: null,
      isAuthenticated: false,
      rememberMe: true,
      setAuth: (user, token) => set({ user, token, isAuthenticated: true }),
      setUser: (user) => set({ user }),
      clearAuth: () => set({ user: null, token: null, isAuthenticated: false }),
      updateUser: (updates) =>
        set((state) => ({
          user: state.user ? { ...state.user, ...updates } : null,
        })),
      setRememberMe: (value) => set({ rememberMe: value }),
    }),
    {
      name: 'pii-redacta-auth',
      storage: createJSONStorage(() => localStorage),
      // Only persist auth data if rememberMe is true
      partialize: (state) => {
        if (state.rememberMe) {
          return {
            user: state.user,
            token: state.token,
            isAuthenticated: state.isAuthenticated,
            rememberMe: state.rememberMe,
          }
        }
        // If not remembering, only persist the preference itself
        return { rememberMe: state.rememberMe }
      },
    }
  )
)
