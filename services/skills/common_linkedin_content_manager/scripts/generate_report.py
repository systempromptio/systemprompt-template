#!/usr/bin/env python3
"""
LinkedIn Content Manager - Report Generator

Generates weekly or monthly performance reports from posts data.
Output: Markdown report suitable for review or sharing.

Usage:
    python generate_report.py --period weekly
    python generate_report.py --period monthly --output report.md
"""

import json
import argparse
from datetime import datetime, timedelta
from pathlib import Path


def load_posts(data_path: Path) -> list:
    """Load posts from JSON file."""
    posts_file = data_path / "posts.json"
    if not posts_file.exists():
        return []
    
    with open(posts_file, "r", encoding="utf-8") as f:
        return json.load(f)


def filter_posts_by_period(posts: list, period: str) -> list:
    """Filter posts by time period (weekly or monthly)."""
    now = datetime.now()
    
    if period == "weekly":
        cutoff = now - timedelta(days=7)
    elif period == "monthly":
        cutoff = now - timedelta(days=30)
    else:
        cutoff = now - timedelta(days=90)
    
    filtered = []
    for post in posts:
        date_str = post.get("published_date") or post.get("scheduled_date")
        if date_str:
            try:
                post_date = datetime.fromisoformat(date_str.replace("Z", "+00:00"))
                if post_date.replace(tzinfo=None) >= cutoff:
                    filtered.append(post)
            except ValueError:
                continue
    
    return filtered


def calculate_metrics(posts: list) -> dict:
    """Calculate aggregate metrics from posts."""
    total_impressions = 0
    total_reactions = 0
    total_comments = 0
    total_shares = 0
    total_clicks = 0
    published_count = 0
    
    for post in posts:
        if post.get("status") == "published":
            published_count += 1
            metrics = post.get("metrics", {})
            total_impressions += metrics.get("impressions", 0)
            total_reactions += metrics.get("reactions", 0)
            total_comments += metrics.get("comments", 0)
            total_shares += metrics.get("shares", 0)
            total_clicks += metrics.get("clicks", 0)
    
    engagement = total_reactions + total_comments + total_shares
    engagement_rate = (engagement / total_impressions * 100) if total_impressions > 0 else 0
    
    return {
        "total_posts": len(posts),
        "published_count": published_count,
        "draft_count": len([p for p in posts if p.get("status") == "draft"]),
        "scheduled_count": len([p for p in posts if p.get("status") == "scheduled"]),
        "total_impressions": total_impressions,
        "total_reactions": total_reactions,
        "total_comments": total_comments,
        "total_shares": total_shares,
        "total_clicks": total_clicks,
        "engagement_rate": round(engagement_rate, 2),
    }


def get_top_posts(posts: list, limit: int = 5) -> list:
    """Get top performing posts by engagement."""
    published = [p for p in posts if p.get("status") == "published"]
    
    def engagement_score(post):
        m = post.get("metrics", {})
        return m.get("reactions", 0) + m.get("comments", 0) * 2 + m.get("shares", 0) * 3
    
    return sorted(published, key=engagement_score, reverse=True)[:limit]


def get_hashtag_performance(posts: list) -> dict:
    """Analyze hashtag usage and performance."""
    hashtag_stats = {}
    
    for post in posts:
        if post.get("status") != "published":
            continue
        
        metrics = post.get("metrics", {})
        engagement = metrics.get("reactions", 0) + metrics.get("comments", 0) + metrics.get("shares", 0)
        
        for tag in post.get("hashtags", []):
            tag = tag.lower()
            if tag not in hashtag_stats:
                hashtag_stats[tag] = {"count": 0, "total_engagement": 0}
            hashtag_stats[tag]["count"] += 1
            hashtag_stats[tag]["total_engagement"] += engagement
    
    # Calculate average engagement per hashtag
    for tag, stats in hashtag_stats.items():
        stats["avg_engagement"] = round(stats["total_engagement"] / stats["count"], 1)
    
    return dict(sorted(hashtag_stats.items(), key=lambda x: x[1]["avg_engagement"], reverse=True))


def generate_report(posts: list, period: str) -> str:
    """Generate markdown report."""
    metrics = calculate_metrics(posts)
    top_posts = get_top_posts(posts)
    hashtags = get_hashtag_performance(posts)
    
    period_label = "Semanal" if period == "weekly" else "Mensual"
    date_str = datetime.now().strftime("%d/%m/%Y")
    
    report = f"""# Informe de Rendimiento LinkedIn - {period_label}

**Fecha de generacion:** {date_str}

---

## Resumen Ejecutivo

| Metrica | Valor |
|---------|-------|
| Publicaciones totales | {metrics['total_posts']} |
| Publicadas | {metrics['published_count']} |
| Programadas | {metrics['scheduled_count']} |
| Borradores | {metrics['draft_count']} |

---

## Metricas de Rendimiento

| KPI | Valor |
|-----|-------|
| Impresiones totales | {metrics['total_impressions']:,} |
| Reacciones | {metrics['total_reactions']:,} |
| Comentarios | {metrics['total_comments']:,} |
| Compartidos | {metrics['total_shares']:,} |
| Clics | {metrics['total_clicks']:,} |
| **Tasa de engagement** | **{metrics['engagement_rate']}%** |

---

## Top 5 Publicaciones

"""
    
    for i, post in enumerate(top_posts, 1):
        m = post.get("metrics", {})
        report += f"""### {i}. {post['title']}

- Impresiones: {m.get('impressions', 0):,}
- Reacciones: {m.get('reactions', 0):,}
- Comentarios: {m.get('comments', 0):,}
- Compartidos: {m.get('shares', 0):,}

"""

    report += """---

## Rendimiento de Hashtags

| Hashtag | Usos | Engagement promedio |
|---------|------|---------------------|
"""
    
    for tag, stats in list(hashtags.items())[:10]:
        report += f"| {tag} | {stats['count']} | {stats['avg_engagement']} |\n"
    
    report += """
---

## Recomendaciones

1. **Contenido con mejor rendimiento:** Los articulos explicativos y casos de exito generan mayor engagement.

2. **Horarios optimos:** Martes y miercoles entre 10:00-12:00 muestran mejores resultados.

3. **Hashtags recomendados:** Priorizar hashtags con mayor engagement promedio en futuras publicaciones.

---

*Informe generado automaticamente por LinkedIn Content Manager*
"""
    
    return report


def main():
    parser = argparse.ArgumentParser(description="Generate LinkedIn content performance report")
    parser.add_argument("--period", choices=["weekly", "monthly"], default="weekly",
                        help="Report period (default: weekly)")
    parser.add_argument("--output", type=str, help="Output file path (default: stdout)")
    parser.add_argument("--data-path", type=str, default="../data",
                        help="Path to data directory")
    
    args = parser.parse_args()
    
    data_path = Path(args.data_path)
    if not data_path.is_absolute():
        data_path = Path(__file__).parent / data_path
    
    posts = load_posts(data_path)
    filtered_posts = filter_posts_by_period(posts, args.period)
    report = generate_report(filtered_posts, args.period)
    
    if args.output:
        with open(args.output, "w", encoding="utf-8") as f:
            f.write(report)
        print(f"Report saved to {args.output}")
    else:
        print(report)


if __name__ == "__main__":
    main()
