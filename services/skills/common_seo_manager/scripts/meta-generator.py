#!/usr/bin/env python3
"""
Meta Tags Generator
Genera meta tags optimizados para SEO.

Uso:
    python meta-generator.py --keyword "desarrollo web valencia" --type service
    python meta-generator.py --keyword "consultoria odoo" --type product --brand "Indaws"
"""

import argparse
import json
import re
from dataclasses import dataclass
from typing import Optional


@dataclass
class MetaTags:
    """Conjunto de meta tags generados."""
    keyword: str
    page_type: str
    brand: str
    
    title: str = ""
    description: str = ""
    og_title: str = ""
    og_description: str = ""
    og_type: str = "website"
    twitter_card: str = "summary_large_image"
    schema_type: str = ""
    schema_json: str = ""


class MetaGenerator:
    """Generador de meta tags optimizados."""
    
    # Templates por tipo de pagina
    TITLE_TEMPLATES = {
        'service': [
            "{keyword} Profesional | {brand}",
            "{keyword} - Expertos en {area} | {brand}",
            "Servicios de {keyword} | {brand}",
        ],
        'product': [
            "{keyword} - Soluciones Empresariales | {brand}",
            "{keyword} para Empresas | {brand}",
            "Comprar {keyword} | {brand}",
        ],
        'blog': [
            "{keyword}: Guia Completa 2024 | {brand}",
            "Como {keyword} - Tutorial Paso a Paso | {brand}",
            "{keyword}: Todo lo que Necesitas Saber | {brand}",
        ],
        'landing': [
            "{keyword} - Solicita Presupuesto | {brand}",
            "{keyword} sin Compromiso | {brand}",
            "Descubre {keyword} | {brand}",
        ],
        'category': [
            "{keyword} - Catalogo Completo | {brand}",
            "Todos los {keyword} | {brand}",
            "{keyword} - Encuentra tu Solucion | {brand}",
        ]
    }
    
    DESCRIPTION_TEMPLATES = {
        'service': [
            "Descubre nuestros servicios de {keyword}. {brand} ofrece soluciones profesionales adaptadas a tu negocio. Solicita informacion sin compromiso.",
            "Expertos en {keyword} con mas de 10 anos de experiencia. En {brand} te acompanamos en cada proyecto. Contacta con nosotros.",
        ],
        'product': [
            "Conoce {keyword} de {brand}. Soluciones empresariales que optimizan tu negocio. Descubre todas las funcionalidades y beneficios.",
            "{keyword}: la solucion que tu empresa necesita. En {brand} te ayudamos a implementarla. Solicita una demo gratuita.",
        ],
        'blog': [
            "Aprende todo sobre {keyword} en esta guia completa. {brand} te explica paso a paso como conseguir resultados. Lee mas.",
            "Guia definitiva de {keyword}. Consejos practicos, ejemplos reales y mejores practicas del sector. Blog de {brand}.",
        ],
        'landing': [
            "Transforma tu negocio con {keyword}. En {brand} te ofrecemos una propuesta personalizada. Rellena el formulario y te contactamos.",
            "{keyword} adaptado a tus necesidades. Descubre como {brand} puede ayudarte a crecer. Presupuesto sin compromiso.",
        ],
        'category': [
            "Explora nuestra seleccion de {keyword}. En {brand} encontraras la solucion perfecta para tu empresa. Ver catalogo completo.",
            "Catalogo de {keyword} de {brand}. Compara opciones y encuentra la que mejor se adapta a tu negocio.",
        ]
    }
    
    SCHEMA_TYPES = {
        'service': 'Service',
        'product': 'Product',
        'blog': 'Article',
        'landing': 'WebPage',
        'category': 'CollectionPage'
    }
    
    def __init__(self, keyword: str, page_type: str, brand: str = "Indaws"):
        self.keyword = keyword.strip()
        self.page_type = page_type.lower()
        self.brand = brand
        self.area = self._extract_area()
        
    def _extract_area(self) -> str:
        """Extrae el area/sector de la keyword."""
        words = self.keyword.split()
        if len(words) > 1:
            return words[-1].title()
        return "Tecnologia"
    
    def _capitalize_keyword(self) -> str:
        """Capitaliza la keyword correctamente."""
        return ' '.join(word.capitalize() for word in self.keyword.split())
    
    def generate(self) -> MetaTags:
        """Genera el conjunto completo de meta tags."""
        meta = MetaTags(
            keyword=self.keyword,
            page_type=self.page_type,
            brand=self.brand
        )
        
        # Generate title (50-60 chars)
        meta.title = self._generate_title()
        
        # Generate description (150-160 chars)
        meta.description = self._generate_description()
        
        # Open Graph
        meta.og_title = meta.title
        meta.og_description = meta.description
        meta.og_type = 'article' if self.page_type == 'blog' else 'website'
        
        # Schema
        meta.schema_type = self.SCHEMA_TYPES.get(self.page_type, 'WebPage')
        meta.schema_json = self._generate_schema(meta)
        
        return meta
    
    def _generate_title(self) -> str:
        """Genera title tag optimizado."""
        templates = self.TITLE_TEMPLATES.get(self.page_type, self.TITLE_TEMPLATES['service'])
        
        for template in templates:
            title = template.format(
                keyword=self._capitalize_keyword(),
                brand=self.brand,
                area=self.area
            )
            if 50 <= len(title) <= 60:
                return title
        
        # Fallback: truncar o ajustar
        title = templates[0].format(
            keyword=self._capitalize_keyword(),
            brand=self.brand,
            area=self.area
        )
        
        if len(title) > 60:
            title = title[:57] + "..."
        
        return title
    
    def _generate_description(self) -> str:
        """Genera meta description optimizada."""
        templates = self.DESCRIPTION_TEMPLATES.get(self.page_type, self.DESCRIPTION_TEMPLATES['service'])
        
        for template in templates:
            desc = template.format(
                keyword=self.keyword,
                brand=self.brand
            )
            if 150 <= len(desc) <= 160:
                return desc
        
        # Ajustar longitud
        desc = templates[0].format(
            keyword=self.keyword,
            brand=self.brand
        )
        
        if len(desc) > 160:
            desc = desc[:157] + "..."
        elif len(desc) < 150:
            desc += " Descubre mas."
        
        return desc
    
    def _generate_schema(self, meta: MetaTags) -> str:
        """Genera Schema.org JSON-LD."""
        schema = {
            "@context": "https://schema.org",
            "@type": meta.schema_type,
            "name": meta.title,
            "description": meta.description,
            "provider": {
                "@type": "Organization",
                "name": self.brand,
                "url": f"https://www.{self.brand.lower()}.es"
            }
        }
        
        if self.page_type == 'service':
            schema["serviceType"] = self._capitalize_keyword()
            
        elif self.page_type == 'product':
            schema["category"] = self._capitalize_keyword()
            
        elif self.page_type == 'blog':
            schema["@type"] = "Article"
            schema["author"] = {
                "@type": "Organization",
                "name": self.brand
            }
            schema["publisher"] = schema["provider"]
        
        return json.dumps(schema, indent=2, ensure_ascii=False)


