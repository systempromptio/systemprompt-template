import type { MarkdownFrontmatter } from '@/types/markdown';

export interface SEOMetadata {
  title: string;
  description: string;
  keywords?: string;
  author?: string;
  image?: string;
  url?: string;
  type?: string;
  publishedTime?: string;
}

export function generateSEOMetadata(
  frontmatter: MarkdownFrontmatter,
  pathname: string
): SEOMetadata {
  const baseUrl = import.meta.env.VITE_BASE_URL || 'https://systemprompt.dev';
  const url = `${baseUrl}${pathname}`;
  const defaultImage = `${baseUrl}/og-default.png`;

  return {
    title: frontmatter.title,
    description: frontmatter.description,
    keywords: frontmatter.keywords,
    author: frontmatter.author || 'SystemPrompt Team',
    image: frontmatter.image
      ? frontmatter.image.startsWith('http')
        ? frontmatter.image
        : `${baseUrl}${frontmatter.image}`
      : defaultImage,
    url,
    type: 'article',
    publishedTime: frontmatter.date,
  };
}

export function generateStructuredData(metadata: SEOMetadata) {
  return {
    '@context': 'https://schema.org',
    '@type': 'Article',
    headline: metadata.title,
    description: metadata.description,
    image: metadata.image,
    author: {
      '@type': 'Organization',
      name: metadata.author,
    },
    publisher: {
      '@type': 'Organization',
      name: 'SystemPrompt',
      logo: {
        '@type': 'ImageObject',
        url: `${import.meta.env.VITE_BASE_URL || 'https://systemprompt.dev'}/logo.png`,
      },
    },
    datePublished: metadata.publishedTime,
    url: metadata.url,
  };
}
