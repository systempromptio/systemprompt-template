import type { FC } from 'react'

interface ConnectionErrorProps {
  error: Error
  isRetrying: boolean
  onRetry: () => void
}

export const ConnectionError: FC<ConnectionErrorProps> = ({ error, isRetrying, onRetry }) => {
  return (
    <div className="flex h-full items-center justify-center">
      <div className="text-center max-w-md">
        <div className="text-xl font-semibold text-red-600">
          {isRetrying ? 'Reconnecting...' : 'Connection Error'}
        </div>
        <div className="mt-2 text-sm text-gray-500">{error.toString()}</div>
        {!isRetrying && (
          <button
            onClick={onRetry}
            className="mt-4 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            Retry Connection
          </button>
        )}
        {isRetrying && (
          <div className="mt-4 flex items-center justify-center gap-2">
            <svg className="animate-spin h-5 w-5 text-blue-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <span className="text-sm text-blue-600">Attempting to reconnect...</span>
          </div>
        )}
      </div>
    </div>
  )
}
