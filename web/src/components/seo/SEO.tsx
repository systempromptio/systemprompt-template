import React from 'react';
import { Helmet } from 'react-helmet-async';
import type { SEOMetadata } from '@/utils/seo';
import { generateStructuredData } from '@/utils/seo';
import { getPageTitle } from '@/utils/markdown';
import { theme } from '@/theme.config';

interface SEOProps {
  metadata: SEOMetadata;
}

export const SEO: React.FC<SEOProps> = ({ metadata }) => {
  const structuredData = generateStructuredData(metadata);
  const fullTitle = getPageTitle(metadata.title);

  return (
    <Helmet>
      <title>{fullTitle}</title>
      <meta name="description" content={metadata.description} />
      {metadata.keywords && <meta name="keywords" content={metadata.keywords} />}
      {metadata.author && <meta name="author" content={metadata.author} />}

      <meta property="og:type" content={metadata.type || 'website'} />
      <meta property="og:title" content={metadata.title} />
      <meta property="og:description" content={metadata.description} />
      {metadata.image && <meta property="og:image" content={metadata.image} />}
      {metadata.url && <meta property="og:url" content={metadata.url} />}
      <meta property="og:site_name" content={theme.branding.name} />

      {metadata.publishedTime && (
        <meta property="article:published_time" content={metadata.publishedTime} />
      )}

      <meta name="twitter:card" content="summary_large_image" />
      <meta name="twitter:title" content={metadata.title} />
      <meta name="twitter:description" content={metadata.description} />
      {metadata.image && <meta name="twitter:image" content={metadata.image} />}
      <meta name="twitter:site" content="@systemprompt" />

      {metadata.url && <link rel="canonical" href={metadata.url} />}

      <script type="application/ld+json">
        {JSON.stringify(structuredData)}
      </script>
    </Helmet>
  );
};
