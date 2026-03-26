# Odoo Pilot JSON2 - Ejemplos Prácticos

## Introducción

Este documento contiene ejemplos prácticos de uso de los scripts de Odoo Pilot con el protocolo JSON2 (Odoo 19.0+) usando autenticación Bearer token.

## Configuración Inicial

### Paso 1: Exportar Variables de Entorno

```bash
# URL de tu instancia Odoo (sin barra final)
export ODOO_URL="https://your-instance.odoo.com"

# Nombre de la base de datos
export ODOO_DB="your_database"

# API Key generada desde Odoo
# (Preferencias → Seguridad de la Cuenta → Claves API)
export ODOO_KEY="YOUR_API_KEY_HERE"
```

### Paso 2: Autenticación

```bash
# Autenticarse con JSON2
cd /path/to/odoo-pilot
eval $(./scripts/auth_json2.sh)
```

**Salida esperada**:
```
✓ Authenticated successfully (JSON2 with Bearer token)
User ID: 1100
Username: David Gómez
Login: david.gomez@your-instance.odoo.com
Odoo Version: 19.0+e
Database: master
Note: Using API key authentication (stateless)
```

## Ejemplos de CRUD Operations

### 1. Buscar Registros (search_read)

#### Ejemplo 1.1: Listar todas las empresas

```bash
./scripts/search_records.sh res.partner \
  '[["is_company","=",true]]' \
  '["name","email","phone","vat"]' \
  10
```

**Explicación**:
- Modelo: `res.partner`
- Dominio: `[["is_company","=",true]]` (solo empresas)
- Campos: `["name","email","phone","vat"]`
- Límite: 10 registros

#### Ejemplo 1.2: Buscar módulos instalados de tipo Application

```bash
./scripts/search_records.sh ir.module.module \
  '[["state","=","installed"],["application","=",true]]' \
  '["name","shortdesc","installed_version","author"]' \
  50
```

**Explicación**:
- Dominio con múltiples condiciones (AND implícito)
- Busca módulos que estén instalados Y sean applications
- Devuelve hasta 50 resultados

#### Ejemplo 1.3: Buscar productos con precio menor a 100€

```bash
./scripts/search_records.sh product.product \
  '[["list_price","<",100],["sale_ok","=",true]]' \
  '["name","list_price","default_code","qty_available"]' \
  20
```

#### Ejemplo 1.4: Buscar con operador OR

```bash
# Buscar partners que sean empresa O tengan un email específico
./scripts/search_records.sh res.partner \
  '["|",["is_company","=",true],["email","=","info@example.com"]]' \
  '["name","email","is_company"]' \
  10
```

**Nota**: El operador `|` es prefijo y afecta a las dos condiciones siguientes.

### 2. Crear Registros (create)

#### Ejemplo 2.1: Crear un nuevo partner (contacto)

```bash
./scripts/create_record.sh res.partner '{
  "name": "Empresa Test S.L.",
  "email": "contacto@empresatest.es",
  "phone": "+34 912 345 678",
  "is_company": true,
  "vat": "ESB12345678",
  "street": "Calle Ejemplo 123",
  "city": "Madrid",
  "zip": "28001",
  "country_id": 69
}'
```

**Salida**:
```
✓ Record created successfully
Record ID: 1234
```

#### Ejemplo 2.2: Crear un producto

```bash
./scripts/create_record.sh product.product '{
  "name": "Producto Nuevo",
  "type": "consu",
  "list_price": 99.99,
  "default_code": "PROD001",
  "sale_ok": true,
  "purchase_ok": true,
  "description": "Descripción del producto"
}'
```

#### Ejemplo 2.3: Crear una tarea de proyecto

```bash
./scripts/create_record.sh project.task '{
  "name": "Nueva tarea de prueba",
  "project_id": 5,
  "user_ids": [[6, 0, [1100]]],
  "priority": "1",
  "description": "Esta es una tarea de prueba creada via API"
}'
```

