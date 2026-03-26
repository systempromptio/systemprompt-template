---
name: "Odoo Importation"
description: "Bulk data import into Odoo using the official load method. Supports XLSX/CSV, batch processing, and external ID generation"
---

---|------------------------|----------------------------------|
| Speed | Very fast (batch) | Slow (1 by 1) |
| Duplicate detection | Automatic (external ID) | Manual (search + create/write) |
| Errors | Per row with detail | Per individual record |
| Code complexity | Low (simple format) | High (manual logic) |
| Relations | Automatic XML IDs | Manual numeric IDs |

## Data Format for load

The `load` method expects two parameters:

```python
result = models.execute_kw(
    DB_NAME, uid, API_KEY,
    'odoo.model', 'load',
    [fields, data],  # ← These two parameters
    {}
)
```

### 1. fields: List of field names

```python
fields = [
    'id',              # External ID (mandatory)
    'name',            # Simple field
    'email',           # Simple field
    'country_id/id',   # Many2one relation (by XML ID)
    'category_id/id',  # Many2many relation (by XML ID)
]
```

### Field Syntax:

| Field Type | Syntax | Example |
|------------|--------|---------|
| Simple field | `'field_name'` | `'name'`, `'email'` |
| Many2one | `'field_name/id'` | `'country_id/id'` |
| Many2many | `'field_name/id'` | `'category_id/id'` |
| External ID | `'id'` | `'id'` (always include) |

### 2. data: List of lists with values

Each row is a list of values in the same order as `fields`.

```python
data = [
    # Row 1
    ['__import__.contact_1', 'John Doe', 'john@email.com', 'base.us', ''],
    # Row 2
    ['__import__.contact_2', 'Jane Smith', 'jane@email.com', 'base.uk', ''],
]
```

## External IDs (Mandatory)

### What is an External ID?

An **External ID** (XML ID) is a unique, human-readable identifier that allows stable reference to records between imports.

### Why are they mandatory?

1. **Prevent duplicates**: If you reimport, it will update instead of duplicate
2. **Traceability**: You can track which records came from imports
3. **Idempotence**: You can run the same import multiple times without issues
4. **Updates**: Modify the file and reimport to update records

### External ID Format

For imports, the format is:

```
__import__.{unique_identifier}
```

**Examples**:
```
__import__.contact_john_doe_001
__import__.product_laptop_dell_x123
__import__.invoice_2024_01_001
```

### External ID Generation Strategies

#### Option 1: Based on record data (Recommended)

```python
def generate_external_id(row, index):
    """Generates external ID based on data"""
    # Normalize name: remove accents, spaces, lowercase
    name = normalize_text(row['Name'])
    name = name.replace(' ', '_')

    # Add index to guarantee uniqueness
    return f"__import__.contact_{name}_{index}"
```

#### Option 2: Based on source system ID

```python
def generate_external_id(row):
    """Generates external ID from original system"""
    original_id = row['source_system_id']
    return f"__import__.contact_legacy_{original_id}"
```

#### Option 3: UUID (if no unique data)

```python
import uuid

def generate_external_id():
    """Generates unique external ID with UUID"""
    unique_id = str(uuid.uuid4())[:8]
    return f"__import__.contact_{unique_id}"
```

## XML IDs for Relations

### What are XML IDs?

**XML IDs** are unique identifiers that Odoo assigns to master data records (countries, currencies, categories, etc.).

### Why use them instead of numeric IDs?

| Aspect | XML ID (✅) | Numeric ID (❌) |
|--------|------------|------------------|
| **Stability** | Identical in all databases | Different per database |
| **Portability** | Works in any environment | Only in specific database |
| **Readability** | `base.es` (Spain) | `68` (what is it?) |

### Common XML IDs in Odoo

#### Countries (res.country)
```
base.es  → Spain
base.mx  → Mexico
base.us  → United States
base.fr  → France
base.uk  → United Kingdom
```

#### States/Provinces (res.country.state)
```
base.state_es_m   → Madrid (Spain)
base.state_us_ca  → California (USA)
base.state_us_ny  → New York (USA)
```

#### Currencies (res.currency)
```
base.EUR  → Euro
base.USD  → US Dollar
base.GBP  → British Pound
```

### Getting XML IDs from Records

```python
def get_xmlid_for_record(uid, models, model_name, record_id):
    """Gets the XML ID of a record given its numeric ID"""
    ext_id = models.execute_kw(
        DB_NAME, uid, API_KEY,
        'ir.model.data', 'search_read',
        [[['model', '=', model_name], ['res_id', '=', int(record_id)]]],
        {'fields': ['name', 'module'], 'limit': 1}
    )

    if ext_id:
        return f"{ext_id[0]['module']}.{ext_id[0]['name']}"
    else:
        return str(int(record_id))  # Fallback to numeric ID
```

## Batch Importing

### Maximum Batch Size

**RULE**: Never exceed **1000 records per batch**.

**Reasons**:
1. **Timeouts**: Very large requests can exceed time limits
2. **Memory**: Large transactions consume a lot of RAM
3. **Debugging**: Smaller batches make it easier to locate errors

### Batching Implementation

