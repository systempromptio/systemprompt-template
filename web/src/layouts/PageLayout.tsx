import React from 'react';
import { GradientBackground } from '@/components/layout/GradientBackground';
import { SiteNav } from '@/components/navigation/SiteNav';
import { Footer } from '@/components/navigation/Footer';

interface PageLayoutProps {
  children: React.ReactNode;
  maxWidth?: 'content' | 'wide' | 'full';
}

export const PageLayout: React.FC<PageLayoutProps> = ({ children, maxWidth = 'content' }) => {
  const maxWidthClass = {
    content: 'max-w-4xl',
    wide: 'max-w-6xl',
    full: 'max-w-content',
  }[maxWidth];

  return (
    <GradientBackground>
      <div className="flex flex-col min-h-screen">
        <SiteNav />

        <main className={`flex-1 ${maxWidthClass} w-full mx-auto px-md lg:px-xl py-xl`}>
          {children}
        </main>

        <Footer />
      </div>
    </GradientBackground>
  );
};
