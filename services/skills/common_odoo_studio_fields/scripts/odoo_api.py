#!/usr/bin/env python3
"""
Odoo API client for Studio field operations.
Supports JSON-RPC (Odoo < 19), JSON2 (Odoo >= 19), and XML-RPC protocols.

Usage:
    from odoo_api import OdooAPI
    api = OdooAPI()
    api.call('res.partner', 'search_count', [[('is_company', '=', True)]])

Environment variables:
    ODOO_URL        - Odoo instance URL (required)
    ODOO_DB         - Database name (required)
    ODOO_SESSION_ID - Session cookie (for JSON-RPC)
    ODOO_KEY        - API key or password
    ODOO_USER       - Username (for JSON-RPC/XML-RPC auth)
    ODOO_UID        - User ID (for XML-RPC, auto-detected if not set)
    ODOO_PROTOCOL   - 'jsonrpc', 'json2', 'xmlrpc', or 'auto' (default: 'auto')
"""

import os
import sys
import json
import re
import uuid
import requests
import xmlrpc.client
from typing import Any, Optional
from urllib.parse import urljoin


class OdooAPIError(Exception):
    """Odoo API error with details."""
    pass


class OdooAPI:
    """Odoo API client supporting JSON-RPC, JSON2, and XML-RPC protocols."""

    def __init__(self):
        self.url = os.environ.get('ODOO_URL', '').rstrip('/')
        self.db = os.environ.get('ODOO_DB', '')
        self.session_id = os.environ.get('ODOO_SESSION_ID', '')
        self.api_key = os.environ.get('ODOO_KEY', '')
        self.user = os.environ.get('ODOO_USER', '')
        self.uid = os.environ.get('ODOO_UID', '')
        self.protocol = os.environ.get('ODOO_PROTOCOL', 'auto')

        if not self.url:
            raise OdooAPIError("ODOO_URL not set")
        if not self.db:
            raise OdooAPIError("ODOO_DB not set")

        # Auto-detect protocol if set to 'auto'
        if self.protocol == 'auto':
            self.protocol = self._auto_detect_protocol()

    def _auto_detect_protocol(self) -> str:
        """Auto-detect the best protocol based on available credentials and server."""
        # If we have a session_id, use JSON-RPC
        if self.session_id:
            return 'jsonrpc'

        # If we have UID, we can use XML-RPC directly
        if self.uid and self.api_key:
            return 'xmlrpc'

        # Try JSON-RPC authentication first
        if self.user and self.api_key:
            try:
                response = requests.post(
                    f"{self.url}/web/session/authenticate",
                    json={
                        "jsonrpc": "2.0",
                        "method": "call",
                        "params": {
                            "db": self.db,
                            "login": self.user,
                            "password": self.api_key
                        },
                        "id": 1
                    },
                    timeout=10
                )
                data = response.json()
                if 'error' not in data and response.cookies.get('session_id'):
                    self.session_id = response.cookies.get('session_id')
                    return 'jsonrpc'
            except:
                pass

            # JSON-RPC failed, try XML-RPC
            try:
                common = xmlrpc.client.ServerProxy(f'{self.url}/xmlrpc/2/common')
                uid = common.authenticate(self.db, self.user, self.api_key, {})
                if uid:
                    self.uid = str(uid)
                    return 'xmlrpc'
            except:
                pass

        # Default to jsonrpc (will fail later if no credentials)
        return 'jsonrpc'

    def _ensure_session(self) -> str:
        """Ensure we have a valid session, authenticate if needed."""
        if self.session_id:
            return self.session_id

        if not self.api_key or not self.user:
            raise OdooAPIError("No session and no credentials (ODOO_KEY, ODOO_USER)")

        # Authenticate
        response = requests.post(
            f"{self.url}/web/session/authenticate",
            json={
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "db": self.db,
                    "login": self.user,
                    "password": self.api_key
                },
                "id": 1
            }
        )

        # Extract session from cookies
        self.session_id = response.cookies.get('session_id', '')
        if not self.session_id:
            raise OdooAPIError("Authentication failed: no session received")

        # Check for auth errors
        data = response.json()
        if 'error' in data:
            msg = data['error'].get('data', {}).get('message', 'Unknown error')
            raise OdooAPIError(f"Authentication failed: {msg}")

        return self.session_id

    def _ensure_uid(self) -> int:
        """Ensure we have a valid UID for XML-RPC, authenticate if needed."""
        if self.uid:
            return int(self.uid)

        if not self.api_key or not self.user:
            raise OdooAPIError("No UID and no credentials (ODOO_KEY, ODOO_USER)")

        # Authenticate via XML-RPC
        try:
            common = xmlrpc.client.ServerProxy(f'{self.url}/xmlrpc/2/common')
            uid = common.authenticate(self.db, self.user, self.api_key, {})
            if not uid:
                raise OdooAPIError("XML-RPC authentication failed: invalid credentials")
            self.uid = str(uid)
            return uid
        except xmlrpc.client.Fault as e:
            raise OdooAPIError(f"XML-RPC authentication failed: {e.faultString}")
        except Exception as e:
            raise OdooAPIError(f"XML-RPC authentication failed: {str(e)}")

    def call(self, model: str, method: str, args: list = None, kwargs: dict = None) -> Any:
        """
        Call an Odoo model method.

        Args:
            model: Model name (e.g., 'res.partner')
            method: Method name (e.g., 'search_read')
            args: Positional arguments
            kwargs: Keyword arguments

        Returns:
            Method result
        """
        args = args or []
        kwargs = kwargs or {}

        if self.protocol == 'json2':
            return self._call_json2(model, method, args, kwargs)
        elif self.protocol == 'xmlrpc':
            return self._call_xmlrpc(model, method, args, kwargs)
        else:
            return self._call_jsonrpc(model, method, args, kwargs)

    def _call_jsonrpc(self, model: str, method: str, args: list, kwargs: dict) -> Any:
        """JSON-RPC call for Odoo < 19."""
        session = self._ensure_session()

        payload = {
            "jsonrpc": "2.0",
            "method": "call",
            "params": {
                "model": model,
                "method": method,
                "args": args,
                "kwargs": kwargs
            },
            "id": int(uuid.uuid4().int % 1000000)
        }

        response = requests.post(
            f"{self.url}/web/dataset/call_kw",
            json=payload,
            cookies={"session_id": session}
        )

        data = response.json()

        if 'error' in data:
            error_data = data['error'].get('data', {})
            msg = error_data.get('message', data['error'].get('message', 'Unknown error'))
            raise OdooAPIError(msg)

        return data.get('result')

    def _call_json2(self, model: str, method: str, args: list, kwargs: dict) -> Any:
        """JSON2 call for Odoo >= 19."""
        headers = {
            "Authorization": f"bearer {self.api_key}",
            "X-Odoo-Database": self.db,
            "Content-Type": "application/json"
        }

        # JSON2 uses kwargs directly in body
        body = kwargs if kwargs else {}
        if args:
            body['args'] = args

        response = requests.post(
            f"{self.url}/json/2/{model}/{method}",
            json=body,
            headers=headers
        )

        data = response.json()

        if 'error' in data:
            msg = data['error'].get('message', 'Unknown error')
            raise OdooAPIError(msg)

        return data

    def _call_xmlrpc(self, model: str, method: str, args: list, kwargs: dict) -> Any:
        """XML-RPC call for Odoo (works with API keys in Odoo 18+)."""
        uid = self._ensure_uid()

        try:
            models = xmlrpc.client.ServerProxy(f'{self.url}/xmlrpc/2/object')
            result = models.execute_kw(
                self.db, uid, self.api_key,
                model, method, args, kwargs or {}
            )
            return result
        except xmlrpc.client.Fault as e:
            raise OdooAPIError(f"XML-RPC error: {e.faultString}")
        except Exception as e:
            raise OdooAPIError(f"XML-RPC error: {str(e)}")

    # ==================== Helper Methods ====================

    def model_exists(self, model: str) -> bool:
        """Check if a model exists."""
        try:
            count = self.call('ir.model', 'search_count', [[['model', '=', model]]])
            return count > 0
        except OdooAPIError:
            return False

    def field_exists(self, model: str, field_name: str) -> bool:
        """Check if a field exists on a model."""
        try:
            count = self.call('ir.model.fields', 'search_count', [
                [['model', '=', model], ['name', '=', field_name]]
            ])
            return count > 0
        except OdooAPIError:
            return False

    def get_model_id(self, model: str) -> Optional[int]:
        """Get the ir.model ID for a model."""
        result = self.call('ir.model', 'search', [[['model', '=', model]]])
        return result[0] if result else None

    def get_field_info(self, model: str, field_name: str) -> Optional[dict]:
        """Get field information."""
        result = self.call('ir.model.fields', 'search_read', [
            [['model', '=', model], ['name', '=', field_name]]
        ], {'fields': ['id', 'name', 'field_description', 'ttype']})
        return result[0] if result else None

    def get_primary_view(self, model: str, view_type: str) -> Optional[dict]:
        """Get the primary view of a model."""
        result = self.call('ir.ui.view', 'search_read', [
            [['model', '=', model], ['type', '=', view_type], ['mode', '=', 'primary']]
        ], {'fields': ['id', 'name', 'arch_db'], 'limit': 1})
        return result[0] if result else None

    @staticmethod
    def generate_studio_name(name: str) -> str:
        """
        Generate a Studio-compatible field name.

        Examples:
            "My Field" -> "x_studio_my_field"
            "x_studio_existing" -> "x_studio_existing"
        """
        # Clean the name
        clean = name.lower().replace(' ', '_')
        clean = re.sub(r'[^a-z0-9_]', '', clean)

        # Add prefix if not present
        if clean.startswith('x_studio_'):
            return clean
        return f"x_studio_{clean}"

    @staticmethod
    def generate_studio_uuid() -> str:
        """Generate a short UUID for Studio XML IDs."""
        return str(uuid.uuid4())[:8]

    @staticmethod
    def map_field_type(user_type: str) -> str:
        """Map user-friendly type names to Odoo ttypes."""
        mapping = {
            'string': 'char', 'varchar': 'char',
            'number': 'integer', 'int': 'integer',
            'decimal': 'float', 'double': 'float',
            'checkbox': 'boolean', 'bool': 'boolean',
            'select': 'selection', 'dropdown': 'selection',
            'relation': 'many2one', 'link': 'many2one',
        }
        return mapping.get(user_type, user_type)


