import { Eye, EyeOff } from 'lucide-react'
import { useSettingsStore } from '@/stores/settings.store'
import { cn } from '@/lib/utils/cn'

export function DebugModeToggle() {
  const debugMode = useSettingsStore((state) => state.debugMode)
  const setDebugMode = useSettingsStore((state) => state.setDebugMode)

  return (
    <button
      onClick={() => setDebugMode(!debugMode)}
      className={cn(
        'flex items-center gap-2 px-3 py-2 rounded-lg transition-all',
        'border border-primary-10 hover:bg-surface-variant',
        debugMode && 'bg-primary/10 border-primary-30'
      )}
      title={debugMode ? 'Hide internal tools' : 'Show internal tools (debug mode)'}
    >
      {debugMode ? (
        <Eye className="w-4 h-4 text-primary" />
      ) : (
        <EyeOff className="w-4 h-4 text-text-secondary" />
      )}
      <span className={cn(
        'text-sm font-medium',
        debugMode ? 'text-primary' : 'text-text-secondary'
      )}>
        Debug Mode
      </span>
    </button>
  )
}
