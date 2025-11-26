import { useState } from 'react'
import { Send, X } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import type { Message } from '@a2a-js/sdk'

interface SmartInputPromptProps {
  taskId: string
  message?: Message
  onSubmit: (response: string) => void
  onCancel: () => void
}

export function SmartInputPrompt({
  taskId,
  message,
  onSubmit,
  onCancel
}: SmartInputPromptProps) {
  const [input, setInput] = useState('')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (input.trim()) {
      onSubmit(input.trim())
      setInput('')
    }
  }

  const messageText = message?.parts?.find(p => p.kind === 'text')?.text || ''
  const dataContent = message?.parts?.find(p => p.kind === 'data')?.data

  return (
    <div className="my-3 p-4 bg-gradient-to-r from-amber-50 to-orange-50 border-2 border-amber-300 rounded-lg shadow-md animate-slideInUp">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 bg-amber-500 rounded-full animate-pulse" />
          <h4 className="font-semibold text-amber-900">Agent Needs Input</h4>
        </div>
        <button
          onClick={onCancel}
          className="p-1 hover:bg-amber-200 rounded transition-colors"
        >
          <X className="w-4 h-4 text-amber-700" />
        </button>
      </div>

      {messageText && (
        <p className="text-sm text-amber-800 mb-3">{messageText}</p>
      )}

      {dataContent && (
        <div className="mb-3 p-3 bg-white rounded border border-amber-200">
          <pre className="text-xs text-gray-700 overflow-x-auto">
            {JSON.stringify(dataContent, null, 2)}
          </pre>
        </div>
      )}

      <form onSubmit={handleSubmit} className="flex gap-2">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Type your response..."
          className="flex-1 px-3 py-2 border border-amber-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
          autoFocus
        />
        <button
          type="submit"
          disabled={!input.trim()}
          className={cn(
            'px-4 py-2 bg-amber-500 text-white rounded-lg font-medium transition-all',
            'hover:bg-amber-600 active:scale-95',
            'disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-amber-500'
          )}
        >
          <Send className="w-4 h-4" />
        </button>
      </form>

      <div className="mt-2 text-xs text-amber-600">
        Task ID: {taskId}
      </div>
    </div>
  )
}
