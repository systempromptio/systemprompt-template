---
name: "CRM Health Monitor"
description: "Evaluate CRM health analyzing pipeline, demos, win rate, and salesperson performance. CRM Health Score 0-100"
---

### Paso 1: Conexion y Extraccion de Datos

EJECUTAR el script `scripts/fetch_crm_data.sh`:

```bash
# Configurar variables de entorno
export ODOO_URL="<url_proporcionada>"
export ODOO_DB="<db_proporcionada>"
export ODOO_KEY="<key_proporcionada>"
export ODOO_USER="<user_proporcionado>"

# Ejecutar extraccion
bash scripts/fetch_crm_data.sh \
  --pilot-dir /path/to/odoo-pilot/scripts \
  > crm_data.json
```

El script extrae:
1. Oportunidades activas (crm.lead, type=opportunity, active=True)
2. Oportunidades ganadas ultimos 12 meses
3. Oportunidades perdidas ultimos 12 meses (manejo de active=False)
4. TODAS las oportunidades ultimos 12 meses (activas + archivadas, para win rate)
5. Etapas del CRM (crm.stage)
6. Comerciales (res.users, IDs extraidos de las oportunidades)

**Verificar**:
- Que se obtuvieron oportunidades activas (si 0 → pipeline vacio, el score sera 0)
- Que se obtuvieron etapas (minimo 2 para calcular "calificada")
- Si las oportunidades perdidas no se pudieron obtener, el win rate sera parcial

---

### Paso 2: Calculo de Metricas (OBLIGATORIO)

EJECUTAR el script Python `scripts/calculate_crm_health.py`:

```bash
# Sin objetivo mensual
python3 scripts/calculate_crm_health.py --input crm_data.json > crm_health.json

# Con objetivo mensual
python3 scripts/calculate_crm_health.py --input crm_data.json --target 50000 > crm_health.json

# Con umbral personalizado de estancamiento
python3 scripts/calculate_crm_health.py --input crm_data.json --target 50000 --stale-days 14 > crm_health.json
```

> **IMPORTANTE**: Este paso es OBLIGATORIO. NUNCA calcules las metricas manualmente.
> El script aplica todas las formulas documentadas en `references/metricas_crm.md`.

**El script calcula**:
- CRM Health Score global (0-100) con penalizaciones
- Scoring individual por comercial (0-100)
- Analisis de demos (si el campo existe)
- Prevision mensual con forecast ponderado
- 5 tipos de problemas de gobernanza
- Escenario de diagnostico

LEER el JSON generado (`crm_health.json`) para continuar.

---

### Paso 2.5: Validacion del Dashboard

PRESENTAR al usuario un resumen del dashboard con los datos calculados:

```
CRM Health Score: {icono} {estado} {score}/100

Pipeline: {total_active} oportunidades | {total_revenue} EUR
Win rate (12m): {win_rate}% ({total_won} ganadas / {total_all_12m} totales)
Estancadas: {stale_count} ({stale_pct}%) - umbrales por etapa: 10d/20d/30d
Vencidas: {overdue_count} ({overdue_pct}%)
Demos: {demo_rate}% de calificadas con demo (informativo, no penaliza)

Pedidos cerrados mes actual: {count} pedidos | {revenue} EUR
Pedidos cerrados mes anterior: {count} pedidos | {revenue} EUR
Opp nuevas este mes: {count}

Escenario: {scenario_code} - {scenario_desc}

Penalizaciones:
  Rendimiento: -{pen_rendimiento} (win rate: {p1}, stale: {p2}, overdue: {p3}, pipeline: {p4})
  Gobernanza: -{pen_gobernanza} (fecha: {g1}, revenue: {g2}, comercial: {g3}, actividad: {g4})

Scoring comerciales:
  {comercial_1}: {score_1}/100 ({estado_1})
  {comercial_2}: {score_2}/100 ({estado_2})
  ...
```

> **DETENTE** aqui y espera la aprobacion del usuario antes de generar el informe.
> El usuario puede pedir ajustes (ej: cambiar objetivo mensual, revisar datos).

---

### Paso 3: Evaluacion y Plan de Accion

LEER `references/patrones_diagnostico_crm.md` para:

1. **Identificar el escenario principal** (ya calculado en el JSON):
   - HEALTHY_PIPELINE, LOW_PIPELINE, STALE_PIPELINE,
     LOW_WIN_RATE, GOVERNANCE_ISSUES, UNBALANCED_TEAM

