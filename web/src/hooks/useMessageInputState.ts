/**
 * Hook for managing message input state.
 *
 * Handles message text, files, and character counting.
 *
 * @module hooks/useMessageInputState
 */

import { useState, useRef, type KeyboardEvent } from 'react'

export function useMessageInputState() {
  const [message, setMessage] = useState('')
  const [files, setFiles] = useState<File[]>([])
  const fileInputRef = useRef<HTMLInputElement>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const handleSend = (onSend: (message: string, files?: File[]) => void) => {
    if (message.trim() || files.length > 0) {
      onSend(message.trim(), files)
      setMessage('')
      setFiles([])
      textareaRef.current?.focus()
    }
  }

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>, onSend: (message: string, files?: File[]) => void) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend(onSend)
    }
  }

  const handleFocus = () => {
    setTimeout(() => {
      textareaRef.current?.scrollIntoView({ behavior: 'smooth', block: 'center' })
    }, 300)
  }

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = Array.from(e.target.files || [])
    setFiles((prev) => [...prev, ...selectedFiles])
    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }

  const removeFile = (index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index))
  }

  const charCount = message.length
  const showCharCount = charCount > 500
  const isNearLimit = charCount > 1500

  return {
    message,
    setMessage,
    files,
    setFiles,
    fileInputRef,
    textareaRef,
    handleSend,
    handleKeyDown,
    handleFocus,
    handleFileSelect,
    removeFile,
    charCount,
    showCharCount,
    isNearLimit,
  }
}
