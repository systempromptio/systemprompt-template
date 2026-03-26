#!/usr/bin/env python3
"""
SEO Analyzer Script
Analiza URLs para detectar problemas de posicionamiento SEO.

Uso:
    python seo-analyzer.py https://ejemplo.com
    python seo-analyzer.py https://ejemplo.com --output json
    python seo-analyzer.py https://ejemplo.com --output html > report.html
"""

import argparse
import json
import sys
import re
from dataclasses import dataclass, field, asdict
from typing import Optional
from urllib.parse import urlparse, urljoin

try:
    import requests
    from bs4 import BeautifulSoup
except ImportError:
    print("Error: Dependencias no instaladas.")
    print("Ejecuta: pip install requests beautifulsoup4 lxml")
    sys.exit(1)


@dataclass
class SEOIssue:
    """Representa un problema SEO detectado."""
    category: str
    title: str
    description: str
    severity: str  # success, warning, error
    value: Optional[str] = None


@dataclass
class SEOReport:
    """Informe completo de auditoria SEO."""
    url: str
    status_code: int
    load_time: float
    score: int = 0
    issues: list = field(default_factory=list)
    
    # Meta tags
    title: str = ""
    title_length: int = 0
    description: str = ""
    description_length: int = 0
    canonical: str = ""
    robots: str = ""
    
    # Open Graph
    og_title: str = ""
    og_description: str = ""
    og_image: str = ""
    
    # Content
    h1_count: int = 0
    h1_text: str = ""
    heading_structure: list = field(default_factory=list)
    word_count: int = 0
    images_total: int = 0
    images_without_alt: int = 0
    
    # Links
    internal_links: int = 0
    external_links: int = 0
    
    # Technical
    is_https: bool = False
    has_viewport: bool = False
    has_lang: bool = False
    
    def to_dict(self):
        return asdict(self)


