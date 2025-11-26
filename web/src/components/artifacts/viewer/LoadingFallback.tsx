/**
 * Loading fallback component.
 *
 * Displays a spinning loader while artifact content loads.
 *
 * @module artifacts/viewer/LoadingFallback
 */

import React from 'react'

export const LoadingFallback = React.memo(function LoadingFallback() {
  return (
    <div className="flex items-center justify-center py-8">
      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
    </div>
  )
})
