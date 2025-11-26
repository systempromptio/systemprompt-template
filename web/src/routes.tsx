import type { RouteObject } from 'react-router-dom';
import App from './App';
import { ErrorFallback } from './components/ErrorFallback';

export const routes: RouteObject[] = [
  {
    path: '/',
    element: <App />,
    errorElement: <ErrorFallback />,
  },
];

// Note: All documentation and legal routes are served as static HTML by Rust backend
// This React Router only handles the interactive chat SPA at "/"
