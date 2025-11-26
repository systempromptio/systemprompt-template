/**
 * SubmissionState component.
 *
 * Displays success message and reset button after form submission.
 *
 * @module artifacts/renderers/SubmissionState
 */

import React from 'react'

interface SubmissionStateProps {
  submitAction?: string
  onReset: () => void
}

export const SubmissionState = React.memo(function SubmissionState({
  submitAction,
  onReset,
}: SubmissionStateProps) {
  return (
    <div className="text-center py-8">
      <div className="text-success text-lg font-medium mb-2">Form submitted successfully!</div>
      <div className="text-sm text-secondary">Data would be sent to: {submitAction || 'Not specified'}</div>
      <button
        onClick={onReset}
        className="mt-4 px-4 py-2 bg-primary text-inverted rounded hover:bg-secondary"
      >
        Submit Another
      </button>
    </div>
  )
})
