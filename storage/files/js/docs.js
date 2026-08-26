import { initMobileToc, initTocHighlight } from './site/docs-toc.js';
import { initCollapsibleNav, initNavActiveState, initSmoothScroll } from './site/docs-nav.js';
import { initPagination } from './site/docs-pagination.js';
import { initExportMarkdown } from './site/docs-export.js';
import { initEvidenceGallery } from './site/docs-evidence-gallery.js';

export function initDocs() {
  initTocHighlight();
  initNavActiveState();
  initSmoothScroll();
  initCollapsibleNav();
  initMobileToc();
  initPagination();
  initExportMarkdown();
  initEvidenceGallery();
}

initDocs();
