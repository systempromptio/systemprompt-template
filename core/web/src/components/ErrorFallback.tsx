import { useRouteError, isRouteErrorResponse } from 'react-router-dom'

export function ErrorFallback() {
  const error = useRouteError()

  let errorMessage = 'An unexpected error occurred'
  let errorCode = '500'
  let errorDetails = ''

  if (isRouteErrorResponse(error)) {
    errorCode = error.status.toString()
    errorMessage = error.statusText || errorMessage

    if (error.data?.message) {
      errorDetails = error.data.message
    }
  } else if (error instanceof Error) {
    errorMessage = error.message
    errorDetails = error.stack || ''
  } else if (typeof error === 'string') {
    errorMessage = error
  }

  return (
    <div style={{
      padding: '40px 20px',
      fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      backgroundColor: '#f5f5f5',
      minHeight: '100vh',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
    }}>
      <div style={{
        maxWidth: '600px',
        backgroundColor: 'white',
        borderRadius: '12px',
        boxShadow: '0 2px 8px rgba(0,0,0,0.1)',
        padding: '40px',
        textAlign: 'center',
      }}>
        <div style={{ fontSize: '48px', marginBottom: '16px' }}>⚠️</div>

        <h1 style={{
          margin: '0 0 12px 0',
          fontSize: '24px',
          color: '#1a1a1a',
          fontWeight: '600',
        }}>
          {errorCode} - {errorMessage}
        </h1>

        {errorDetails && (
          <details style={{
            marginTop: '20px',
            textAlign: 'left',
            padding: '12px',
            backgroundColor: '#f9f9f9',
            borderRadius: '6px',
            border: '1px solid #e0e0e0',
          }}>
            <summary style={{
              cursor: 'pointer',
              fontWeight: '500',
              color: '#666',
              userSelect: 'none',
            }}>
              Error Details
            </summary>
            <pre style={{
              marginTop: '12px',
              overflowX: 'auto',
              fontSize: '12px',
              color: '#333',
              fontFamily: 'monospace',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
            }}>
              {errorDetails}
            </pre>
          </details>
        )}

        <p style={{
          margin: '20px 0 0 0',
          color: '#666',
          fontSize: '14px',
          lineHeight: '1.6',
        }}>
          Something went wrong. Try refreshing the page or going back home.
        </p>

        <div style={{
          display: 'flex',
          gap: '12px',
          justifyContent: 'center',
          marginTop: '24px',
        }}>
          <button
            onClick={() => window.location.reload()}
            style={{
              padding: '10px 20px',
              backgroundColor: '#4c6ef5',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              fontSize: '14px',
              fontWeight: '500',
              cursor: 'pointer',
              transition: 'background-color 0.2s',
            }}
            onMouseOver={(e) => (e.currentTarget.style.backgroundColor = '#3d5ae1')}
            onMouseOut={(e) => (e.currentTarget.style.backgroundColor = '#4c6ef5')}
          >
            Reload Page
          </button>

          <button
            onClick={() => window.location.href = '/'}
            style={{
              padding: '10px 20px',
              backgroundColor: '#e9ecef',
              color: '#1a1a1a',
              border: 'none',
              borderRadius: '6px',
              fontSize: '14px',
              fontWeight: '500',
              cursor: 'pointer',
              transition: 'background-color 0.2s',
            }}
            onMouseOver={(e) => (e.currentTarget.style.backgroundColor = '#dee2e6')}
            onMouseOut={(e) => (e.currentTarget.style.backgroundColor = '#e9ecef')}
          >
            Go Home
          </button>
        </div>
      </div>
    </div>
  )
}
