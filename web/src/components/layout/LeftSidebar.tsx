import React, { useState } from 'react';
import { cn } from '@/lib/utils/cn';

interface LeftSidebarProps {
  agentsContent: React.ReactNode;
  toolsContent: React.ReactNode;
  footer?: React.ReactNode;
}

type TabType = 'agents' | 'tools';

export const LeftSidebar: React.FC<LeftSidebarProps> = ({
  agentsContent,
  toolsContent,
  footer,
}) => {
  const [activeTab, setActiveTab] = useState<TabType>('agents');

  return (
    <div className="flex flex-col h-full w-full">
      {/* Tabs */}
      <div className="flex border-b border-primary/10 bg-surface-variant/30">
        <button
          onClick={() => setActiveTab('agents')}
          className={cn(
            'flex-1 px-md py-sm text-sm font-heading font-medium uppercase tracking-wide transition-all',
            activeTab === 'agents'
              ? 'text-primary border-b-2 border-primary bg-surface'
              : 'text-text-secondary hover:text-primary hover:bg-primary/5'
          )}
        >
          Agents
        </button>
        <button
          onClick={() => setActiveTab('tools')}
          className={cn(
            'flex-1 px-md py-sm text-sm font-heading font-medium uppercase tracking-wide transition-all',
            activeTab === 'tools'
              ? 'text-primary border-b-2 border-primary bg-surface'
              : 'text-text-secondary hover:text-primary hover:bg-primary/5'
          )}
        >
          Tools
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto scrollbar-thin">
        {activeTab === 'agents' ? agentsContent : toolsContent}
      </div>

      {/* Footer */}
      {footer && (
        <div className="px-md py-md border-t border-primary/10 bg-surface-variant/30">
          {footer}
        </div>
      )}
    </div>
  );
};

export default LeftSidebar;