class SEOAnalyzer:
    """Analizador SEO para URLs."""
    
    HEADERS = {
        'User-Agent': 'Mozilla/5.0 (compatible; IndawsSEOBot/1.0; +https://indaws.es/bot)'
    }
    
    def __init__(self, url: str):
        self.url = url
        self.parsed_url = urlparse(url)
        self.report = None
        self.soup = None
        
    def analyze(self) -> SEOReport:
        """Ejecuta el analisis completo."""
        # Fetch page
        try:
            start_time = __import__('time').time()
            response = requests.get(self.url, headers=self.HEADERS, timeout=30)
            load_time = round(__import__('time').time() - start_time, 2)
        except requests.RequestException as e:
            print(f"Error fetching URL: {e}", file=sys.stderr)
            sys.exit(1)
        
        self.soup = BeautifulSoup(response.text, 'html.parser')
        
        self.report = SEOReport(
            url=self.url,
            status_code=response.status_code,
            load_time=load_time
        )
        
        # Run all checks
        self._check_meta_tags()
        self._check_open_graph()
        self._check_headings()
        self._check_content()
        self._check_images()
        self._check_links()
        self._check_technical()
        
        # Calculate score
        self._calculate_score()
        
        return self.report
    
    def _check_meta_tags(self):
        """Analiza meta tags."""
        # Title
        title_tag = self.soup.find('title')
        if title_tag:
            self.report.title = title_tag.get_text().strip()
            self.report.title_length = len(self.report.title)
            
            if 50 <= self.report.title_length <= 60:
                self.report.issues.append(SEOIssue(
                    category="meta",
                    title="Title Tag",
                    description=f"El titulo tiene una longitud optima de {self.report.title_length} caracteres",
                    severity="success",
                    value=f"{self.report.title_length} chars"
                ))
            elif self.report.title_length < 50:
                self.report.issues.append(SEOIssue(
                    category="meta",
                    title="Title Tag",
                    description="El titulo es demasiado corto, considera expandirlo",
                    severity="warning",
                    value=f"{self.report.title_length} chars"
                ))
            else:
                self.report.issues.append(SEOIssue(
                    category="meta",
                    title="Title Tag",
                    description="El titulo es demasiado largo, puede ser truncado",
                    severity="warning",
                    value=f"{self.report.title_length} chars"
                ))
        else:
            self.report.issues.append(SEOIssue(
                category="meta",
                title="Title Tag",
                description="No se encontro etiqueta title en la pagina",
                severity="error",
                value="Missing"
            ))
        
        # Meta Description
        meta_desc = self.soup.find('meta', attrs={'name': 'description'})
        if meta_desc and meta_desc.get('content'):
            self.report.description = meta_desc['content']
            self.report.description_length = len(self.report.description)
            
            if 150 <= self.report.description_length <= 160:
                self.report.issues.append(SEOIssue(
                    category="meta",
                    title="Meta Description",
                    description=f"La descripcion tiene una longitud optima de {self.report.description_length} caracteres",
                    severity="success",
                    value=f"{self.report.description_length} chars"
                ))
            elif self.report.description_length < 150:
                self.report.issues.append(SEOIssue(
                    category="meta",
                    title="Meta Description",
                    description="La descripcion es corta, considera expandirla",
                    severity="warning",
                    value=f"{self.report.description_length} chars"
                ))
            else:
                self.report.issues.append(SEOIssue(
                    category="meta",
                    title="Meta Description",
                    description="La descripcion es demasiado larga, puede ser truncada",
                    severity="warning",
                    value=f"{self.report.description_length} chars"
                ))
        else:
            self.report.issues.append(SEOIssue(
                category="meta",
                title="Meta Description",
                description="No se encontro meta description",
                severity="error",
                value="Missing"
            ))
        
        # Canonical
        canonical = self.soup.find('link', attrs={'rel': 'canonical'})
        if canonical and canonical.get('href'):
            self.report.canonical = canonical['href']
            self.report.issues.append(SEOIssue(
                category="meta",
                title="Canonical URL",
                description="URL canonica correctamente definida",
                severity="success",
                value="OK"
            ))
        else:
            self.report.issues.append(SEOIssue(
                category="meta",
                title="Canonical URL",
                description="No se encontro URL canonica",
                severity="warning",
                value="Missing"
            ))
        
        # Robots
        robots = self.soup.find('meta', attrs={'name': 'robots'})
        if robots and robots.get('content'):
            self.report.robots = robots['content']
    
    def _check_open_graph(self):
        """Analiza Open Graph tags."""
        og_tags = {
            'og:title': 'og_title',
            'og:description': 'og_description',
            'og:image': 'og_image'
        }
        
        missing = []
        for prop, attr in og_tags.items():
            tag = self.soup.find('meta', attrs={'property': prop})
            if tag and tag.get('content'):
                setattr(self.report, attr, tag['content'])
            else:
                missing.append(prop)
        
        if not missing:
            self.report.issues.append(SEOIssue(
                category="meta",
                title="Open Graph Tags",
                description="Todos los Open Graph tags estan presentes",
                severity="success",
                value="OK"
            ))
        else:
            self.report.issues.append(SEOIssue(
                category="meta",
                title="Open Graph Tags",
                description=f"Faltan tags: {', '.join(missing)}",
                severity="error",
                value=f"{len(missing)} missing"
            ))
    
    def _check_headings(self):
        """Analiza estructura de encabezados."""
        # H1
        h1_tags = self.soup.find_all('h1')
        self.report.h1_count = len(h1_tags)
        
        if self.report.h1_count == 1:
            self.report.h1_text = h1_tags[0].get_text().strip()
            self.report.issues.append(SEOIssue(
                category="content",
                title="Encabezado H1",
                description="La pagina tiene un unico H1",
                severity="success",
                value="1 H1"
            ))
        elif self.report.h1_count == 0:
            self.report.issues.append(SEOIssue(
                category="content",
                title="Encabezado H1",
                description="La pagina no tiene encabezado H1",
                severity="error",
                value="0 H1"
            ))
        else:
            self.report.issues.append(SEOIssue(
                category="content",
                title="Encabezado H1",
                description=f"La pagina tiene {self.report.h1_count} H1, deberia tener solo uno",
                severity="warning",
                value=f"{self.report.h1_count} H1"
            ))
        
        # Heading structure
        headings = self.soup.find_all(['h1', 'h2', 'h3', 'h4', 'h5', 'h6'])
        levels = [int(h.name[1]) for h in headings]
        self.report.heading_structure = levels
        
        # Check for skipped levels
        prev = 0
        skipped = []
        for level in levels:
            if level > prev + 1 and prev > 0:
                skipped.append(f"H{prev} to H{level}")
            prev = level
        
        if skipped:
            self.report.issues.append(SEOIssue(
                category="content",
                title="Estructura de Encabezados",
                description=f"Se saltan niveles: {', '.join(skipped)}",
                severity="warning",
                value=f"Skip {skipped[0].split()[2]}"
            ))
    
    def _check_content(self):
        """Analiza contenido de la pagina."""
        # Word count
        text = self.soup.get_text()
        words = re.findall(r'\b\w+\b', text)
        self.report.word_count = len(words)
        
        if self.report.word_count >= 300:
            self.report.issues.append(SEOIssue(
                category="content",
                title="Longitud del Contenido",
                description="La pagina tiene contenido suficiente",
                severity="success",
                value=f"{self.report.word_count:,} words"
            ))
        else:
            self.report.issues.append(SEOIssue(
                category="content",
                title="Longitud del Contenido",
                description="El contenido es muy corto, considera expandirlo",
                severity="warning",
                value=f"{self.report.word_count} words"
            ))
    
    def _check_images(self):
        """Analiza imagenes."""
        images = self.soup.find_all('img')
        self.report.images_total = len(images)
        
        without_alt = [img for img in images if not img.get('alt')]
        self.report.images_without_alt = len(without_alt)
        
        if self.report.images_without_alt == 0 and self.report.images_total > 0:
            self.report.issues.append(SEOIssue(
                category="content",
                title="Imagenes Alt Text",
                description="Todas las imagenes tienen atributo alt",
                severity="success",
                value=f"{self.report.images_total} imgs"
            ))
        elif self.report.images_without_alt > 0:
            self.report.issues.append(SEOIssue(
                category="content",
                title="Imagenes sin Alt Text",
                description=f"{self.report.images_without_alt} imagenes no tienen atributo alt",
                severity="warning",
                value=f"{self.report.images_without_alt} imgs"
            ))
    
    def _check_links(self):
        """Analiza enlaces."""
        links = self.soup.find_all('a', href=True)
        
        for link in links:
            href = link['href']
            parsed = urlparse(urljoin(self.url, href))
            
            if parsed.netloc == self.parsed_url.netloc or not parsed.netloc:
                self.report.internal_links += 1
            else:
                self.report.external_links += 1
    
    def _check_technical(self):
        """Checks tecnicos."""
        # HTTPS
        self.report.is_https = self.parsed_url.scheme == 'https'
        if self.report.is_https:
            self.report.issues.append(SEOIssue(
                category="technical",
                title="HTTPS",
                description="El sitio usa certificado SSL",
                severity="success",
                value="Secure"
            ))
        else:
            self.report.issues.append(SEOIssue(
                category="technical",
                title="HTTPS",
                description="El sitio no usa HTTPS",
                severity="error",
                value="Insecure"
            ))
        
        # Viewport
        viewport = self.soup.find('meta', attrs={'name': 'viewport'})
        self.report.has_viewport = bool(viewport)
        if self.report.has_viewport:
            self.report.issues.append(SEOIssue(
                category="technical",
                title="Mobile Friendly",
                description="La pagina tiene meta viewport configurado",
                severity="success",
                value="OK"
            ))
        else:
            self.report.issues.append(SEOIssue(
                category="technical",
                title="Mobile Friendly",
                description="Falta meta viewport para dispositivos moviles",
                severity="error",
                value="Missing"
            ))
        
        # Lang attribute
        html_tag = self.soup.find('html')
        self.report.has_lang = bool(html_tag and html_tag.get('lang'))
        
        # Load time
        if self.report.load_time <= 2.0:
            self.report.issues.append(SEOIssue(
                category="technical",
                title="Tiempo de Carga",
                description="La pagina carga rapidamente",
                severity="success",
                value=f"{self.report.load_time}s"
            ))
        elif self.report.load_time <= 4.0:
            self.report.issues.append(SEOIssue(
                category="technical",
                title="Tiempo de Carga",
                description="La pagina tarda mas de 2 segundos en cargar",
                severity="warning",
                value=f"{self.report.load_time}s"
            ))
        else:
            self.report.issues.append(SEOIssue(
                category="technical",
                title="Tiempo de Carga",
                description="La pagina es muy lenta, optimiza el rendimiento",
                severity="error",
                value=f"{self.report.load_time}s"
            ))
    
    def _calculate_score(self):
        """Calcula puntuacion SEO."""
        total_checks = len(self.report.issues)
        if total_checks == 0:
            self.report.score = 0
            return
        
        weights = {'success': 1.0, 'warning': 0.5, 'error': 0.0}
        total_score = sum(weights.get(issue.severity, 0) for issue in self.report.issues)
        
        self.report.score = int((total_score / total_checks) * 100)


