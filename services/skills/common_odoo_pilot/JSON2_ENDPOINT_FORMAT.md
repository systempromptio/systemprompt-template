# JSON2 API - Formato de Endpoints y Peticiones

## Formato Correcto del Endpoint

### Estructura de URL
```
POST /json/2/{model}/{method}
```

**Ejemplos**:
- `/json/2/res.partner/search_read`
- `/json/2/product.product/create`
- `/json/2/sale.order/action_confirm`
- `/json/2/ir.module.module/button_immediate_install`

### Headers Requeridos

```http
Content-Type: application/json; charset=utf-8
Authorization: bearer {API_KEY}
X-Odoo-Database: {DATABASE_NAME}
```

### Body de la Petición

**NO uses** la estructura JSON-RPC:
```json
❌ INCORRECTO:
{
  "jsonrpc": "2.0",
  "method": "call",
  "params": {
    "model": "res.partner",
    "method": "search_read",
    ...
  },
  "id": 1
}
```

**USA** el formato directo de JSON2:
```json
✅ CORRECTO:
{
  "domain": [["is_company", "=", true]],
  "fields": ["name", "email"],
  "limit": 10
}
```

## Ejemplos por Método

### 1. search_read

**Endpoint**: `/json/2/{model}/search_read`

**Petición**:
```http
POST /json/2/res.partner/search_read HTTP/1.1
Host: your-instance.odoo.com
Authorization: bearer YOUR_API_KEY_HERE
X-Odoo-Database: your_database
Content-Type: application/json

{
  "domain": [
    ["name", "ilike", "%deco%"],
    ["is_company", "=", true]
  ],
  "fields": ["name", "email", "phone"],
  "limit": 10,
  "offset": 0
}
```

**Respuesta**:
```json
[
  {
    "id": 1,
    "name": "Deco Company",
    "email": "info@deco.com",
    "phone": "+1234567890"
  },
  ...
]
```

### 2. create

**Endpoint**: `/json/2/{model}/create`

**Petición**:
```http
POST /json/2/res.partner/create HTTP/1.1
Host: your-instance.odoo.com
Authorization: bearer YOUR_API_KEY_HERE
X-Odoo-Database: your_database
Content-Type: application/json

{
  "name": "New Company",
  "email": "info@newcompany.com",
  "is_company": true
}
```

**Respuesta**:
```json
{
  "id": 123
}
```

### 3. write

**Endpoint**: `/json/2/{model}/write`

**Petición**:
```http
POST /json/2/res.partner/write HTTP/1.1
Host: your-instance.odoo.com
Authorization: bearer YOUR_API_KEY_HERE
X-Odoo-Database: your_database
Content-Type: application/json

{
  "ids": [1, 2, 3],
  "values": {
    "phone": "+34912345678",
    "mobile": "+34600111222"
  }
}
```

**Respuesta**:
```json
true
```

### 4. unlink

**Endpoint**: `/json/2/{model}/unlink`

**Petición**:
```http
POST /json/2/res.partner/unlink HTTP/1.1
Host: your-instance.odoo.com
Authorization: bearer YOUR_API_KEY_HERE
X-Odoo-Database: your_database
Content-Type: application/json

{
  "ids": [123, 124, 125]
}
```

**Respuesta**:
```json
true
```

### 5. Métodos Personalizados (action methods)

**Endpoint**: `/json/2/{model}/{custom_method}`

**Ejemplo - Confirmar orden de venta**:
```http
POST /json/2/sale.order/action_confirm HTTP/1.1
Host: your-instance.odoo.com
Authorization: bearer YOUR_API_KEY_HERE
X-Odoo-Database: your_database
Content-Type: application/json

{
  "args": [[5]],
  "kwargs": {}
}
```

**Ejemplo - name_get**:
```http
POST /json/2/res.partner/name_get HTTP/1.1
Host: your-instance.odoo.com
Authorization: bearer YOUR_API_KEY_HERE
X-Odoo-Database: your_database
Content-Type: application/json

{
  "args": [[1, 2, 3]],
  "kwargs": {}
}
```

**Respuesta**:
```json
[
  [1, "Company A"],
  [2, "Company B"],
  [3, "Company C"]
]
```

## Diferencias con JSON-RPC

### JSON-RPC (Legacy - Odoo 8-18)

```http
POST /jsonrpc HTTP/1.1
Content-Type: application/json
Cookie: session_id=...

{
  "jsonrpc": "2.0",
  "method": "call",
  "params": {
    "service": "object",
    "method": "execute_kw",
    "args": [
      "database",
      uid,
      "api_key",
      "res.partner",
      "search_read",
      [domain],
      {"fields": [...], "limit": 10}
    ]
  },
  "id": null
}
```

### JSON2 (Modern - Odoo 19+)

```http
POST /json/2/res.partner/search_read HTTP/1.1
Content-Type: application/json
Authorization: bearer API_KEY
X-Odoo-Database: database

{
  "domain": [...],
  "fields": [...],
  "limit": 10
}
```

## Tabla Comparativa

| Aspecto | JSON-RPC | JSON2 |
|---------|----------|-------|
| Endpoint | `/jsonrpc` (único) | `/json/2/{model}/{method}` |
| Autenticación | Cookie/Session | Bearer Token |
| DB en | Body (params) | Header (X-Odoo-Database) |
| Modelo en | Body (args) | URL |
| Método en | Body (args) | URL |
| Wrapper | service/execute_kw | Directo |
| Respuesta | `{"result": [...]}` | `[...]` directo |
| Estado | Session (stateful) | Stateless |

