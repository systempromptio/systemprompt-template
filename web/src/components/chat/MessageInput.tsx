import { Send, Square } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import { useThemeValues } from '@/theme'
import { useMessageInputState } from '@/hooks/useMessageInputState'
import { FilePreview } from './FilePreview'
import { CharacterCounter } from './CharacterCounter'
import { InputFooter } from './InputFooter'

interface MessageInputProps {
  onSend: (message: string, files?: File[]) => void
  disabled?: boolean
  isStreaming?: boolean
  onStopStreaming?: () => void
}

export function MessageInput({ onSend, disabled, isStreaming, onStopStreaming }: MessageInputProps) {
  const theme = useThemeValues()
  const { message, setMessage, files, fileInputRef, textareaRef, handleSend: executeSend, handleKeyDown: executeKeyDown, handleFocus, handleFileSelect, removeFile, charCount, showCharCount, isNearLimit } = useMessageInputState()

  const handleSend = () => executeSend(onSend)
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => executeKeyDown(e, onSend)

  return (
    <div className="border-t border-primary/10">
      <FilePreview files={files} onRemove={removeFile} />

      <div className="px-lg py-lg">
        <div className="relative">
          <textarea
            ref={textareaRef}
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyDown={handleKeyDown}
            onFocus={handleFocus}
            placeholder={isStreaming ? 'AI is responding...' : 'Type a message...'}
            disabled={disabled || isStreaming}
            className={cn(
              'w-full px-lg py-md pr-16 resize-none',
              'bg-transparent text-text-primary placeholder:text-text-secondary',
              'border border-primary/30',
              'text-lg leading-relaxed font-body',
              'min-h-[56px] max-h-40',
              'transition-all duration-fast',
              'disabled:cursor-not-allowed disabled:opacity-50',
              'caret-primary',
              'focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 focus:border-primary'
            )}
            style={{
              borderRadius: `${theme.components.card.borderRadius.default}px ${theme.components.card.borderRadius.topRight}px ${theme.components.card.borderRadius.default}px ${theme.components.card.borderRadius.default}px`,
              background: 'transparent',
              height: 'auto',
              overflowY: message.split('\n').length > 4 ? 'auto' : 'hidden',
            }}
            rows={1}
          />

          <CharacterCounter charCount={charCount} isVisible={showCharCount} isNearLimit={isNearLimit} />

          {isStreaming ? (
            <button
              onClick={onStopStreaming}
              className={cn(
                'absolute right-2 bottom-2',
                'flex items-center justify-center',
                'w-12 h-12 rounded-full',
                'text-error',
                'transition-all duration-fast',
                'cursor-pointer',
                'hover:text-error/80 hover:scale-110',
                'focus:outline-none focus:ring-2 focus:ring-error focus:ring-offset-2'
              )}
              aria-label="Stop generating"
            >
              <Square className="w-5 h-5" />
            </button>
          ) : (
            <button
              onClick={handleSend}
              disabled={disabled || (!message.trim() && files.length === 0)}
              className={cn(
                'absolute right-2 bottom-2',
                'flex items-center justify-center',
                'w-12 h-12 rounded-full',
                'transition-all duration-fast',
                'focus:outline-none',
                (disabled || (!message.trim() && files.length === 0)) && ['text-text-disabled', 'cursor-not-allowed opacity-50'],
                !(disabled || (!message.trim() && files.length === 0)) && ['text-primary', 'cursor-pointer', 'hover:text-primary/80 hover:scale-110']
              )}
              aria-label="Send message"
            >
              <Send className="w-5 h-5" />
            </button>
          )}
        </div>

        <InputFooter disabled={disabled} isStreaming={isStreaming} onAttachClick={() => fileInputRef.current?.click()} fileInputRef={fileInputRef} />
      </div>

      <input ref={fileInputRef} type="file" multiple onChange={handleFileSelect} className="hidden" disabled={disabled || isStreaming} />
    </div>
  )
}
