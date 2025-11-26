export interface MarkdownFrontmatter {
  title: string;
  description: string;
  keywords: string;
  author: string;
  date: string;
  image: string;
  slug: string;
}

export interface MarkdownContent {
  frontmatter: MarkdownFrontmatter;
  content: string;
  slug: string;
}