def format_text(report: SEOReport) -> str:
    """Formatea el reporte en texto plano."""
    lines = [
        f"\n{'='*60}",
        f"  AUDITORIA SEO: {report.url}",
        f"{'='*60}",
        f"",
        f"  PUNTUACION: {report.score}/100",
        f"  Status: {report.status_code} | Tiempo: {report.load_time}s",
        f"",
    ]
    
    categories = {}
    for issue in report.issues:
        if issue.category not in categories:
            categories[issue.category] = []
        categories[issue.category].append(issue)
    
    category_names = {
        'meta': 'META TAGS',
        'content': 'CONTENIDO',
        'technical': 'TECNICO'
    }
    
    severity_icons = {
        'success': '[OK]',
        'warning': '[!!]',
        'error': '[XX]'
    }
    
    for cat, issues in categories.items():
        lines.append(f"\n  {category_names.get(cat, cat.upper())}")
        lines.append(f"  {'-'*40}")
        for issue in issues:
            icon = severity_icons.get(issue.severity, '[??]')
            value = f" ({issue.value})" if issue.value else ""
            lines.append(f"  {icon} {issue.title}{value}")
            lines.append(f"      {issue.description}")
    
    lines.append(f"\n{'='*60}\n")
    
    return '\n'.join(lines)


def format_json(report: SEOReport) -> str:
    """Formatea el reporte en JSON."""
    data = report.to_dict()
    # Convert issues to dicts
    data['issues'] = [asdict(i) for i in report.issues]
    return json.dumps(data, indent=2, ensure_ascii=False)


def main():
    parser = argparse.ArgumentParser(description='Analizador SEO de URLs')
    parser.add_argument('url', help='URL a analizar')
    parser.add_argument('--output', '-o', choices=['text', 'json', 'html'],
                       default='text', help='Formato de salida')
    
    args = parser.parse_args()
    
    # Validate URL
    if not args.url.startswith(('http://', 'https://')):
        args.url = 'https://' + args.url
    
    analyzer = SEOAnalyzer(args.url)
    report = analyzer.analyze()
    
    if args.output == 'json':
        print(format_json(report))
    else:
        print(format_text(report))


if __name__ == '__main__':
    main()