## Estructura de Body según Método

### CRUD Operations

```json
// CREATE
{
  "field1": "value1",
  "field2": "value2"
}

// READ (search_read)
{
  "domain": [["field", "op", "value"]],
  "fields": ["field1", "field2"],
  "limit": 10,
  "offset": 0
}

// UPDATE (write)
{
  "ids": [1, 2, 3],
  "values": {
    "field1": "new_value"
  }
}

// DELETE (unlink)
{
  "ids": [1, 2, 3]
}
```

### Custom Methods

```json
{
  "args": [arg1, arg2, ...],  // Posicional
  "kwargs": {                  // Keyword arguments
    "key1": "value1",
    "key2": "value2"
  }
}
```

**Ejemplo args para métodos**:
```json
// Para button_immediate_install con IDs de módulos
{
  "args": [[123, 124]],  // Lista de IDs
  "kwargs": {}
}

// Para name_get
{
  "args": [[1, 2, 3]],  // IDs de registros
  "kwargs": {}
}
```

## Respuestas JSON2

### Respuesta Exitosa

**search_read**:
```json
[
  {"id": 1, "name": "Record 1"},
  {"id": 2, "name": "Record 2"}
]
```

**create**:
```json
{"id": 123}
```

**write/unlink**:
```json
true
```

**Custom methods**:
```json
// Depende del método
// name_get:
[[1, "Name 1"], [2, "Name 2"]]

// action_confirm:
true
```

### Respuesta con Error

```json
{
  "error": {
    "code": 200,
    "message": "Odoo Server Error",
    "data": {
      "name": "odoo.exceptions.AccessError",
      "message": "Access Denied",
      "debug": "Traceback...",
      "arguments": ["Access Denied"]
    }
  }
}
```

## Curl Examples

### Autenticación (verificar API key)
```bash
curl -X POST "https://your-instance.odoo.com/json/2/res.users/search_read" \
  -H "Content-Type: application/json" \
  -H "Authorization: bearer YOUR_API_KEY" \
  -H "X-Odoo-Database: your_database" \
  -d '{
    "domain": [],
    "fields": ["id", "name", "login"],
    "limit": 1
  }'
```

### Buscar empresas
```bash
curl -X POST "https://your-instance.odoo.com/json/2/res.partner/search_read" \
  -H "Content-Type: application/json" \
  -H "Authorization: bearer YOUR_API_KEY" \
  -H "X-Odoo-Database: your_database" \
  -d '{
    "domain": [["is_company", "=", true]],
    "fields": ["name", "email"],
    "limit": 10
  }'
```

### Crear producto
```bash
curl -X POST "https://your-instance.odoo.com/json/2/product.product/create" \
  -H "Content-Type: application/json" \
  -H "Authorization: bearer YOUR_API_KEY" \
  -H "X-Odoo-Database: your_database" \
  -d '{
    "name": "New Product",
    "list_price": 99.99,
    "type": "consu"
  }'
```

### Actualizar registros
```bash
curl -X POST "https://your-instance.odoo.com/json/2/res.partner/write" \
  -H "Content-Type: application/json" \
  -H "Authorization: bearer YOUR_API_KEY" \
  -H "X-Odoo-Database: your_database" \
  -d '{
    "ids": [123],
    "values": {
      "phone": "+34912345678"
    }
  }'
```

### Instalar módulo
```bash
# Paso 1: Buscar módulo
MODULE_ID=$(curl -s -X POST "https://your-instance.odoo.com/json/2/ir.module.module/search_read" \
  -H "Content-Type: application/json" \
  -H "Authorization: bearer YOUR_API_KEY" \
  -H "X-Odoo-Database: your_database" \
  -d '{
    "domain": [["name", "=", "sale_management"]],
    "fields": ["id"],
    "limit": 1
  }' | jq '.[0].id')

# Paso 2: Instalar
curl -X POST "https://your-instance.odoo.com/json/2/ir.module.module/button_immediate_install" \
  -H "Content-Type: application/json" \
  -H "Authorization: bearer YOUR_API_KEY" \
  -H "X-Odoo-Database: your_database" \
  -d "{
    \"args\": [[${MODULE_ID}]],
    \"kwargs\": {}
  }"
```

## Notas Importantes

1. **Sin Wrapper JSON-RPC**: JSON2 no usa el wrapper `jsonrpc`, `method`, `params`, `id`
2. **Modelo y Método en URL**: No en el body
3. **Header X-Odoo-Database**: Base de datos en header, no en body
4. **Respuesta Directa**: Sin wrapper `{"result": ...}`, devuelve datos directamente
5. **Bearer Token**: API key en Authorization header, no en el body
6. **Stateless**: Sin gestión de sesiones/cookies
7. **Content-Type**: Debe incluir `application/json`

## Referencias

- [Documentación Oficial Odoo 19.0 JSON2](https://www.odoo.com/documentation/19.0/developer/reference/external_api.html)
- [Migration Guide JSON-RPC to JSON2](https://www.odoo.com/documentation/19.0/developer/reference/external_api.html#migrating-from-xml-rpc-json-rpc)
