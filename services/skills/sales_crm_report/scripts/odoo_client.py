# -*- coding: utf-8 -*-
"""
INDAWS CRM Report - Odoo XML-RPC Client
========================================
Handles all communication with the Odoo instance via XML-RPC.
All credentials are sourced from the config module (environment variables).
"""

import xmlrpc.client
from datetime import datetime

from . import config


class OdooClient:
    """XML-RPC wrapper for Odoo CRM operations."""

    def __init__(self):
        self.url = config.ODOO_URL.rstrip("/") + "/"
        self.db = config.ODOO_DB
        self.username = config.ODOO_USERNAME
        self.password = config.ODOO_PASSWORD
        self.uid = None
        self.models = None

    def connect(self):
        """Authenticate via XML-RPC and store uid + models proxy."""
        common = xmlrpc.client.ServerProxy(f"{self.url}xmlrpc/2/common")
        self.uid = common.authenticate(self.db, self.username, self.password, {})
        if not self.uid:
            raise Exception("Odoo authentication failed. Check credentials.")
        self.models = xmlrpc.client.ServerProxy(f"{self.url}xmlrpc/2/object")
        return self.uid

    def _execute(self, model, method, *args, **kwargs):
        """Shorthand for models.execute_kw."""
        return self.models.execute_kw(
            self.db, self.uid, self.password, model, method, *args, **kwargs
        )

    def fetch_leads(self, cutoff_date):
        """Fetch CRM leads (opportunities) from Odoo.

        Domain: opportunities only, created >= cutoff OR probability >= 0,
        active in [True, False] to include archived leads.
        """
        domain = [
            "|",
            ["create_date", ">=", cutoff_date.strftime("%Y-%m-%d")],
            ["probability", ">=", 0],
            ["active", "in", [True, False]],
            ["type", "=", "opportunity"],
        ]
        fields = [
            "name", "user_id", "stage_id", "source_id", "expected_revenue",
            "probability", "date_deadline", "create_date", "write_date",
            "partner_name", "date_closed", "x_studio_demo_realizada",
            "x_studio_fecha_demo", "active",
        ]
        leads = self._execute("crm.lead", "search_read", [domain], {"fields": fields})
        return leads

    def fetch_stages(self):
        """Return {id: name} dict for all CRM stages."""
        records = self._execute(
            "crm.stage", "search_read", [[]], {"fields": ["name"]}
        )
        return {s["id"]: s["name"] for s in records}

    def fetch_sources(self):
        """Return {id: name} dict for all UTM sources."""
        records = self._execute(
            "utm.source", "search_read", [[]], {"fields": ["name"]}
        )
        return {s["id"]: s["name"] for s in records}

    def fetch_users(self):
        """Return {id: name} dict for all Odoo users."""
        records = self._execute(
            "res.users", "search_read", [[]], {"fields": ["name"]}
        )
        return {u["id"]: u["name"] for u in records}

    def fetch_sales_orders(self, start_date):
        """Fetch confirmed sale.order records since start_date."""
        domain = [
            ["state", "in", ["sale", "done"]],
            ["date_order", ">=", start_date.strftime("%Y-%m-%d %H:%M:%S")],
        ]
        fields = [
            "name", "user_id", "amount_untaxed", "date_order", "state", "partner_id",
        ]
        orders = self._execute("sale.order", "search_read", [domain], {"fields": fields})
        return orders

    def fetch_all_data(self, cutoff_date):
        """Fetch leads, stages, sources, and users in one call.

        Returns: (leads, stages, sources, users)
        """
        leads = self.fetch_leads(cutoff_date)
        stages = self.fetch_stages()
        sources = self.fetch_sources()
        users = self.fetch_users()
        return leads, stages, sources, users

    def get_user_ids(self):
        """Find Odoo user IDs for salespeople defined in SALESPEOPLE_OBJECTIVES."""
        names = list(config.SALESPEOPLE_OBJECTIVES.keys())
        users = self._execute(
            "res.users", "search_read",
            [[["name", "in", names]]],
            {"fields": ["id", "name"]},
        )
        user_ids = [u["id"] for u in users]
        print(
            f"   - Found {len(user_ids)} users for activities: "
            f"{', '.join(u['name'] for u in users)}"
        )
        return user_ids

    def test_connection(self):
        """Simple connection test. Returns True on success, False on failure."""
        try:
            self.connect()
            return True
        except Exception:
            return False
