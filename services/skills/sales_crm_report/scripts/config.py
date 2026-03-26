# -*- coding: utf-8 -*-
"""
INDAWS CRM Report - Centralized Configuration
==============================================
All credentials are loaded from environment variables.
Business rules, thresholds, and brand constants are defined here.
"""

import os

# =============================================================================
# ODOO CONNECTION (all from environment variables)
# =============================================================================
ODOO_URL = os.environ.get("ODOO_URL", "https://www.indaws.es/")
ODOO_DB = os.environ.get("ODOO_DB", "master")
ODOO_USERNAME = os.environ.get("ODOO_USER", "")
ODOO_PASSWORD = os.environ.get("ODOO_KEY", "")

# =============================================================================
# SALESPEOPLE & MONTHLY OBJECTIVES
# =============================================================================
SALESPEOPLE_OBJECTIVES = {
    "Iñaki Núñez": 80000,
    "Jose Vicente Tarazona": 80000,
    "Aitana Arranz Carrasco": 10000,
    "Anxo Nieto Lojo": 10000,
    "Celia de Lara Zarzuela": 25000,
}

SALESPEOPLE_EMAILS = {
    "Iñaki Núñez": "inaki@indaws.es",
    "Jose Vicente Tarazona": "josevicente@indaws.es",
    "Aitana Arranz Carrasco": "aitana.arranz@indaws.es",
    "Anxo Nieto Lojo": "anxo.nieto@indaws.es",
    "Celia de Lara Zarzuela": "celia@indaws.es",
}

EXCLUDED_SALESPEOPLE = [
    "Víctor Peris",
    "Julio Pons",
]

# =============================================================================
# ACTIVITY THRESHOLDS
# =============================================================================
DAYS_WARNING = 5
DAYS_CRITICAL = 7
PROPOSAL_FOLLOWUP_DAYS = 3
HIGH_VALUE_THRESHOLD = 15000
END_OF_MONTH_PRESSURE_DAYS = 7

# =============================================================================
# TARGETS
# =============================================================================
WEEKLY_NEW_OPPORTUNITIES_TARGET = 1
TARGET_CONVERSION_RATE = 0.30
GROWTH_TARGET_ANNUAL = 0.50
WEEKLY_LEAD_TARGET = 10
DEMOS_MONTHLY_TARGET = 16

# =============================================================================
# STAGE CLASSIFICATIONS
# =============================================================================
STAGE_WON_KEYWORDS = ["Ganado", "Won", "Cerrado ganado"]
STAGE_LOST_KEYWORDS = ["Perdido", "Lost", "Cerrado perdido"]
STAGE_WAITING_KEYWORDS = ["En Espera", "Waiting", "Standby"]
STAGE_PROPOSAL_KEYWORDS = ["Propuesta", "Proposal", "Cotización", "Quote"]
STAGE_EARLY_KEYWORDS = ["Nuevo", "New", "1er contacto", "Cualificado", "Qualified"]

# =============================================================================
# PRODUCT/SERVICE CATEGORIES
# =============================================================================
PRODUCT_CATEGORIES = {
    "Análisis": ["análisis", "consultoría", "auditoría", "diagnóstico"],
    "Implementación": ["implementación", "implantación", "migración", "proyecto"],
    "Productos Indaws Core": ["odoo", "erp", "crm", "facturación", "contabilidad"],
    "Otros": [],
}

# =============================================================================
# DATA LOOKBACK
# =============================================================================
MONTHS_LOOKBACK = 6

# =============================================================================
# EMAIL CONFIGURATION (SMTP)
# =============================================================================
EMAIL_SMTP_SERVER = os.environ.get("EMAIL_SMTP_SERVER", "smtp.gmail.com")
EMAIL_SMTP_PORT = int(os.environ.get("EMAIL_SMTP_PORT", "587"))
EMAIL_USE_TLS = os.environ.get("EMAIL_USE_TLS", "true").lower() == "true"
EMAIL_FROM = os.environ.get("EMAIL_FROM", "victor@indaws.es")
EMAIL_USERNAME = os.environ.get("EMAIL_USERNAME", "victor@indaws.es")
EMAIL_PASSWORD = os.environ.get("EMAIL_SMTP_PASSWORD", "")

EMAIL_RECIPIENTS = [
    "inaki@indaws.es",
    "victor@indaws.es",
    "aitana.arranz@indaws.es",
    "anxo.nieto@indaws.es",
    "josevicente@indaws.es",
    "celia@indaws.es",
    "julio@indaws.es",
]

EMAIL_SUBJECT_PREFIX = "Reporte CRM Semanal INDAWS"

# =============================================================================
# BRAND CONSTANTS (Indaws Brand Guidelines)
# =============================================================================
BRAND_COLORS = {
    "blue_lilac": "#6B68FA",
    "blue_space": "#1C265D",
    "warm_yellow": "#E5B92B",
    "light_sky": "#8AC2DB",
}

FONT_FAMILY = "Bogle"
FONT_FALLBACK = "Verdana, sans-serif"
FONT_IMPORT_URL = "https://fonts.googleapis.com/css2?family=Bogle:wght@400;600;700;800&display=swap"

# Semantic alert colors (allowed alongside brand colors for data visualization)
ALERT_COLORS = {
    "success": "#16A34A",
    "warning": "#F59E0B",
    "danger": "#DC2626",
    "info": "#3B82F6",
    "neutral": "#64748B",
}
