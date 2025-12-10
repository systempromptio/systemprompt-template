import React from 'react';
import { Link } from 'react-router-dom';
import { Github, Twitter, Mail, Linkedin } from 'lucide-react';
import { theme } from '@/theme.config';

const socialIconMap = {
  github: Github,
  twitter: Twitter,
  email: Mail,
  linkedin: Linkedin,
};

export const Footer: React.FC = () => {
  const currentYear = new Date().getFullYear();
  const footerLinks = theme.navigation?.footer || {};
  const socialLinks = theme.navigation?.social || [];

  return (
    <footer className="border-t border-primary/10 bg-surface/50 backdrop-blur-sm mt-auto">
      <div className="max-w-content mx-auto px-md lg:px-xl py-xl">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-lg mb-lg">
          <div>
            <h3 className="text-sm font-semibold text-text-primary mb-md">Legal</h3>
            <ul className="space-y-sm">
              {footerLinks.legal?.map((link) => (
                <li key={link.path}>
                  <Link
                    to={link.path}
                    className="text-sm text-text-secondary hover:text-primary transition-colors duration-fast relative inline-block after:content-[''] after:absolute after:w-0 after:h-[1px] after:bottom-0 after:left-0 after:bg-primary after:transition-all after:duration-300 hover:after:w-full"
                  >
                    {link.label}
                  </Link>
                </li>
              ))}
            </ul>
          </div>

          <div>
            <h3 className="text-sm font-semibold text-text-primary mb-md">Resources</h3>
            <ul className="space-y-sm">
              {footerLinks.resources?.map((link) => (
                <li key={link.path}>
                  <Link
                    to={link.path}
                    className="text-sm text-text-secondary hover:text-primary transition-colors duration-fast relative inline-block after:content-[''] after:absolute after:w-0 after:h-[1px] after:bottom-0 after:left-0 after:bg-primary after:transition-all after:duration-300 hover:after:w-full"
                  >
                    {link.label}
                  </Link>
                </li>
              ))}
            </ul>
          </div>

          <div>
            <h3 className="text-sm font-semibold text-text-primary mb-md">Connect</h3>
            <div className="flex gap-md">
              {socialLinks.map((social) => {
                const IconComponent = socialIconMap[social.type] || Mail;
                return (
                  <a
                    key={social.label}
                    href={social.href}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-text-secondary hover:text-primary transition-colors duration-fast"
                    aria-label={social.label}
                  >
                    <IconComponent size={20} />
                  </a>
                );
              })}
            </div>
          </div>
        </div>

        <div className="pt-md border-t border-primary/10 text-center text-sm text-text-secondary">
          <p>© {currentYear} {theme.branding.name}. {theme.branding.description}</p>
        </div>
      </div>
    </footer>
  );
};
