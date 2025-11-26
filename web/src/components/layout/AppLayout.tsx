import React, { useState, useEffect } from 'react';
import { GradientBackground } from './GradientBackground';
import { MobileDrawer } from './MobileDrawer';

interface AppLayoutProps {
  header: React.ReactNode;
  leftSidebar?: React.ReactNode;
  children: React.ReactNode;
  showLeftSidebar?: boolean;
  activeView?: 'conversation' | 'tasks' | 'artifacts';
  onViewChange?: (view: 'conversation' | 'tasks' | 'artifacts') => void;
  mobileMenuOpen: boolean;
  onMobileMenuClose: () => void;
}

export const AppLayout: React.FC<AppLayoutProps> = ({
  header,
  leftSidebar,
  children,
  showLeftSidebar = true,
  mobileMenuOpen,
  onMobileMenuClose,
}) => {
  const [isMobile, setIsMobile] = useState(() => {
    if (typeof window === 'undefined') return false;
    return window.innerWidth < 768;
  });

  useEffect(() => {
    const handleResize = () => {
      setIsMobile(window.innerWidth < 768);
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  return (
    <GradientBackground>
      <div className="flex flex-col h-full w-full overflow-hidden">
        {/* Header - pinned at top */}
        <div className="flex-shrink-0">
          {header}
        </div>

        {/* Mobile drawer */}
        <MobileDrawer
          isOpen={mobileMenuOpen}
          onClose={onMobileMenuClose}
          side="left"
          title="MENU"
        >
          {leftSidebar}
        </MobileDrawer>

        {/* Conditional rendering: only render ONE layout at a time */}
        {isMobile ? (
          /* Mobile layout */
          <main className="flex-1 min-h-0 overflow-hidden">
            {children}
          </main>
        ) : (
          /* Desktop layout */
          <div className="flex-1 overflow-hidden grid" style={{
            gridTemplateColumns: showLeftSidebar ? 'minmax(240px, 25%) 1fr' : '0 1fr',
            transition: 'grid-template-columns 300ms ease-in-out',
          }}>
            <aside className="flex flex-col border-r border-primary/10 overflow-hidden">
              {showLeftSidebar && leftSidebar}
            </aside>
            <main className="flex flex-col overflow-hidden">
              {children}
            </main>
          </div>
        )}
      </div>
    </GradientBackground>
  );
};

export default AppLayout;
