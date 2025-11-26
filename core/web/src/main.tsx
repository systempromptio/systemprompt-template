import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import './index.css'
import { ThemeProvider } from './theme'
import { routes } from './routes'
import { ErrorBoundary } from './components/ErrorBoundary'
import { ErrorToastContainer } from './components/ErrorToast'
import { globalErrorHandler } from './lib/errorHandling'

globalErrorHandler.initialize()

const router = createBrowserRouter(routes)

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ErrorBoundary>
      <ThemeProvider>
        <RouterProvider router={router} />
        <ErrorToastContainer />
      </ThemeProvider>
    </ErrorBoundary>
  </StrictMode>
)
