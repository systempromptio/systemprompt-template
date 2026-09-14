import '../components/sp-evidence-lightbox.js';

const EVIDENCE_SELECTOR = '.docs-content img[src^="/files/images/evidence/"]';

export function initEvidenceGallery() {
  const images = [...document.querySelectorAll(EVIDENCE_SELECTOR)];
  if (images.length > 0) {
    const items = images.map((img) => ({ src: img.src, alt: img.alt }));
    const lightbox = document.createElement('sp-evidence-lightbox');
    lightbox.setAttribute('data-evidence-lightbox', '');
    document.querySelector('.docs-content').append(lightbox);
    lightbox.setItems(items);
    groupIntoGalleries(images);
    for (const [index, img] of images.entries()) {
      const button = img.closest('[data-evidence-item]');
      button.addEventListener('click', () => lightbox.openAt(index));
    }
  }
}

function groupIntoGalleries(images) {
  for (const img of images) {
    const paragraph = img.closest('p');
    const host = paragraph ?? img;
    const previous = host.previousElementSibling;
    const gallery = previous?.matches('[data-evidence-gallery]')
      ? previous
      : createGallery(host);
    const item = document.createElement('button');
    item.type = 'button';
    item.className = 'sp-evidence-gallery__item';
    item.setAttribute('data-evidence-item', '');
    item.setAttribute('aria-label', `Open screenshot: ${img.alt}`);
    gallery.append(item);
    item.append(img);
    if (paragraph !== null && paragraph.childElementCount === 0) {
      paragraph.remove();
    }
  }
}

function createGallery(beforeElement) {
  const gallery = document.createElement('div');
  gallery.className = 'sp-evidence-gallery';
  gallery.setAttribute('data-evidence-gallery', '');
  beforeElement.before(gallery);
  return gallery;
}
