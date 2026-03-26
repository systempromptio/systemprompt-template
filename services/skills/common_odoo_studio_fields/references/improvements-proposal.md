# Propuesta de Mejoras - odoo-studio-fields

Análisis comparativo entre la implementación actual y las convenciones reales de Odoo Studio.

## Estado Actual vs Convenciones Studio

### ✅ Ya implementado correctamente

| Característica | Implementación |
|----------------|----------------|
| Prefijo `x_studio_` | ✅ `generate_studio_name()` |
| `state = 'manual'` | ✅ En `field_values` |
| Registro en ir.model.data | ✅ Con `studio = True` |
| Opciones de selección | ✅ En ir.model.fields.selection |
| UUID en XML ID | ✅ `generate_studio_uuid()` |
| Module studio_customization | ✅ Usado correctamente |

### ❌ Falta o necesita mejora

## 1. Auto-sufijo para campos duplicados

**Problema:** La skill falla si el campo ya existe.

**Comportamiento Studio:** Añade `_1`, `_2`, etc. automáticamente.

**Mejora propuesta:**
```python
def find_available_name(api, model, base_name):
    """Encuentra nombre disponible añadiendo sufijo si es necesario."""
    if not api.field_exists(model, base_name):
        return base_name

    suffix = 1
    while suffix <= 99:
        new_name = f"{base_name}_{suffix}"
        if not api.field_exists(model, new_name):
            return new_name
        suffix += 1
    raise OdooAPIError(f"Too many fields with base name {base_name}")
```

**Opción CLI:**
```bash
./create_field.py crm.lead "Import" float '{"auto_suffix": true}'
```

---

## 2. Soporte para XML-RPC

**Problema:** Solo funciona JSON-RPC, pero algunas instancias requieren XML-RPC (especialmente con API Keys en Odoo 18).

**Mejora propuesta en odoo_api.py:**
```python
def __init__(self):
    # ...
    self.protocol = os.environ.get('ODOO_PROTOCOL', 'auto')  # auto, jsonrpc, xmlrpc

def _call_xmlrpc(self, model, method, args, kwargs):
    """XML-RPC call usando xmlrpc.client."""
    import xmlrpc.client
    models = xmlrpc.client.ServerProxy(f'{self.url}/xmlrpc/2/object')
    return models.execute_kw(
        self.db, self.uid, self.api_key,
        model, method, args, kwargs or {}
    )
```

---

## 3. Posicionamiento en grupos específicos

**Problema:** `add_to_view.py` no soporta añadir campos dentro de grupos con nombres como `studio_group_xxx_left`.

**Mejora propuesta:**
```bash
# Nuevo parámetro: --group
./add_to_view.py crm.lead form x_studio_campo inside studio_group_5dd_left '{}'
```

**Implementación:**
```python
if reference.startswith('studio_group_'):
    xpath_expr = f"//group[@name='{reference}']"
```

---

## 4. Script para crear páginas/pestañas

**Nuevo script: create_page.py**
```bash
./create_page.py crm.lead form "Mi Pestaña" '{"after_page": "internal_notes"}'
```

**Genera:**
```xml
<page string="Mi Pestaña" name="studio_page_{uuid}">
  <group name="studio_group_{uuid}">
    <group name="studio_group_{uuid}_left"/>
    <group name="studio_group_{uuid}_right"/>
  </group>
</page>
```

---

## 5. Soporte para campos monetarios

**Problema:** Los campos monetary necesitan un currency_id asociado.

**Mejora:**
```python
if ttype == 'monetary':
    field_values['currency_field'] = options.get('currency_field', 'currency_id')
    # Verificar que el campo currency existe o crearlo
```

**Uso:**
```bash
./create_field.py sale.order "Custom Price" monetary '{"currency_field": "currency_id"}'
```

---

## 6. Campos relacionales avanzados

