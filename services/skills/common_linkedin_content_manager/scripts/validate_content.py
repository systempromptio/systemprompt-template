#!/usr/bin/env python3
"""
LinkedIn Content Manager - Brand Compliance Validator

Validates post content against Indaws brand guidelines.
Checks: emoji usage, vocabulary, tone, formatting.

Usage:
    python validate_content.py --file posts.json
    python validate_content.py --text "Post content to validate"
"""

import json
import re
import argparse
from pathlib import Path
from typing import List, Tuple


# Prohibited vocabulary (cost/problem language)
PROHIBITED_WORDS = [
    r"\bproblema\b", r"\bproblemas\b",
    r"\bcoste\b", r"\bcostes\b", r"\bcosto\b", r"\bcostos\b",
    r"\bgasto\b", r"\bgastos\b",
    r"\bprecio\b", r"\bprecios\b",
    r"\bbarato\b", r"\bbaratos\b", r"\bbarata\b", r"\bbaratas\b",
    r"\bcaro\b", r"\bcaros\b", r"\bcara\b", r"\bcaras\b",
    r"\bfallo\b", r"\bfallos\b", r"\bfalla\b", r"\bfallas\b",
    r"\bvendemos\b", r"\bvender\b", r"\bventa\b",
]

# Value vocabulary alternatives
VALUE_ALTERNATIVES = {
    "problema": "reto, caso de negocio, oportunidad",
    "coste": "inversion, valoracion",
    "gasto": "inversion",
    "precio": "inversion, valoracion",
    "barato": "rentable, optimizado",
    "caro": "valor premium, inversion estrategica",
    "fallo": "reto, area de mejora",
    "vendemos": "te acompanamos en, desarrollamos",
}

# Emoji regex pattern
EMOJI_PATTERN = re.compile(
    "["
    "\U0001F600-\U0001F64F"  # emoticons
    "\U0001F300-\U0001F5FF"  # symbols & pictographs
    "\U0001F680-\U0001F6FF"  # transport & map symbols
    "\U0001F1E0-\U0001F1FF"  # flags
    "\U00002702-\U000027B0"  # dingbats
    "\U0001F900-\U0001F9FF"  # supplemental symbols
    "\U0001FA00-\U0001FA6F"  # chess symbols
    "\U0001FA70-\U0001FAFF"  # symbols extended
    "]+",
    flags=re.UNICODE
)


def check_emojis(text: str) -> List[str]:
    """Check for emoji usage."""
    issues = []
    emojis = EMOJI_PATTERN.findall(text)
    if emojis:
        issues.append(f"Se encontraron emojis: {', '.join(emojis)}. Eliminalos para cumplir con las directrices de marca.")
    return issues


def check_vocabulary(text: str) -> List[str]:
    """Check for prohibited vocabulary."""
    issues = []
    text_lower = text.lower()
    
    for pattern in PROHIBITED_WORDS:
        matches = re.findall(pattern, text_lower)
        if matches:
            word = matches[0]
            alternative = VALUE_ALTERNATIVES.get(word, "vocabulario de valor")
            issues.append(f"Palabra prohibida '{word}' encontrada. Alternativas: {alternative}")
    
    return issues


def check_hook(text: str) -> List[str]:
    """Check for strong opening hook."""
    issues = []
    lines = text.strip().split('\n')
    
    if lines:
        first_line = lines[0].strip()
        if len(first_line) < 20:
            issues.append("La primera linea es muy corta. Un buen hook debe captar atencion inmediatamente.")
        if first_line.startswith("Hola") or first_line.startswith("Buenos"):
            issues.append("Evita saludos genericos como apertura. Empieza con una afirmacion impactante o pregunta.")
    
    return issues


def check_length(text: str) -> List[str]:
    """Check content length."""
    issues = []
    length = len(text)
    
    if length < 100:
        issues.append(f"Contenido muy corto ({length} caracteres). Los posts efectivos suelen tener al menos 200 caracteres.")
    elif length > 3000:
        issues.append(f"Contenido excede el limite ({length}/3000 caracteres). LinkedIn truncara el texto.")
    elif length > 2500:
        issues.append(f"Contenido cerca del limite ({length}/3000 caracteres). Considera recortar para mayor legibilidad.")
    
    return issues


