/**
 * Accessibility utilities and hooks for keyboard navigation and focus management
 */

import { useEffect, useRef } from 'react'

/**
 * Hook for managing focus trap within a modal or dialog
 * Keeps focus within the element and optionally returns focus on unmount
 */
export function useFocusTrap(
  ref: React.RefObject<HTMLElement | null>,
  options: { enabled?: boolean; returnFocus?: boolean } = {}
) {
  const { enabled = true, returnFocus = true } = options
  const previousActiveElement = useRef<Element | null>(null)

  useEffect(() => {
    if (!enabled || !ref.current) return

    previousActiveElement.current = document.activeElement as Element | null

    const element = ref.current
    const focusableElements = element.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    )

    const firstElement = focusableElements[0] as HTMLElement
    const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return

      if (e.shiftKey) {
        if (document.activeElement === firstElement) {
          e.preventDefault()
          lastElement?.focus()
        }
      } else {
        if (document.activeElement === lastElement) {
          e.preventDefault()
          firstElement?.focus()
        }
      }
    }

    if (firstElement) {
      firstElement.focus()
    }

    element.addEventListener('keydown', handleKeyDown)

    return () => {
      element.removeEventListener('keydown', handleKeyDown)
      if (returnFocus && previousActiveElement.current instanceof HTMLElement) {
        previousActiveElement.current.focus()
      }
    }
  }, [enabled, returnFocus])
}

/**
 * Keyboard shortcut configuration
 */
interface KeyboardShortcut {
  key: string
  ctrl?: boolean
  shift?: boolean
  alt?: boolean
  callback: (e: KeyboardEvent) => void
  description?: string
}

/**
 * Hook for registering keyboard shortcuts
 */
export function useKeyboardShortcuts(
  shortcuts: KeyboardShortcut[],
  enabled = true
) {
  useEffect(() => {
    if (!enabled || shortcuts.length === 0) return

    const handleKeyDown = (e: KeyboardEvent) => {
      for (const shortcut of shortcuts) {
        const keyMatches = e.key.toLowerCase() === shortcut.key.toLowerCase()
        const ctrlMatches = shortcut.ctrl ? e.ctrlKey || e.metaKey : !e.ctrlKey && !e.metaKey
        const shiftMatches = shortcut.shift ? e.shiftKey : !e.shiftKey
        const altMatches = shortcut.alt ? e.altKey : !e.altKey

        if (keyMatches && ctrlMatches && shiftMatches && altMatches) {
          shortcut.callback(e)
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [shortcuts, enabled])
}

/**
 * Arrow navigation configuration
 */
interface ArrowNavigationOptions {
  orientation?: 'horizontal' | 'vertical'
  loop?: boolean
}

/**
 * Hook for arrow key navigation between focusable elements
 */
export function useArrowNavigation(
  itemRefs: React.MutableRefObject<HTMLDivElement[]>,
  options: ArrowNavigationOptions = {}
) {
  const { orientation = 'vertical', loop = false } = options

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const refs = itemRefs.current
      if (!refs || refs.length === 0) return

      const currentElement = document.activeElement as HTMLElement
      const currentIndex = refs.indexOf(currentElement as HTMLDivElement)

      let nextIndex = -1

      if (orientation === 'vertical') {
        if (e.key === 'ArrowDown') {
          e.preventDefault()
          nextIndex = currentIndex + 1
          if (nextIndex >= refs.length) {
            nextIndex = loop ? 0 : refs.length - 1
          }
        } else if (e.key === 'ArrowUp') {
          e.preventDefault()
          nextIndex = currentIndex - 1
          if (nextIndex < 0) {
            nextIndex = loop ? refs.length - 1 : 0
          }
        }
      } else if (orientation === 'horizontal') {
        if (e.key === 'ArrowRight') {
          e.preventDefault()
          nextIndex = currentIndex + 1
          if (nextIndex >= refs.length) {
            nextIndex = loop ? 0 : refs.length - 1
          }
        } else if (e.key === 'ArrowLeft') {
          e.preventDefault()
          nextIndex = currentIndex - 1
          if (nextIndex < 0) {
            nextIndex = loop ? refs.length - 1 : 0
          }
        }
      }

      if (nextIndex >= 0 && nextIndex < refs.length) {
        const nextElement = refs[nextIndex]
        const focusableChild = nextElement.querySelector('button, [href], input, [tabindex]') as HTMLElement
        if (focusableChild) {
          focusableChild.focus()
        } else {
          nextElement.focus()
        }
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [itemRefs, orientation, loop])
}
