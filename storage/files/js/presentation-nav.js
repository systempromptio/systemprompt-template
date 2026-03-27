(function() {
  const html = document.documentElement;
  if (!html.classList.contains('presentation')) return;

  const slides = document.querySelectorAll('.pres-slide');
  const nav = document.querySelector('.pres-nav');
  const counter = document.getElementById('pres-current');
  const total = document.getElementById('pres-total');
  if (!slides.length || !nav) return;

  if (total) total.textContent = slides.length;

  slides.forEach((slide, i) => {
    const dot = document.createElement('a');
    dot.href = '#' + slide.id;
    if (i === 0) dot.classList.add('active');
    nav.append(dot);
  });

  function showNotesForSlide(index) {
    document.querySelectorAll('.pres-notes').forEach((n) => { n.style.display = 'none'; });
    if (html.classList.contains('notes-visible')) {
      const notes = slides[index].querySelector('.pres-notes');
      if (notes) notes.style.display = 'block';
    }
  }

  const observer = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        const index = Array.from(slides).findIndex((s) => s === entry.target);
        if (counter) counter.textContent = index + 1;
        nav.querySelectorAll('a').forEach((dot, i) => { dot.classList.toggle('active', i === index); });
        entry.target.querySelectorAll('.pres-reveal').forEach((el) => { el.classList.add('visible'); });
        showNotesForSlide(index);
      }
    });
  }, { threshold: 0.5 });

  slides.forEach((slide) => { observer.observe(slide); });

  document.addEventListener('keydown', (e) => {
    const current = Math.round(window.scrollY / window.innerHeight);

    if (e.key === 'n' || e.key === 'N') {
      e.preventDefault();
      html.classList.toggle('notes-visible');
      showNotesForSlide(current);
      return;
    }
    if (e.key === 'ArrowDown' || e.key === 'ArrowRight' || e.key === ' ') {
      e.preventDefault();
      slides[Math.min(current + 1, slides.length - 1)].scrollIntoView({ behavior: 'smooth' });
    }
    if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
      e.preventDefault();
      slides[Math.max(current - 1, 0)].scrollIntoView({ behavior: 'smooth' });
    }
    if (e.key === 'Home') { e.preventDefault(); slides[0].scrollIntoView({ behavior: 'smooth' }); }
    if (e.key === 'End') { e.preventDefault(); slides[slides.length - 1].scrollIntoView({ behavior: 'smooth' }); }
  });
})();
