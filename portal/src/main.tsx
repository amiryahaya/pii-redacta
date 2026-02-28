import React from 'react'
import ReactDOM from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter } from 'react-router-dom'
import App from './App'
import { ErrorBoundary } from './components/ErrorBoundary'
import './index.css'

const SENTRY_DSN = import.meta.env.VITE_SENTRY_DSN

// Load Sentry lazily to avoid bundling ~200kB when DSN is not configured (M8)
if (SENTRY_DSN) {
  import('@sentry/react').then((Sentry) => {
    Sentry.init({
      dsn: SENTRY_DSN,
      environment: import.meta.env.MODE,
      release: import.meta.env.VITE_APP_VERSION || 'dev',
    })
  })
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      retry: 1,
    },
  },
})

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <App />
        </BrowserRouter>
      </QueryClientProvider>
    </ErrorBoundary>
  </React.StrictMode>,
)
