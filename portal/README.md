# PII Redacta Portal

React-based web portal for managing PII Redacta accounts.

## Tech Stack

- **Framework**: React 18 + TypeScript
- **Build Tool**: Vite
- **Styling**: Tailwind CSS
- **State Management**: Zustand
- **Data Fetching**: TanStack Query (React Query)
- **Routing**: React Router
- **Icons**: Lucide React

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build
```

## Project Structure

```
portal/
├── src/
│   ├── components/     # Reusable UI components
│   ├── pages/         # Page components
│   ├── hooks/         # Custom React hooks
│   ├── stores/        # Zustand state stores
│   ├── lib/           # Utility functions, API client
│   ├── types/         # TypeScript type definitions
│   ├── App.tsx        # Main app component
│   ├── main.tsx       # Entry point
│   └── index.css      # Global styles + Tailwind
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.js
```

## Features

- **Authentication**: JWT-based auth with automatic token refresh
- **Dashboard**: Overview of usage, limits, and quick actions
- **API Key Management**: Create, view, revoke API keys
- **Usage Analytics**: Track requests, files, pages processed
- **Billing**: View and upgrade subscription plans
- **Settings**: Profile, password, notifications

## Environment Variables

Create a `.env` file in the portal directory:

```env
VITE_API_URL=http://localhost:8080/api/v1
```

The development server proxies `/api` requests to `http://localhost:8080`.
