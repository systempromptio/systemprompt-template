import { cn } from '@/lib/utils/cn'

interface SectionTitleProps {
  children: React.ReactNode
  className?: string
}

export function SectionTitle({ children, className }: SectionTitleProps) {
  return (
    <span className={cn(
      'text-sm font-heading font-medium uppercase tracking-wide text-text-primary',
      className
    )}>
      {children}
    </span>
  )
}
