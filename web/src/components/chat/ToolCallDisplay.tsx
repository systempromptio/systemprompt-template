import { useState, useEffect } from 'react'
import { ChevronDown, ChevronRight, Check, Loader2, AlertCircle, ArrowRight } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import type { ToolCallExecution } from '@/stores/chat.store'

interface ToolCallDisplayProps {
  toolCalls: ToolCallExecution[]
  isAgenticMode?: boolean
  currentIteration?: number
}

export function ToolCallDisplay({ toolCalls, isAgenticMode, currentIteration }: ToolCallDisplayProps) {
  const [expandedCalls, setExpandedCalls] = useState<Set<string>>(new Set())
  const [newCallIds, setNewCallIds] = useState<Set<string>>(new Set())

  useEffect(() => {
    const latestCall = toolCalls[toolCalls.length - 1]
    if (latestCall && !latestCall.result) {
      setNewCallIds(new Set([latestCall.id]))
      const timer = setTimeout(() => setNewCallIds(new Set()), 2000)
      return () => clearTimeout(timer)
    }
  }, [toolCalls])

  const toggleExpanded = (id: string) => {
    setExpandedCalls((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  if (toolCalls.length === 0) return null

  return (
    <div className="space-y-2 my-2">
      {isAgenticMode && currentIteration && (
        <div className="text-xs text-gray-500 font-medium px-3 py-1.5 bg-blue-50 rounded-md inline-flex items-center gap-2 animate-slideInRight">
          <div className="w-1.5 h-1.5 bg-blue-500 rounded-full animate-pulse" />
          Iteration {currentIteration}
        </div>
      )}

      {toolCalls.map((toolCall, idx) => {
        const isExpanded = expandedCalls.has(toolCall.id)
        const isComplete = !!toolCall.result
        const isError = toolCall.result?.is_error ?? false
        const isNew = newCallIds.has(toolCall.id)
        const prevCall = idx > 0 ? toolCalls[idx - 1] : null

        return (
          <div key={toolCall.id}>
            {prevCall && (
              <div className="flex justify-center py-1">
                <ArrowRight className="w-4 h-4 text-gray-400" />
              </div>
            )}
            <div
              className={cn(
                'border rounded-lg overflow-hidden transition-all duration-300',
                isError ? 'border-red-200 bg-red-50' : 'border-blue-200 bg-blue-50',
                isNew && 'shadow-lg ring-2 ring-blue-300 animate-slideInUp'
              )}
            >
            {/* Tool Call Header */}
            <button
              onClick={() => toggleExpanded(toolCall.id)}
              className="w-full px-3 py-2.5 flex items-center gap-2 hover:bg-white/30 transition-colors"
            >
              {isExpanded ? (
                <ChevronDown className="w-4 h-4 text-gray-600" />
              ) : (
                <ChevronRight className="w-4 h-4 text-gray-600" />
              )}

              {/* Status Icon */}
              {isComplete ? (
                isError ? (
                  <AlertCircle className="w-4 h-4 text-red-600" />
                ) : (
                  <Check className="w-4 h-4 text-green-600" />
                )
              ) : (
                <Loader2 className="w-4 h-4 text-blue-600 animate-spin" />
              )}

              {/* Tool Name */}
              <span className="font-mono text-sm font-medium text-gray-700">
                {toolCall.name}
              </span>

              {/* Execution Time */}
              <span className="text-xs text-gray-500 ml-auto flex items-center gap-2">
                {isComplete ? (
                  <>
                    {isError ? 'Failed' : 'Complete'}
                    <span className="opacity-60">•</span>
                    <span>{formatDuration(toolCall)}</span>
                  </>
                ) : (
                  <span className="flex items-center gap-1.5">
                    Executing
                    <span className="flex gap-0.5">
                      <span className="w-1 h-1 bg-blue-600 rounded-full animate-pulse" style={{ animationDelay: '0ms' }} />
                      <span className="w-1 h-1 bg-blue-600 rounded-full animate-pulse" style={{ animationDelay: '150ms' }} />
                      <span className="w-1 h-1 bg-blue-600 rounded-full animate-pulse" style={{ animationDelay: '300ms' }} />
                    </span>
                  </span>
                )}
              </span>
            </button>

            {/* Tool Call Details (Expanded) */}
            {isExpanded && (
              <div className="border-t border-gray-200 p-3 space-y-2 bg-white">
                {/* Arguments */}
                <div>
                  <div className="text-xs font-medium text-gray-600 mb-1">Arguments:</div>
                  <pre className="text-xs bg-gray-50 p-2 rounded border border-gray-200 overflow-x-auto">
                    {JSON.stringify(toolCall.arguments, null, 2)}
                  </pre>
                </div>

                {/* Result */}
                {toolCall.result && (
                  <div>
                    <div className="text-xs font-medium text-gray-600 mb-1">Result:</div>
                    <pre className={cn(
                      "text-xs p-2 rounded border overflow-x-auto",
                      isError
                        ? "bg-red-50 border-red-200 text-red-800"
                        : "bg-gray-50 border-gray-200"
                    )}>
                      {typeof toolCall.result.content === 'string'
                        ? toolCall.result.content
                        : JSON.stringify(toolCall.result.content, null, 2)}
                    </pre>
                  </div>
                )}

                {/* Timestamp */}
                <div className="text-xs text-gray-400">
                  {toolCall.timestamp.toLocaleTimeString()}
                </div>
              </div>
            )}
          </div>
          </div>
        )
      })}
    </div>
  )
}

function formatDuration(toolCall: ToolCallExecution): string {
  if (!toolCall.result) return ''

  const start = toolCall.timestamp
  const end = new Date()
  const duration = end.getTime() - start.getTime()

  if (duration < 1000) return `${duration}ms`
  if (duration < 60000) return `${(duration / 1000).toFixed(1)}s`
  return `${(duration / 60000).toFixed(1)}m`
}