**Nota sobre relaciones**:
- `[[6, 0, [1100]]]` → Reemplazar lista con IDs [1100]
- `[[4, 1100]]` → Agregar ID 1100 a la lista
- `[[3, 1100]]` → Quitar ID 1100 de la lista

### 3. Actualizar Registros (write)

#### Ejemplo 3.1: Actualizar un partner

```bash
# Actualizar partner ID 1234
./scripts/update_record.sh res.partner '[1234]' '{
  "phone": "+34 912 999 888",
  "mobile": "+34 600 111 222",
  "email": "nuevo@email.es"
}'
```

#### Ejemplo 3.2: Actualizar múltiples productos

```bash
# Actualizar precio de productos con IDs 1, 2, 3
./scripts/update_record.sh product.product '[1,2,3]' '{
  "list_price": 150.00
}'
```

#### Ejemplo 3.3: Archivar un registro

```bash
# Archivar partner ID 1234
./scripts/update_record.sh res.partner '[1234]' '{
  "active": false
}'
```

### 4. Eliminar Registros (unlink)

⚠️ **ADVERTENCIA**: La eliminación es permanente. Úsalo con precaución.

#### Ejemplo 4.1: Eliminar un registro

```bash
# Eliminar partner ID 1234
./scripts/delete_record.sh res.partner '[1234]'
```

#### Ejemplo 4.2: Eliminar múltiples registros

```bash
# Eliminar productos con IDs 100, 101, 102
./scripts/delete_record.sh product.product '[100,101,102]'
```

**Recomendación**: En lugar de eliminar, considera archivar (establecer `active: false`).

### 5. Ejecutar Métodos Personalizados (execute)

#### Ejemplo 5.1: Obtener nombre de registros (name_get)

```bash
./scripts/execute_method.sh res.partner name_get \
  '[[1,2,3]]' \
  '{}'
```

**Salida**:
```json
[
  [1, "Empresa ABC"],
  [2, "Juan Pérez"],
  [3, "María García"]
]
```

#### Ejemplo 5.2: Confirmar una orden de venta

```bash
./scripts/execute_method.sh sale.order action_confirm \
  '[[5]]' \
  '{}'
```

**Explicación**:
- Método: `action_confirm`
- Args: `[[5]]` → IDs de órdenes a confirmar
- Kwargs: `{}` → Sin parámetros adicionales

#### Ejemplo 5.3: Obtener parámetro del sistema

```bash
./scripts/execute_method.sh ir.config_parameter get_param \
  '["web.base.url"]' \
  '{}'
```

## Ejemplos de Gestión de Módulos

### 6. Instalar Módulos

#### Ejemplo 6.1: Instalar módulo de ventas

```bash
./scripts/install_module.sh sale_management
```

**Salida**:
```
Installing module: sale_management
✓ Module installed successfully
```

#### Ejemplo 6.2: Instalar módulo de contabilidad

```bash
./scripts/install_module.sh account
```

### 7. Actualizar Módulos

```bash
./scripts/upgrade_module.sh sale_management
```

### 8. Desinstalar Módulos

⚠️ **CUIDADO**: Desinstalar módulos puede eliminar datos.

```bash
./scripts/uninstall_module.sh sale_management
```

## Workflows Comunes

### Workflow 1: Buscar, Actualizar y Verificar

```bash
# 1. Buscar partners con email específico
RESULT=$(./scripts/search_records.sh res.partner \
  '[["email","=","old@email.com"]]' \
  '["id","name","email"]' \
  1)

# 2. Extraer ID
PARTNER_ID=$(echo "$RESULT" | jq '.[0].id')

# 3. Actualizar email
./scripts/update_record.sh res.partner "[${PARTNER_ID}]" '{
  "email": "new@email.com"
}'

# 4. Verificar cambio
./scripts/search_records.sh res.partner \
  "[[\"id\",\"=\",${PARTNER_ID}]]" \
  '["id","name","email"]' \
  1
```

