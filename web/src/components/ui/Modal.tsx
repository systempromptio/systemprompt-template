import React, { useRef, useId } from 'react'
import { createPortal } from 'react-dom'
import { X } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import { Button } from './Button'
import { ModalHeader } from './ModalHeader'
import { useModalState } from './hooks/useModalState'
import { useFocusTrap } from '@/lib/accessibility'
import { ErrorBoundary } from '@/components/ErrorBoundary'

export type ModalVariant = 'default' | 'accent' | 'error' | 'success';
export type ModalSize = 'sm' | 'md' | 'lg' | 'xl';

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  children: React.ReactNode;
  variant?: ModalVariant;
  size?: ModalSize;
  showCloseButton?: boolean;
  closeOnBackdrop?: boolean;
  closeOnEscape?: boolean;
  className?: string;
}


interface ModalBodyProps {
  children: React.ReactNode;
  className?: string;
}

interface ModalFooterProps {
  children: React.ReactNode;
  className?: string;
}

const variantStyles: Record<ModalVariant, { border: string; backdrop: string }> = {
  default: {
    border: 'border-primary/20',
    backdrop: 'bg-black/50',
  },
  accent: {
    border: 'border-primary',
    backdrop: 'bg-black/60',
  },
  error: {
    border: 'border-error',
    backdrop: 'bg-black/60',
  },
  success: {
    border: 'border-success',
    backdrop: 'bg-black/60',
  },
};

const sizeStyles: Record<ModalSize, string> = {
  sm: 'modal-sm',
  md: 'modal-md',
  lg: 'modal-lg',
  xl: 'modal-xl',
};

export const Modal: React.FC<ModalProps> = ({
  isOpen,
  onClose,
  title,
  children,
  variant = 'default',
  size = 'md',
  showCloseButton = true,
  closeOnBackdrop = true,
  closeOnEscape = true,
  className,
}) => {
  const { isMobile } = useModalState(isOpen, closeOnEscape, onClose)
  const modalRef = useRef<HTMLDivElement>(null)
  const titleId = useId()

  useFocusTrap(modalRef as React.RefObject<HTMLElement | null>, {
    enabled: isOpen,
    returnFocus: true,
  })

  if (!isOpen) return null

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (closeOnBackdrop && e.target === e.currentTarget) {
      onClose()
    }
  }

  const modalContent = (
    <div
      className={cn(
        'fixed inset-0 z-modal',
        isMobile ? 'flex flex-col' : 'flex items-center justify-center p-4',
        variantStyles[variant].backdrop
      )}
      onClick={handleBackdropClick}
      role="presentation"
    >
      <div
        ref={modalRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={title ? titleId : undefined}
        aria-label={!title ? 'Modal dialog' : undefined}
        className={cn(
          'relative',
          'bg-surface dark:bg-surface-dark',
          isMobile ? 'w-full h-full flex flex-col' : 'w-full flex flex-col',
          !isMobile && size === 'xl' ? 'max-h-[calc(100vh-2rem)]' : !isMobile && 'max-h-[calc(100vh-4rem)]',
          !isMobile && 'border-2',
          variantStyles[variant].border,
          !isMobile && 'rounded-lg shadow-lg',
          isMobile ? 'animate-slideUp' : 'animate-scaleIn',
          !isMobile && sizeStyles[size],
          className
        )}
        style={
          isMobile
            ? {
                paddingBottom: 'env(safe-area-inset-bottom)',
              }
            : undefined
        }
        onClick={(e) => e.stopPropagation()}
      >
        <ErrorBoundary fallbackVariant="compact" retryable={false}>
          <ModalHeader
            title={title}
            titleId={title ? titleId : undefined}
            onClose={onClose}
            showCloseButton={showCloseButton}
            isMobile={isMobile}
          />
          {!title && showCloseButton && (
            <div className={cn('absolute z-10', isMobile ? 'top-sm right-sm' : 'top-md right-md')}>
              <Button
                variant="ghost"
                size="sm"
                icon={X}
                onClick={onClose}
                aria-label="Close modal"
              />
            </div>
          )}
          <div className="flex-1 overflow-y-auto">{children}</div>
        </ErrorBoundary>
      </div>
    </div>
  )

  return createPortal(modalContent, document.body)
}

export { ModalHeader }

export const ModalBody: React.FC<ModalBodyProps> = ({ children, className }) => {
  return (
    <div className={cn('p-md', className)}>
      {children}
    </div>
  );
};

export const ModalFooter: React.FC<ModalFooterProps> = ({ children, className }) => {
  return (
    <div className={cn('p-md border-t border-primary/10 bg-surface-variant/50 flex items-center justify-end gap-sm', className)}>
      {children}
    </div>
  );
};

export default Modal;
