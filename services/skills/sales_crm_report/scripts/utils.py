# -*- coding: utf-8 -*-
"""
INDAWS CRM Report - Utility Functions
======================================
Shared helpers used across all modules.
"""

from datetime import datetime


def parse_date(date_str):
    """Parse Odoo date string to datetime object.

    Handles ISO format with/without timezone and plain YYYY-MM-DD.
    Returns None for empty or unparseable strings.
    """
    if not date_str:
        return None
    try:
        return datetime.fromisoformat(date_str.replace('Z', '+00:00'))
    except (ValueError, TypeError):
        try:
            return datetime.strptime(date_str[:10], '%Y-%m-%d')
        except (ValueError, TypeError):
            return None


def truncate_text(text, limit):
    """Truncate text to a maximum length, adding ellipsis if needed."""
    if not text:
        return ""
    if len(text) <= limit:
        return text
    return text[:limit - 3] + "..."