### Many2many
```python
if ttype == 'many2many':
    field_values['relation'] = options['relation']
    field_values['relation_table'] = options.get('relation_table')  # Auto-generar si no se especifica
    field_values['column1'] = options.get('column1', f'{model.replace(".", "_")}_id')
    field_values['column2'] = options.get('column2', f'{relation.replace(".", "_")}_id')
```

### One2many
```python
if ttype == 'one2many':
    field_values['relation'] = options['relation']
    field_values['relation_field'] = options['relation_field']  # Campo inverso requerido
```

---

## 7. Batch creation (crear múltiples campos)

**Nuevo script: create_fields_batch.py**
```bash
./create_fields_batch.py crm.lead fields.json
```

**fields.json:**
```json
{
  "fields": [
    {"name": "Campo 1", "type": "char"},
    {"name": "Campo 2", "type": "boolean"},
    {"name": "Campo 3", "type": "selection", "options": {...}}
  ],
  "view": {
    "type": "form",
    "group": "studio_group_5dd_left"
  }
}
```

---

## 8. Validación de convenciones

**Nuevo script: validate_studio.py**

Verifica que los campos/vistas creados siguen las convenciones de Studio:
```bash
./validate_studio.py crm.lead

# Output:
# ✓ x_studio_expedient_1: Válido
# ✗ x_custom_field: No sigue convención x_studio_
# ✓ Vista 2411: Formato correcto
```

---

## 9. Export de customizaciones

**Nuevo script: export_customizations.py**

Similar al wizard de Studio para exportar como módulo:
```bash
./export_customizations.py crm.lead --output ./my_customization_module
```

Genera estructura de módulo Odoo con los campos y vistas.

---

## Estado de Implementación

| Prioridad | Mejora | Estado |
|-----------|--------|--------|
| 🔴 Alta | Auto-sufijo campos | ✅ Implementado |
| 🔴 Alta | Soporte XML-RPC | ✅ Implementado |
| 🟡 Media | Grupos específicos | ✅ Implementado |
| 🟡 Media | Crear páginas | ✅ Implementado (`create_page.py`) |
| 🟡 Media | Campos monetarios | ✅ Implementado |
| 🟡 Media | Campos relacionales | ✅ Implementado (many2many, one2many) |
| 🟢 Baja | Batch creation | ✅ Implementado (`create_fields_batch.py`) |
| 🟢 Baja | Validación | ✅ Implementado (`validate_studio.py`) |
| 🟢 Baja | Export módulo | ✅ Implementado (`export_customizations.py`) |
| 🟡 Media | Crear modelos | ✅ Implementado (`create_model.py`) |
| 🟡 Media | Crear menús | ✅ Implementado (`create_menu.py`) |
| 🟡 Media | Agrupar campos en xpath existente | ✅ Implementado (add_to_view.py mejorado) |
| 🟡 Media | Importar zip Studio | ✅ Implementado (`validate_zip.py`, `import_studio_zip.py`) |

---

## Roadmap - Próximas Mejoras

### Tipos de campo pendientes

| Tipo | Descripción | Dificultad | Prioridad |
|------|-------------|------------|-----------|
| `binary` | Archivos/adjuntos | 🟢 Fácil | Media |
| `image` | Imágenes (binary especial) | 🟢 Fácil | Media |
| `reference` | Many2one polimórfico | 🟡 Media | Baja |
| `properties` | Campos dinámicos (Odoo 17+) | 🔴 Alta | Baja |
| `json` | Datos JSON (Odoo 17+) | 🟡 Media | Baja |

### Otras mejoras futuras

| Mejora | Descripción | Prioridad |
|--------|-------------|-----------|
| Widgets personalizados | Especificar widget al añadir campo a vista (`phone`, `image`, `badge`, etc.) | 🟡 Media |
| Importar desde CSV/Excel | Definir campos en hoja de cálculo e importar en batch | 🟡 Media |
| Rollback | Deshacer último campo/vista creado (guarda historial de cambios) | 🟢 Alta |

---

*Documento generado: 2026-02-02*
*Última actualización: 2026-02-10 - Añadido validate_zip.py, import_studio_zip.py, docs de importación*
