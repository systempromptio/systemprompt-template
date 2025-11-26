import React from 'react';

interface CardGridProps {
  children: React.ReactNode;
  columns?: 1 | 2 | 3 | 4;
  gap?: 'sm' | 'md' | 'lg';
  className?: string;
}

/**
 * Grid layout utility for cards
 */
export const CardGrid: React.FC<CardGridProps> = ({
  children,
  columns = 1,
  gap = 'md',
  className = '',
}) => {
  const getGapValue = () => {
    switch (gap) {
      case 'sm':
        return '8px';
      case 'md':
        return '16px';
      case 'lg':
        return '24px';
    }
  };

  return (
    <div
      className={`grid ${className}`}
      style={{
        gridTemplateColumns: `repeat(${columns}, 1fr)`,
        gap: getGapValue(),
      }}
    >
      {children}
    </div>
  );
};

export default CardGrid;
