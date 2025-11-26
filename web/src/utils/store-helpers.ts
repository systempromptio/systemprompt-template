/**
 * Object with creation/update timestamps for deduplication and conflict resolution.
 */
export interface Timestamped {
  created_at?: string | number
  updated_at?: string | number
}

/**
 * Compare timestamps of two items to determine recency.
 * Prefers updated_at over created_at for determining item age.
 *
 * @param newItem - New item to check
 * @param existingItem - Existing item to compare against
 * @returns True if newItem is newer than existingItem
 */
export function isNewerThan(
  newItem: Timestamped,
  existingItem: Timestamped
): boolean {
  const existingTime = new Date(
    existingItem.updated_at || existingItem.created_at || 0
  ).getTime()

  const newTime = new Date(
    newItem.updated_at || newItem.created_at || 0
  ).getTime()

  return newTime > existingTime
}

/**
 * Determine if a new item should replace an existing item.
 * Always replaces if no existing item. Otherwise compares timestamps.
 * Used for deduplication in stores where newer items should win conflicts.
 *
 * @param newItem - New item to potentially add
 * @param existingItem - Existing item (if any)
 * @returns True if new item should replace existing item
 */
export function shouldReplaceItem<T extends Timestamped>(
  newItem: T,
  existingItem: T | undefined
): boolean {
  if (!existingItem) return true
  return isNewerThan(newItem, existingItem)
}
