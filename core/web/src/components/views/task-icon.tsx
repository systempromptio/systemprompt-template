/**
 * Task status icon renderer.
 *
 * Returns appropriate icon for task status states.
 *
 * @module components/views/task-icon
 */

import React from 'react'
import { CheckSquare, Clock, CheckCircle, XCircle } from 'lucide-react'

/**
 * Gets the status icon for a given task state.
 *
 * @param status - Task status state
 * @returns React element with appropriate icon
 */
export function getStatusIcon(status: string): React.ReactNode {
  switch (status) {
    case 'completed':
      return <CheckCircle className="w-5 h-5 text-success" />
    case 'failed':
    case 'rejected':
      return <XCircle className="w-5 h-5 text-error" />
    case 'working':
      return <Clock className="w-5 h-5 text-warning" />
    default:
      return <CheckSquare className="w-5 h-5 text-text-secondary" />
  }
}