def format_output(meta: MetaTags, format_type: str = 'html') -> str:
    """Formatea la salida."""
    if format_type == 'json':
        return json.dumps({
            'title': meta.title,
            'title_length': len(meta.title),
            'description': meta.description,
            'description_length': len(meta.description),
            'og_title': meta.og_title,
            'og_description': meta.og_description,
            'og_type': meta.og_type,
            'twitter_card': meta.twitter_card,
            'schema': json.loads(meta.schema_json)
        }, indent=2, ensure_ascii=False)
    
    elif format_type == 'html':
        return f'''<!-- SEO Meta Tags generados por Indaws SEO Manager -->
<!-- Keyword: {meta.keyword} | Tipo: {meta.page_type} -->

<!-- Basic Meta Tags -->
<title>{meta.title}</title>
<meta name="description" content="{meta.description}">

<!-- Open Graph / Facebook -->
<meta property="og:type" content="{meta.og_type}">
<meta property="og:title" content="{meta.og_title}">
<meta property="og:description" content="{meta.og_description}">
<meta property="og:image" content="[URL_DE_LA_IMAGEN]">
<meta property="og:url" content="[URL_DE_LA_PAGINA]">

<!-- Twitter -->
<meta name="twitter:card" content="{meta.twitter_card}">
<meta name="twitter:title" content="{meta.og_title}">
<meta name="twitter:description" content="{meta.og_description}">
<meta name="twitter:image" content="[URL_DE_LA_IMAGEN]">

<!-- Schema.org JSON-LD -->
<script type="application/ld+json">
{meta.schema_json}
</script>

<!-- Stats -->
<!-- Title: {len(meta.title)} caracteres (objetivo: 50-60) -->
<!-- Description: {len(meta.description)} caracteres (objetivo: 150-160) -->
'''
    
    else:  # text
        return f'''
{'='*60}
  META TAGS GENERADOS
{'='*60}

  Keyword: {meta.keyword}
  Tipo: {meta.page_type}
  Brand: {meta.brand}

  TITLE ({len(meta.title)} chars)
  {'-'*40}
  {meta.title}

  DESCRIPTION ({len(meta.description)} chars)
  {'-'*40}
  {meta.description}

  OPEN GRAPH
  {'-'*40}
  og:type = {meta.og_type}
  og:title = {meta.og_title}
  og:description = {meta.og_description}

  TWITTER
  {'-'*40}
  twitter:card = {meta.twitter_card}

  SCHEMA.ORG
  {'-'*40}
{meta.schema_json}

{'='*60}
'''


def main():
    parser = argparse.ArgumentParser(description='Generador de Meta Tags SEO')
    parser.add_argument('--keyword', '-k', required=True, help='Keyword principal')
    parser.add_argument('--type', '-t', required=True, 
                       choices=['service', 'product', 'blog', 'landing', 'category'],
                       help='Tipo de pagina')
    parser.add_argument('--brand', '-b', default='Indaws', help='Nombre de la marca')
    parser.add_argument('--output', '-o', choices=['text', 'json', 'html'],
                       default='html', help='Formato de salida')
    
    args = parser.parse_args()
    
    generator = MetaGenerator(args.keyword, args.type, args.brand)
    meta = generator.generate()
    
    print(format_output(meta, args.output))


if __name__ == '__main__':
    main()
