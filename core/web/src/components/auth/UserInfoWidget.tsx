import { LogOut, Fingerprint } from 'lucide-react';
import { useAuth } from '@/hooks/useAuth';
import { useAuthStore } from '@/stores/auth.store';
import { RoleBadge } from './RoleBadge';
import { Avatar, SectionTitle } from '@/components/ui';

interface UserInfoWidgetProps {
  compact?: boolean;
}

export function UserInfoWidget({ compact = false }: UserInfoWidgetProps) {
  const { isRealUser, email, username, primaryRole, logout, showLogin } = useAuth();
  const userId = useAuthStore((state) => state.userId);

  const displayName = isRealUser
    ? (username || email || 'User')
    : 'Anonymous';

  if (compact) {
    return (
      <button
        onClick={isRealUser ? logout : () => showLogin()}
        className="flex items-center justify-center min-w-[44px] min-h-[44px] border border-primary/30 transition-all duration-fast hover:border-primary/60 hover:scale-105 cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2"
        style={{ borderRadius: '18px 6px 18px 18px', background: 'transparent' }}
        title={isRealUser ? 'Sign Out' : 'Sign In'}
      >
        <Avatar
          username={username}
          email={email}
          userId={userId}
          size="sm"
          clickable={true}
        />
      </button>
    );
  }

  return (
    <button
      onClick={isRealUser ? logout : () => showLogin()}
      className="group relative flex items-center gap-sm px-md py-sm border border-primary/30 text-text-secondary hover:border-primary/60 hover:scale-105 transition-all duration-fast cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 w-full"
      style={{ borderRadius: '18px 6px 18px 18px', background: 'transparent' }}
      title={isRealUser ? 'Sign Out' : 'Sign In'}
    >
      <Avatar
        username={username}
        email={email}
        userId={userId}
        size="sm"
        clickable={true}
      />
      <SectionTitle className="truncate">
        {displayName}
      </SectionTitle>
      <RoleBadge role={primaryRole} size="sm" />
      <div className="flex-1" />
      {isRealUser ? (
        <LogOut className="w-4 h-4 text-text-secondary flex-shrink-0" />
      ) : (
        <Fingerprint className="w-4 h-4 text-primary flex-shrink-0" />
      )}
    </button>
  );
}