### Workflow 2: Crear Partner con Contactos

```bash
# 1. Crear empresa
COMPANY_ID=$(./scripts/create_record.sh res.partner '{
  "name": "Mi Empresa S.L.",
  "is_company": true,
  "email": "info@miempresa.es"
}')

echo "Empresa creada con ID: $COMPANY_ID"

# 2. Crear contacto asociado a la empresa
CONTACT_ID=$(./scripts/create_record.sh res.partner "{
  \"name\": \"Juan Manager\",
  \"email\": \"juan@miempresa.es\",
  \"parent_id\": ${COMPANY_ID},
  \"type\": \"contact\"
}")

echo "Contacto creado con ID: $CONTACT_ID"
```

### Workflow 3: Backup de Datos

```bash
#!/bin/bash
# Script para hacer backup de partners

BACKUP_DIR="./backup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

# Exportar todos los partners
./scripts/search_records.sh res.partner \
  '[]' \
  '["name","email","phone","vat","street","city","zip","country_id"]' \
  0 > "$BACKUP_DIR/partners.json"

echo "Backup guardado en: $BACKUP_DIR/partners.json"
```

### Workflow 4: Migración de Datos

```bash
#!/bin/bash
# Migrar contactos de una instancia a otra

SOURCE_URL="https://source.odoo.com"
TARGET_URL="https://target.odoo.com"
SOURCE_KEY="source-api-key"
TARGET_KEY="target-api-key"

# 1. Exportar de origen
export ODOO_URL="$SOURCE_URL"
export ODOO_KEY="$SOURCE_KEY"
export ODOO_DB="source_db"
eval $(./scripts/auth_json2.sh)

./scripts/search_records.sh res.partner \
  '[["is_company","=",true]]' \
  '["name","email","phone","vat"]' \
  0 > partners_export.json

# 2. Importar a destino
export ODOO_URL="$TARGET_URL"
export ODOO_KEY="$TARGET_KEY"
export ODOO_DB="target_db"
eval $(./scripts/auth_json2.sh)

# (Aquí iría lógica para procesar JSON e insertar)
```

## Ejemplos Avanzados

### Ejemplo Avanzado 1: Búsqueda con Paginación

```bash
#!/bin/bash
# Obtener todos los productos en páginas de 100

LIMIT=100
OFFSET=0
ALL_RESULTS=""

while true; do
  RESULTS=$(./scripts/search_records.sh product.product \
    '[]' \
    '["name","list_price"]' \
    $LIMIT \
    $OFFSET)

  COUNT=$(echo "$RESULTS" | jq 'length')

  if [ "$COUNT" -eq 0 ]; then
    break
  fi

  ALL_RESULTS="$ALL_RESULTS $RESULTS"
  OFFSET=$((OFFSET + LIMIT))

  echo "Obtenidos $OFFSET registros..."
done

echo "$ALL_RESULTS" > all_products.json
```

### Ejemplo Avanzado 2: Procesamiento en Lote

```bash
#!/bin/bash
# Actualizar precios de productos en lote

# Lista de IDs de productos
PRODUCT_IDS=(1 2 3 4 5 6 7 8 9 10)

# Procesar en lotes de 5
BATCH_SIZE=5
for ((i=0; i<${#PRODUCT_IDS[@]}; i+=BATCH_SIZE)); do
  BATCH=("${PRODUCT_IDS[@]:i:BATCH_SIZE}")
  IDS=$(IFS=,; echo "[${BATCH[*]}]")

  echo "Actualizando lote: $IDS"

  ./scripts/update_record.sh product.product "$IDS" '{
    "list_price": 99.99
  }'

  # Delay para no saturar el servidor
  sleep 1
done
```

### Ejemplo Avanzado 3: Monitoreo de Cambios

