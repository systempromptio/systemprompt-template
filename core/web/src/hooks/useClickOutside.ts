/**
 * Click outside detection hook.
 *
 * Detects clicks outside a referenced element and triggers callback.
 *
 * @module hooks/useClickOutside
 */

import { useEffect } from 'react'

/**
 * Hook for detecting clicks outside a referenced element.
 *
 * @param ref - React ref to element to watch
 * @param callback - Function called when click outside detected
 *
 * @example
 * ```typescript
 * const ref = useRef<HTMLDivElement>(null)
 * useClickOutside(ref, () => setOpen(false))
 * ```
 */
export function useClickOutside(
  ref: React.RefObject<HTMLElement | null>,
  callback: () => void
): void {
  useEffect(() => {
    function handleClickOutside(event: MouseEvent | TouchEvent): void {
      if (ref.current && !ref.current.contains(event.target as Node)) {
        callback()
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    document.addEventListener('touchstart', handleClickOutside)
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('touchstart', handleClickOutside)
    }
  }, [ref, callback])
}
