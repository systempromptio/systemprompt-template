import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { Menu, X } from 'lucide-react';
import { theme } from '@/theme.config';

const navLinks = [
  { path: '/', label: 'Home' },
  { path: '/about', label: 'About' },
  { path: '/features', label: 'Features' },
  { path: '/docs', label: 'Docs' },
  { path: '/pricing', label: 'Pricing' },
];

export const SiteNav: React.FC = () => {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const location = useLocation();

  const isActivePath = (path: string) => {
    if (path === '/') return location.pathname === '/';
    return location.pathname.startsWith(path);
  };

  return (
    <nav className="relative z-navigation">
      <div className="max-w-content mx-auto px-md lg:px-xl">
        <div className="flex items-center justify-between h-16">
          <Link to="/" className="flex items-center gap-sm">
            <picture>
              <source srcSet="/assets/logos/logo.webp" type="image/webp" />
              <source srcSet="/assets/logos/logo.svg" type="image/svg+xml" />
              <img src="/assets/logos/logo.svg" alt={theme.branding.name} className="h-8" />
            </picture>
          </Link>

          <div className="hidden md:flex items-center gap-lg">
            {navLinks.map((link) => (
              <Link
                key={link.path}
                to={link.path}
                className={`text-sm font-medium transition-colors duration-fast ${
                  isActivePath(link.path)
                    ? 'text-primary'
                    : 'text-text-secondary hover:text-text-primary'
                }`}
              >
                {link.label}
              </Link>
            ))}
          </div>

          <button
            className="md:hidden p-sm text-text-primary"
            onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            aria-label="Toggle menu"
          >
            {mobileMenuOpen ? <X size={24} /> : <Menu size={24} />}
          </button>
        </div>
      </div>

      {mobileMenuOpen && (
        <div className="md:hidden absolute top-full left-0 right-0 bg-surface border-b border-primary/10 shadow-lg animate-slideInUp">
          <div className="px-md py-md space-y-xs">
            {navLinks.map((link) => (
              <Link
                key={link.path}
                to={link.path}
                onClick={() => setMobileMenuOpen(false)}
                className={`block px-md py-sm rounded-md text-sm font-medium transition-colors duration-fast ${
                  isActivePath(link.path)
                    ? 'bg-primary/10 text-primary'
                    : 'text-text-secondary hover:bg-surface-variant hover:text-text-primary'
                }`}
              >
                {link.label}
              </Link>
            ))}
          </div>
        </div>
      )}
    </nav>
  );
};
