import React, { useEffect } from 'react';
import { createPortal } from 'react-dom';
import { X } from 'lucide-react';
import { cn } from '@/lib/utils/cn';

interface MobileDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  side: 'left' | 'right';
  children: React.ReactNode;
  title?: string;
}

export const MobileDrawer: React.FC<MobileDrawerProps> = ({
  isOpen,
  onClose,
  side,
  children,
  title,
}) => {
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }

    return () => {
      document.body.style.overflow = '';
    };
  }, [isOpen]);

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isOpen, onClose]);

  const drawerContent = (
    <div
      className={cn(
        'fixed inset-0 z-modal transition-opacity duration-300',
        isOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'
      )}
    >
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
        role="presentation"
      />

      <div
        className={cn(
          'absolute top-0 bottom-0 bg-surface/95 backdrop-blur-md flex flex-col',
          'shadow-2xl transition-transform duration-300 ease-in-out',
          'w-[80vw] max-w-[320px]',
          side === 'left' ? 'left-0 border-r-2 border-primary/20' : 'right-0 border-l-2 border-primary/20',
          isOpen
            ? 'translate-x-0'
            : side === 'left'
            ? '-translate-x-full'
            : 'translate-x-full'
        )}
        style={{
          paddingTop: 'env(safe-area-inset-top)',
          paddingBottom: 'env(safe-area-inset-bottom)',
        }}
      >
        <div className="flex items-center justify-between px-md py-md">
          <h2 className="text-sm font-heading font-medium uppercase tracking-wide text-text-secondary">
            {title || (side === 'left' ? 'Agents' : 'Tools')}
          </h2>
          <button
            onClick={onClose}
            className={cn(
              'p-xs rounded-lg transition-all duration-fast',
              'hover:bg-primary/10 active:scale-95',
              'min-w-[44px] min-h-[44px] flex items-center justify-center'
            )}
            aria-label="Close drawer"
          >
            <X className="w-5 h-5 text-text-secondary" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto scrollbar-thin">
          {children}
        </div>
      </div>
    </div>
  );

  return createPortal(drawerContent, document.body);
};

export default MobileDrawer;
