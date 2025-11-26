import React from 'react';
import ReactMarkdown, { type Components } from 'react-markdown';
import remarkGfm from 'remark-gfm';

interface MarkdownContentProps {
  content: string;
}

export const MarkdownContent: React.FC<MarkdownContentProps> = ({ content }) => {
  const components: Partial<Components> = {
          h1: ({ children }) => (
            <h1 className="text-xxl font-heading font-bold text-text-primary mb-lg mt-xl">
              {children}
            </h1>
          ),
          h2: ({ children }) => (
            <h2 className="text-xl font-heading font-semibold text-text-primary mb-md mt-lg border-b border-primary/10 pb-sm">
              {children}
            </h2>
          ),
          h3: ({ children }) => (
            <h3 className="text-lg font-heading font-semibold text-text-primary mb-sm mt-md">
              {children}
            </h3>
          ),
          h4: ({ children }) => (
            <h4 className="text-md font-heading font-medium text-text-primary mb-sm mt-md">
              {children}
            </h4>
          ),
          p: ({ children }) => (
            <p className="text-md text-text-secondary leading-relaxed mb-md">
              {children}
            </p>
          ),
          a: ({ href, children }) => (
            <a
              href={href}
              className="text-primary hover:underline transition-all duration-fast"
              target={href?.startsWith('http') ? '_blank' : undefined}
              rel={href?.startsWith('http') ? 'noopener noreferrer' : undefined}
            >
              {children}
            </a>
          ),
          ul: ({ children }) => (
            <ul className="list-disc list-inside mb-md space-y-xs text-text-secondary">
              {children}
            </ul>
          ),
          ol: ({ children }) => (
            <ol className="list-decimal list-inside mb-md space-y-xs text-text-secondary">
              {children}
            </ol>
          ),
          li: ({ children }) => (
            <li className="text-md text-text-secondary leading-relaxed ml-md">
              {children}
            </li>
          ),
          blockquote: ({ children }) => (
            <blockquote className="border-l-4 border-primary pl-md py-sm my-md bg-surface-variant/50 rounded-r-md">
              <div className="text-text-secondary italic">{children}</div>
            </blockquote>
          ),
          code: (props) => {
            const { inline, className, children } = props as { inline?: boolean; className?: string; children?: React.ReactNode };
            const match = /language-(\w+)/.exec(className || '');
            const language = match ? match[1] : '';

            // Treat as block code only if explicitly marked as not inline OR has a language class
            const isBlockCode = inline === false || (className && className.includes('language-'));

            if (isBlockCode) {
              return (
                <div className="my-md rounded-md overflow-hidden border border-primary/10">
                  <pre className="p-md bg-surface-dark overflow-x-auto">
                    <code className="text-sm font-mono text-text-primary">
                      {String(children).replace(/\n$/, '')}
                    </code>
                  </pre>
                  {language && (
                    <div className="px-sm py-xs bg-surface-variant text-xs text-text-secondary border-t border-primary/10">
                      {language}
                    </div>
                  )}
                </div>
              );
            }

            return (
              <code className="px-xs py-0.5 bg-surface-dark text-primary text-sm rounded-sm font-mono border border-primary/10">
                {children}
              </code>
            );
          },
          pre: ({ children }) => {
            // Pass through - code renderer handles block code styling
            return <>{children}</>;
          },
          table: ({ children }) => (
            <div className="my-md overflow-x-auto">
              <table className="min-w-full border border-primary/10 rounded-md">
                {children}
              </table>
            </div>
          ),
          thead: ({ children }) => (
            <thead className="bg-surface-variant">{children}</thead>
          ),
          tbody: ({ children }) => (
            <tbody className="divide-y divide-primary/10">{children}</tbody>
          ),
          tr: ({ children }) => <tr>{children}</tr>,
          th: ({ children }) => (
            <th className="px-md py-sm text-left text-sm font-semibold text-text-primary border-b border-primary/10">
              {children}
            </th>
          ),
          td: ({ children }) => (
            <td className="px-md py-sm text-sm text-text-secondary">
              {children}
            </td>
          ),
          hr: () => <hr className="my-lg border-t border-primary/10" />,
          img: ({ src, alt }) => (
            <img
              src={src}
              alt={alt}
              className="my-md rounded-md max-w-full h-auto border border-primary/10"
            />
          ),
  };

  return (
    <div className="prose prose-invert max-w-none">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={components}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};
