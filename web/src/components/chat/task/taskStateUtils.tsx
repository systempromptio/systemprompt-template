/**
 * Task state utilities and helpers.
 *
 * @module chat/task/taskStateUtils
 */

import React from 'react'
import { Loader2, CheckCircle2, XCircle, AlertCircle, Lock, MessageSquare, Ban, Circle } from 'lucide-react'
import type { TaskState, Message } from '@a2a-js/sdk'

interface TextPart {
  kind: 'text'
  text: string
}

export interface StateInfo {
  icon: React.JSX.Element
  label: string
  iconColor: string
  textColor: string
  bgColor: string
  borderColor: string
}

export function getStateInfo(state: TaskState): StateInfo {
  switch (state) {
    case 'submitted':
      return {
        icon: <Circle className="w-5 h-5 animate-pulse" />,
        label: 'Submitted',
        iconColor: 'text-gray-400',
        textColor: 'text-gray-700',
        bgColor: 'bg-gray-50',
        borderColor: 'border-gray-200',
      }
    case 'working':
      return {
        icon: <Loader2 className="w-5 h-5 animate-spin" />,
        label: 'Working...',
        iconColor: 'text-blue-500',
        textColor: 'text-blue-700',
        bgColor: 'bg-blue-50',
        borderColor: 'border-blue-200',
      }
    case 'input-required':
      return {
        icon: <MessageSquare className="w-5 h-5 animate-pulse" />,
        label: 'Input Required',
        iconColor: 'text-amber-500',
        textColor: 'text-amber-700',
        bgColor: 'bg-amber-50',
        borderColor: 'border-amber-200',
      }
    case 'completed':
      return {
        icon: <CheckCircle2 className="w-5 h-5" />,
        label: 'Completed',
        iconColor: 'text-green-500',
        textColor: 'text-green-700',
        bgColor: 'bg-green-50',
        borderColor: 'border-green-200',
      }
    case 'canceled':
      return {
        icon: <Ban className="w-5 h-5" />,
        label: 'Canceled',
        iconColor: 'text-yellow-500',
        textColor: 'text-yellow-700',
        bgColor: 'bg-yellow-50',
        borderColor: 'border-yellow-200',
      }
    case 'failed':
      return {
        icon: <XCircle className="w-5 h-5" />,
        label: 'Failed',
        iconColor: 'text-red-500',
        textColor: 'text-red-700',
        bgColor: 'bg-red-50',
        borderColor: 'border-red-200',
      }
    case 'rejected':
      return {
        icon: <AlertCircle className="w-5 h-5" />,
        label: 'Rejected',
        iconColor: 'text-red-500',
        textColor: 'text-red-700',
        bgColor: 'bg-red-50',
        borderColor: 'border-red-200',
      }
    case 'auth-required':
      return {
        icon: <Lock className="w-5 h-5 animate-pulse" />,
        label: 'Authentication Required',
        iconColor: 'text-purple-500',
        textColor: 'text-purple-700',
        bgColor: 'bg-purple-50',
        borderColor: 'border-purple-200',
      }
    default:
      return {
        icon: <AlertCircle className="w-5 h-5" />,
        label: 'Unknown',
        iconColor: 'text-gray-400',
        textColor: 'text-gray-700',
        bgColor: 'bg-gray-50',
        borderColor: 'border-gray-200',
      }
  }
}

export function extractMessageText(message: Message | string | null | undefined): string {
  if (typeof message === 'string') return message
  if (!message || !message.parts) return ''

  const textPart = message.parts.find((p): p is TextPart => p.kind === 'text')
  return textPart?.text || ''
}
