import React from 'react';
import { Menu } from 'lucide-react';
import { useSettingsStore } from '@/stores/settings.store';
import { theme } from '@/theme.config';

interface HeaderProps {
  centerContent?: React.ReactNode;
  rightContent?: React.ReactNode;
}

export const Header: React.FC<HeaderProps> = ({
  centerContent,
  rightContent,
}) => {
  const { toggleLeftSidebar } = useSettingsStore();

  return (
    <header className="hidden md:grid grid-cols-[auto_1fr_auto] h-header border-b border-primary/10 relative z-[100]">
      <div className="flex items-center gap-md px-md py-md">
        <button
          onClick={toggleLeftSidebar}
          className="p-2 rounded-md hover:bg-primary/5 text-primary/70 hover:text-primary transition-colors min-w-[44px] min-h-[44px] flex items-center justify-center"
          aria-label="Toggle menu"
        >
          <Menu size={20} />
        </button>
        <picture>
          <source srcSet="/assets/logos/logo.webp" type="image/webp" />
          <source srcSet="/assets/logos/logo.svg" type="image/svg+xml" />
          <img src="/assets/logos/logo.svg" alt={theme.branding.name} className="h-6 lg:h-8" />
        </picture>
      </div>

      <div className="flex items-center justify-center px-md py-md">
        {centerContent}
      </div>

      <div className="flex items-center px-md py-md">
        {rightContent}
      </div>
    </header>
  );
};

export default Header;