```python
BATCH_SIZE = 1000

def import_with_batching(uid, models, fields, all_data):
    """Imports data in batches of BATCH_SIZE"""
    total_records = len(all_data)
    total_batches = (total_records + BATCH_SIZE - 1) // BATCH_SIZE

    print(f"Total records: {total_records}")
    print(f"Batch size: {BATCH_SIZE}")
    print(f"Total batches: {total_batches}")

    total_imported = 0
    total_errors = 0

    for i in range(total_batches):
        start_idx = i * BATCH_SIZE
        end_idx = min((i + 1) * BATCH_SIZE, total_records)
        batch_data = all_data[start_idx:end_idx]

        print(f"\\nProcessing batch {i+1}/{total_batches} ({len(batch_data)} records)...")

        try:
            result = models.execute_kw(
                DB_NAME, uid, API_KEY,
                'res.partner', 'load',
                [fields, batch_data],
                {}
            )

            ids = result.get('ids', [])
            messages = result.get('messages', [])

            # Analyze result
            errors = [msg for msg in messages if msg.get('type') == 'error']

            if errors:
                print(f"  ❌ Errors found:")
                for error in errors:
                    row_num = error.get('rows', {}).get('from', '?')
                    print(f"     Row {row_num}: {error.get('message')}")
                total_errors += len(errors)
            else:
                imported = len(ids) if ids else len(batch_data)
                print(f"  ✅ Batch processed: {imported} records")
                total_imported += imported

        except Exception as e:
            print(f"  ❌ Error processing batch: {e}")
            total_errors += len(batch_data)

    return total_imported, total_errors
```

## Response and Error Handling

### Response Structure

```python
result = {
    'ids': [14, 15, 16],  # IDs of created/updated records
    'messages': [         # List of errors/warnings
        {
            'type': 'error',        # 'error' or 'warning'
            'message': 'Error description',
            'rows': {
                'from': 2,          # Row where it occurred (0-indexed)
                'to': 2
            },
            'field': 'email'        # Field that caused the error
        }
    ]
}
```

### Interpreting Results

#### ✅ Successful Import

```python
if not result.get('messages'):
    ids = result.get('ids', [])
    print(f"✅ Successfully imported {len(ids)} records")
```

#### ❌ Errors

```python
errors = [m for m in result.get('messages', []) if m.get('type') == 'error']
if errors:
    print(f"❌ {len(errors)} errors:")
    for error in errors:
        row = error.get('rows', {}).get('from', '?')
        field = error.get('field', 'unknown')
        message = error.get('message', 'Unknown error')
        print(f"   Row {row+2} (field '{field}'): {message}")
```

## Pre-Import Normalization

### Why Normalize?

Data from external files usually has:
- **Typographical errors**: "Graná" instead of "Granada"
- **Format variations**: "España", "Spain"
- **Inconsistencies**: "Madrid" vs "MADRID" vs "madrid"
- **Missing or incomplete data**

### Text Normalization Function

```python
import unicodedata

def normalize_text(text):
    """
    Normalizes text for comparison:
    - Lowercase
    - No accents
    - No extra spaces
    """
    if pd.isna(text) or not text:
        return ""

    text = str(text).lower().strip()

    # Remove accents
    text = unicodedata.normalize('NFD', text)
    text = ''.join(
        char for char in text
        if unicodedata.category(char) != 'Mn'
    )

    return text
```

### Fuzzy Matching

```python
from difflib import get_close_matches

def find_best_match(value, candidates, threshold=0.6):
    """Finds the best match using text similarity"""
    if not value or pd.isna(value):
        return None

    normalized_value = normalize_text(value)

    # Create dictionary of normalized candidates
    normalized_candidates = {}
    for candidate in candidates:
        name = normalize_text(candidate.get('name', ''))
        if name:
            normalized_candidates[name] = candidate

    # Exact search first
    if normalized_value in normalized_candidates:
        return normalized_candidates[normalized_value]

    # Fuzzy search
    matches = get_close_matches(
        normalized_value,
        normalized_candidates.keys(),
        n=1,
        cutoff=threshold
    )

    if matches:
        return normalized_candidates[matches[0]]

    return None
```

## Complete Workflow

```
1. READ ORIGINAL FILE (XLSX/CSV)
   ↓
2. CONNECT TO ODOO (authenticate, get uid)
   ↓
3. GET ODOO CATALOGS (countries, states, etc.)
   ↓
4. NORMALIZE DATA (fix countries, validate fields)
   ↓
5. MAP TO XML IDs (convert relations to XML IDs)
   ↓
6. GENERATE NORMALIZED FILE (for audit)
   ↓
7. PREPARE DATA FOR LOAD (fields + data arrays)
   ↓
8. IMPORT WITH LOAD METHOD IN BATCHES
   ↓
9. REPORT RESULTS (total imported, errors)
```

## Common Errors and Solutions

| Error | Cause | Solution |
|-------|-------|----------|
| `No matching records found for external id 'X'` | Incorrect XML ID in relation | Verify XML ID exists |
| `Field X is required` | Missing required value | Add value for field |
| `Record already exists with this email` | Uniqueness constraint | Change duplicate value |
| `Invalid value for field X` | Incorrect data type | Verify value format |

## Best Practices

1. **Always use external IDs**: Prevents duplicates on reimport
2. **Normalize data first**: Clean and validate before importing
3. **Use XML IDs for relations**: More portable than numeric IDs
4. **Batch imports**: Never exceed 1000 records per batch
5. **Test with small sample**: Import 10-20 records first to validate
6. **Save normalized file**: Keep audit trail of transformations
7. **Handle errors gracefully**: Report errors clearly to user

## Reference Documentation

For more information about Odoo connection and API operations, see:
- Skill: `common-skills:odoo-connection` - Use this skill when you need to connect to Odoo instances, execute API calls, authenticate, or interact with Odoo models. Supports both Odoo < 19.0 (JSON-RPC) and Odoo >= 19.0 (JSON2 protocol).
- Odoo Documentation: https://www.odoo.com/documentation/