```bash
#!/bin/bash
# Monitorear cambios en módulos instalados

PREVIOUS_STATE="previous_modules.json"
CURRENT_STATE="current_modules.json"

# Guardar estado actual
./scripts/search_records.sh ir.module.module \
  '[["state","=","installed"]]' \
  '["name","installed_version"]' \
  0 > "$CURRENT_STATE"

# Comparar con anterior (si existe)
if [ -f "$PREVIOUS_STATE" ]; then
  diff <(jq -S . "$PREVIOUS_STATE") <(jq -S . "$CURRENT_STATE") \
    > module_changes.diff

  if [ -s module_changes.diff ]; then
    echo "⚠️  Cambios detectados en módulos:"
    cat module_changes.diff
  else
    echo "✓ No hay cambios en módulos"
  fi
fi

# Actualizar estado anterior
cp "$CURRENT_STATE" "$PREVIOUS_STATE"
```

## Dominios Odoo (Sintaxis Avanzada)

### Operadores Básicos

```bash
# Igual
'[["field","=","value"]]'

# No igual
'[["field","!=","value"]]'

# Mayor que
'[["field",">",100]]'

# Menor que
'[["field","<",100]]'

# Mayor o igual
'[["field",">=",100]]'

# Menor o igual
'[["field","<=",100]]'

# Contiene (like)
'[["field","like","text"]]'

# En lista
'[["field","in",[1,2,3]]]'

# No en lista
'[["field","not in",[1,2,3]]]'
```

### Operadores Lógicos

```bash
# AND (implícito)
'[["field1","=","a"],["field2","=","b"]]'

# OR
'["|",["field1","=","a"],["field2","=","b"]]'

# NOT
'["!",["field","=","value"]]'

# Complejo: (A OR B) AND C
'["&","|",["a","=",1],["b","=",2],["c","=",3]]'
```

## Solución de Problemas Comunes

### Error: "JSON2 endpoint not available"

```bash
# Verificar si el endpoint está disponible
curl -I "https://your-instance.odoo.com/json/2/call"

# Si devuelve 404, usar JSON-RPC en su lugar
eval $(./scripts/auth_jsonrpc.sh)
```

### Error: "Authentication failed"

```bash
# Verificar API key
echo $ODOO_KEY

# Generar nueva API key desde Odoo:
# Preferencias → Seguridad de la Cuenta → Nueva Clave API

# Re-autenticar
eval $(./scripts/auth_json2.sh)
```

### Error: "Permission denied"

```bash
# Verificar permisos del usuario
./scripts/search_records.sh res.users \
  "[['id','=',$ODOO_UID]]" \
  '["name","groups_id"]' \
  1
```

## Referencias Rápidas

### Modelos Comunes

| Modelo | Descripción |
|--------|-------------|
| `res.partner` | Contactos/Empresas |
| `res.users` | Usuarios |
| `product.product` | Productos |
| `sale.order` | Órdenes de venta |
| `purchase.order` | Órdenes de compra |
| `account.move` | Facturas/Asientos |
| `project.task` | Tareas de proyecto |
| `hr.employee` | Empleados |
| `ir.module.module` | Módulos |

### Variables de Entorno

```bash
ODOO_URL      # URL de la instancia
ODOO_DB       # Nombre de la base de datos
ODOO_KEY      # API Key
ODOO_UID      # User ID (set por auth)
ODOO_VERSION  # Versión de Odoo (set por auth)
ODOO_PROTOCOL # "json2" o "jsonrpc" (set por auth)
```

## Conclusión

Estos ejemplos cubren los casos de uso más comunes. Para más información, consulta:

- [PROTOCOL_GUIDE.md](PROTOCOL_GUIDE.md) - Guía completa de protocolos
- [CHANGELOG_JSON2.md](CHANGELOG_JSON2.md) - Detalles de implementación
- [Odoo 19.0 API Docs](https://www.odoo.com/documentation/19.0/developer/reference/external_api.html)