2. **Seleccionar acciones** del escenario correspondiente:
   - Acciones inmediatas (esta semana)
   - Acciones a corto plazo (proximas 2 semanas)
   - Acciones estrategicas

3. **Personalizar acciones** con datos reales:
   - Usar nombres de oportunidades/comerciales reales del JSON
   - Adaptar las recomendaciones al contexto especifico
   - Priorizar las acciones segun las penalizaciones mas altas

---

### Paso 4: Generacion del Informe

> **SOLO** generar el informe tras la aprobacion del Paso 2.5.

LEER `templates/informe_crm.md` y generar el informe completo:

1. **Seccion 1 - Dashboard CRM**: Score, indicadores clave, pedidos cerrados (mes actual/anterior), opp nuevas, opp asignadas por comercial, escenario
2. **Seccion 2 - Vision del Pipeline**: Tablas por etapa y por comercial (pipeline ponderado, columna cierre este mes)
3. **Seccion 3 - Prevision del Mes**: TODAS las oportunidades que cierran este mes, forecast, cobertura
4. **Seccion 4 - Analisis de Demos**: Metricas de demo, por comercial (sin tabla de calificadas sin demo)
5. **Seccion 5 - Scoring por Comercial**: Tabla resumen con num opp, pipeline ponderado, scores individuales
6. **Seccion 6 - Detalle por Comercial**: Bloque detallado con opp vencidas (total + mes anterior)
7. **Seccion 7 - Problemas de Gobernanza**: 5 tipos con listas detalladas
8. **Seccion 8 - Plan de Accion**: Acciones del escenario detectado (no considerar falta de demos)

**Reglas de generacion**:
- Si el campo `x_studio_fecha_demo` no existe: omitir seccion 4 con aviso
- Si no hay objetivo mensual: omitir filas de objetivo/cobertura en seccion 3
- Si hay menos de 3 comerciales: omitir analisis de balance de equipo
- Usar datos reales del JSON, nunca inventar nombres o cifras
- Revenue siempre en EUR con 2 decimales
- Porcentajes con 1 decimal

GUARDAR el informe como `informe_salud_crm_{fecha}.md`.

---

## Estructura de Archivos

```
crm-health-monitor/
├── SKILL.md                           # Este archivo
├── scripts/
│   ├── fetch_crm_data.sh             # Extraccion de datos via odoo-pilot
│   └── calculate_crm_health.py       # Motor de calculo (OBLIGATORIO)
├── references/
│   ├── metricas_crm.md               # Algoritmos y formulas
│   └── patrones_diagnostico_crm.md   # Escenarios y acciones
└── templates/
    └── informe_crm.md                # Template del informe
```

---

## Reglas Importantes

1. **NUNCA** calcules metricas manualmente. Siempre usa `calculate_crm_health.py`.
2. **NUNCA** generes el informe sin la aprobacion del dashboard (Paso 2.5).
3. **NUNCA** inventes datos. Todos los numeros vienen del JSON calculado.
4. Si el campo `x_studio_fecha_demo` no existe, penalizaciones de demo = 0.
5. El demo rate es INFORMATIVO, no penaliza (ni global ni por comercial).
6. Si no hay objetivo mensual, penalizacion de pipeline bajo = 0.
7. Si hay menos de 5 opp totales en 12 meses, win rate = 50% (neutro, sin penalizacion).
8. Las oportunidades perdidas en Odoo tienen `active=False`. Si no se pueden obtener, avisar.
9. Los problemas de gobernanza se detectan automaticamente (5 tipos, sin duplicadas ni sin demo).
10. El scoring por comercial es relativo al equipo (vs media), no absoluto.
11. El estancamiento usa umbrales por etapa: 10d (mayoría), 20d (En Espera), 30d (En análisis).
12. El informe esta dirigido al director comercial / responsable de ventas.

---

## Success Criteria

- [ ] Datos extraidos correctamente (opp activas, ganadas, perdidas, etapas, usuarios)
- [ ] `calculate_crm_health.py` ejecutado sin errores
- [ ] Dashboard presentado y aprobado por el usuario
- [ ] CRM Health Score calculado (0-100) con penalizaciones detalladas
- [ ] Scoring individual por comercial calculado
- [ ] Escenario de diagnostico identificado correctamente
- [ ] Plan de accion personalizado con datos reales
- [ ] Informe Markdown generado con las 8 secciones
- [ ] Fichero guardado como `informe_salud_crm_{fecha}.md`