def check_hashtags(hashtags: List[str]) -> List[str]:
    """Check hashtag usage."""
    issues = []
    count = len(hashtags)
    
    if count == 0:
        issues.append("Sin hashtags. Anade 3-5 hashtags relevantes para aumentar el alcance.")
    elif count < 3:
        issues.append(f"Pocos hashtags ({count}). Se recomiendan 3-5 para optimizar la visibilidad.")
    elif count > 5:
        issues.append(f"Demasiados hashtags ({count}). Mas de 5 puede parecer spam. Prioriza los mas relevantes.")
    
    return issues


def check_call_to_action(text: str) -> List[str]:
    """Check for engagement prompts."""
    issues = []
    
    cta_patterns = [
        r"\?$",  # ends with question
        r"comentar?",
        r"que opinas",
        r"tu experiencia",
        r"cuentanos",
        r"escribeme",
        r"hablemos",
    ]
    
    has_cta = any(re.search(p, text.lower()) for p in cta_patterns)
    
    if not has_cta:
        issues.append("No se detecta llamada a la accion. Considera terminar con una pregunta o invitacion al dialogo.")
    
    return issues


def check_formal_tu(text: str) -> List[str]:
    """Check for professional 'tu' vs formal 'usted'."""
    issues = []
    
    usted_patterns = [
        r"\busted\b",
        r"\bsu empresa\b",
        r"\bsus necesidades\b",
        r"\bestimado\b",
        r"\bestimada\b",
    ]
    
    for pattern in usted_patterns:
        if re.search(pattern, text.lower()):
            issues.append("Se detecta tratamiento formal 'usted'. Indaws usa 'tu' profesional.")
            break
    
    return issues


def validate_content(text: str, hashtags: List[str] = None) -> Tuple[bool, List[str]]:
    """Run all validation checks."""
    all_issues = []
    
    all_issues.extend(check_emojis(text))
    all_issues.extend(check_vocabulary(text))
    all_issues.extend(check_hook(text))
    all_issues.extend(check_length(text))
    all_issues.extend(check_call_to_action(text))
    all_issues.extend(check_formal_tu(text))
    
    if hashtags:
        all_issues.extend(check_hashtags(hashtags))
    
    is_valid = len(all_issues) == 0
    return is_valid, all_issues


def validate_post(post: dict) -> Tuple[bool, List[str]]:
    """Validate a post object."""
    content = post.get("content", "")
    hashtags = post.get("hashtags", [])
    return validate_content(content, hashtags)


def main():
    parser = argparse.ArgumentParser(description="Validate LinkedIn content against brand guidelines")
    parser.add_argument("--file", type=str, help="JSON file with posts to validate")
    parser.add_argument("--text", type=str, help="Text content to validate directly")
    parser.add_argument("--quiet", action="store_true", help="Only show issues, not passes")
    
    args = parser.parse_args()
    
    if args.text:
        # Direct text validation
        is_valid, issues = validate_content(args.text)
        
        if is_valid:
            print("VALIDO: El contenido cumple con las directrices de marca.")
        else:
            print("PROBLEMAS DETECTADOS:")
            for issue in issues:
                print(f"  - {issue}")
        
        return 0 if is_valid else 1
    
    elif args.file:
        # File validation
        file_path = Path(args.file)
        if not file_path.exists():
            print(f"Error: File not found: {file_path}")
            return 1
        
        with open(file_path, "r", encoding="utf-8") as f:
            posts = json.load(f)
        
        total = len(posts)
        valid_count = 0
        
        for post in posts:
            is_valid, issues = validate_post(post)
            title = post.get("title", "Sin titulo")
            
            if is_valid:
                valid_count += 1
                if not args.quiet:
                    print(f"OK: {title}")
            else:
                print(f"\nPROBLEMAS: {title}")
                for issue in issues:
                    print(f"  - {issue}")
        
        print(f"\n--- Resumen: {valid_count}/{total} posts validos ---")
        return 0 if valid_count == total else 1
    
    else:
        parser.print_help()
        return 1


if __name__ == "__main__":
    exit(main())
