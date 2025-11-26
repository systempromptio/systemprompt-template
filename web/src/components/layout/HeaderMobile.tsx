import React, { useState, useRef, useEffect } from 'react';
import { Menu, LogOut, LogIn } from 'lucide-react';
import { cn } from '@/lib/utils/cn';
import { useAuth } from '@/hooks/useAuth';
import { useAuthStore } from '@/stores/auth.store';
import { RoleBadge } from '@/components/auth/RoleBadge';
import { Avatar } from '@/components/ui/Avatar';

interface HeaderMobileProps {
  onMenuClick: () => void;
  centerContent?: React.ReactNode;
}

export const HeaderMobile: React.FC<HeaderMobileProps> = ({
  onMenuClick,
  centerContent,
}) => {
  const [showUserMenu, setShowUserMenu] = useState(false);
  const userMenuRef = useRef<HTMLDivElement>(null);
  const { isRealUser, email, username, primaryRole, logout, showLogin } = useAuth();
  const userId = useAuthStore((state) => state.userId);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (userMenuRef.current && !userMenuRef.current.contains(event.target as Node)) {
        setShowUserMenu(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <header
      className="md:hidden flex items-center justify-between px-md py-sm border-b border-primary/10 bg-surface relative w-full"
      style={{
        minHeight: '56px',
        zIndex: 100,
      }}
    >
      <button
        onClick={onMenuClick}
        className={cn(
          'p-xs rounded-lg transition-all duration-fast',
          'hover:bg-primary/10 active:scale-95',
          'min-w-[44px] min-h-[44px] flex items-center justify-center'
        )}
        aria-label="Open menu"
      >
        <Menu className="w-6 h-6 text-primary" />
      </button>

      <div className="flex-1 flex items-center justify-center px-sm">
        {centerContent}
      </div>

      <div className="relative" ref={userMenuRef}>
        <button
          onClick={() => setShowUserMenu(!showUserMenu)}
          className={cn(
            'transition-all duration-fast active:scale-95',
            'min-w-[44px] min-h-[44px] flex items-center justify-center'
          )}
          aria-label="User menu"
        >
          <Avatar
            username={username}
            email={email}
            userId={userId}
            size="sm"
            clickable={false}
            showGlow={isRealUser}
            animated={isRealUser}
          />
        </button>

        {showUserMenu && (
          <div className="absolute right-0 top-full mt-sm bg-surface border border-primary/20 rounded-lg shadow-lg min-w-[200px] z-modal">
            {isRealUser && (
              <div className="px-md py-sm border-b border-primary/10">
                <div className="text-sm font-medium text-text-primary truncate mb-xs">
                  {username || email || 'User'}
                </div>
                <RoleBadge role={primaryRole} size="sm" />
              </div>
            )}

            {isRealUser ? (
              <button
                onClick={() => {
                  logout();
                  setShowUserMenu(false);
                }}
                className="w-full flex items-center gap-sm px-md py-sm text-sm text-text-primary hover:bg-primary/10 transition-fast"
              >
                <LogOut className="w-4 h-4" />
                <span>Logout</span>
              </button>
            ) : (
              <button
                onClick={() => {
                  showLogin();
                  setShowUserMenu(false);
                }}
                className="w-full flex items-center gap-sm px-md py-sm text-sm text-text-primary hover:bg-primary/10 transition-fast"
              >
                <LogIn className="w-4 h-4" />
                <span>Login</span>
              </button>
            )}
          </div>
        )}
      </div>
    </header>
  );
};

export default HeaderMobile;
