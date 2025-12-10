import { Lock, ExternalLink } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { Icon } from '@/components/ui/Icon'
import { Card } from '@/components/ui/Card'
import type { Message } from '@a2a-js/sdk'

interface AuthRequiredPromptProps {
  taskId: string
  message?: Message
  onAuthorize: () => void
  onCancel: () => void
}

export function AuthRequiredPrompt({
  taskId,
  message,
  onAuthorize,
  onCancel
}: AuthRequiredPromptProps) {
  const messageText = message?.parts?.find(p => p.kind === 'text')?.text || ''
  const dataContent = message?.parts?.find(p => p.kind === 'data')?.data

  interface AuthInfoData {
    schemes?: string[]
    service?: string
    permissions?: string[]
    url?: string
  }

  const extractAuthInfo = (data: unknown): AuthInfoData | null => {
    if (!data || typeof data !== 'object') return null

    const record = data as Record<string, unknown>
    return {
      schemes: Array.isArray(record.schemes) ? record.schemes as string[] : [],
      service: typeof record.service === 'string' ? record.service : 'external service',
      permissions: Array.isArray(record.permissions) ? record.permissions as string[] : [],
      url: typeof record.url === 'string' ? record.url : undefined
    }
  }

  const authInfo = extractAuthInfo(dataContent)

  return (
    <Card variant="accent" padding="md" elevation="md" className="my-sm animate-slideInUp">
      <div className="flex items-start justify-between mb-sm">
        <div className="flex items-center gap-sm">
          <Icon icon={Lock} size="md" color="primary" className="animate-pulse" />
          <h4 className="font-heading font-semibold text-text-primary">Authentication Required</h4>
        </div>
        <Button
          variant="ghost"
          size="xs"
          icon={ExternalLink}
          onClick={onCancel}
          aria-label="Close"
        />
      </div>

      <div className="space-y-sm">
        {messageText && (
          <p className="text-sm text-text-primary">{messageText}</p>
        )}

        {authInfo && (
          <Card variant="default" padding="sm" elevation="none">
            <div className="space-y-xs">
              <div className="text-sm">
                <span className="font-medium text-text-primary">Service:</span>{' '}
                <span className="text-text-secondary">{authInfo.service}</span>
              </div>

              {authInfo.schemes && authInfo.schemes.length > 0 && (
                <div className="text-sm">
                  <span className="font-medium text-text-primary">Auth Methods:</span>{' '}
                  <span className="text-text-secondary">{authInfo.schemes.join(', ')}</span>
                </div>
              )}

              {authInfo.permissions && authInfo.permissions.length > 0 && (
                <div>
                  <div className="text-sm font-medium text-text-primary mb-xs">
                    Permissions Requested:
                  </div>
                  <ul className="list-disc list-inside text-sm text-text-secondary space-y-xs">
                    {authInfo.permissions.map((permission: string, idx: number) => (
                      <li key={idx}>{permission}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          </Card>
        )}

        <div className="flex gap-sm">
          <Button
            variant="primary"
            size="md"
            icon={Lock}
            iconPosition="left"
            onClick={onAuthorize}
            className="flex-1 shadow-md"
          >
            Grant Access
          </Button>

          <Button
            variant="secondary"
            size="md"
            onClick={onCancel}
          >
            Cancel
          </Button>
        </div>

        <div className="text-xs text-text-secondary">
          Task ID: {taskId}
        </div>
      </div>
    </Card>
  )
}
