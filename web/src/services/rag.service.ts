import { getApiUrl } from '@/utils/env';
import { useAuthStore } from '@/stores/auth.store';

export interface Category {
  id: string;
  name: string;
  slug: string;
  description?: string;
}

export interface Tag {
  id: string;
  name: string;
  slug: string;
}

/**
 * RAG Service for categories and tags
 *
 * Note: Blog posts are now loaded statically at build time.
 * See @/content/blog for static blog post loading.
 */
export class RagService {
  private static async fetchJson<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const authHeader = useAuthStore.getState().getAuthHeader();
    if (!authHeader) {
      throw new Error('Missing authentication');
    }

    const url = getApiUrl(endpoint);
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': authHeader,
        ...options.headers,
      },
    });

    if (!response.ok) {
      throw new Error(`API error: ${response.statusText}`);
    }

    return response.json();
  }

  static async listCategories(): Promise<Category[]> {
    return this.fetchJson<Category[]>('/api/v1/rag/categories');
  }

  static async listTags(): Promise<Tag[]> {
    return this.fetchJson<Tag[]>('/api/v1/rag/tags');
  }
}