# ==================== CLI Interface ====================

def main():
    """CLI interface for odoo_api.py"""
    import argparse

    parser = argparse.ArgumentParser(description='Odoo API client')
    subparsers = parser.add_subparsers(dest='command', help='Commands')

    # call command
    call_parser = subparsers.add_parser('call', help='Call a model method')
    call_parser.add_argument('model', help='Model name')
    call_parser.add_argument('method', help='Method name')
    call_parser.add_argument('--args', default='[]', help='JSON args')
    call_parser.add_argument('--kwargs', default='{}', help='JSON kwargs')

    # model_exists command
    model_parser = subparsers.add_parser('model_exists', help='Check if model exists')
    model_parser.add_argument('model', help='Model name')

    # field_exists command
    field_parser = subparsers.add_parser('field_exists', help='Check if field exists')
    field_parser.add_argument('model', help='Model name')
    field_parser.add_argument('field', help='Field name')

    # studio_name command
    name_parser = subparsers.add_parser('studio_name', help='Generate studio field name')
    name_parser.add_argument('name', help='Field name')

    args = parser.parse_args()

    try:
        api = OdooAPI()

        if args.command == 'call':
            result = api.call(
                args.model,
                args.method,
                json.loads(args.args),
                json.loads(args.kwargs)
            )
            print(json.dumps(result))

        elif args.command == 'model_exists':
            exists = api.model_exists(args.model)
            print('true' if exists else 'false')
            sys.exit(0 if exists else 1)

        elif args.command == 'field_exists':
            exists = api.field_exists(args.model, args.field)
            print('true' if exists else 'false')
            sys.exit(0 if exists else 1)

        elif args.command == 'studio_name':
            print(OdooAPI.generate_studio_name(args.name))

        else:
            parser.print_help()
            sys.exit(1)

    except OdooAPIError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
