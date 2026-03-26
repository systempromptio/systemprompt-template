# Metricas CRM - Algoritmos y Formulas

Documento de referencia con todos los algoritmos de calculo del CRM Health Score,
scoring por comercial, analisis de demos y prevision mensual.

---

## 1. Datos de Entrada

### 1.1 Modelos Odoo

| Modelo | Contenido | Clave |
|--------|-----------|-------|
| `crm.lead` (activas) | Oportunidades abiertas (type=opportunity, active=True) | Pipeline actual |
| `crm.lead` (ganadas) | Oportunidades ganadas ultimos 12 meses (stage_id=won) | Win rate |
| `crm.lead` (perdidas) | Oportunidades perdidas ultimos 12 meses (active=False) | Win rate |
| `crm.lead` (todas 12m) | TODAS las opp ultimos 12 meses (activas + archivadas) | Win rate total |
| `crm.stage` | Etapas del pipeline CRM | Clasificacion de avance |
| `res.users` | Comerciales (user_id de oportunidades) | Scoring individual |

### 1.2 Query de Oportunidades Totales (12m)

Para obtener TODAS las oportunidades (activas e inactivas) se usa el filtro:

```
["|", ["active", "=", True], ["active", "!=", True], ["type", "=", "opportunity"], ["company_id", "=", 1], ["create_date", ">=", "FECHA_12M_ATRAS"]]
```

El operador `|` en notacion polaca aplica OR a las 2 siguientes condiciones,
asegurando que se obtienen tanto las activas como las archivadas.

### 1.3 Campos Clave de crm.lead

| Campo | Uso |
|-------|-----|
| `user_id` | Comercial asignado |
| `team_id` | Equipo de ventas |
| `stage_id` | Etapa actual en el pipeline |
| `probability` | Probabilidad de cierre (0-100) |
| `expected_revenue` | Ingreso esperado |
| `date_deadline` | Fecha prevista de cierre |
| `create_date` | Fecha de creacion |
| `write_date` | Ultima actualizacion |
| `date_last_stage_update` | Ultimo cambio de etapa |
| `date_closed` | Fecha de cierre (ganadas) |
| `priority` | Prioridad (0=normal, 1=alta, 2=muy alta) |
| `x_studio_fecha_demo` | Fecha de demo realizada/programada (campo custom) |

---

## 2. CRM Health Score Global (0-100)

### 2.1 Formula Principal

```
S = clamp(100 - Pen_rendimiento - Pen_gobernanza, 0, 100)
```

Donde:
- `Pen_rendimiento` = suma de penalizaciones de rendimiento
- `Pen_gobernanza` = suma de penalizaciones de gobernanza

### 2.2 Penalizaciones de Rendimiento

| # | Penalizacion | Formula | Max | Cuando |
|---|-------------|---------|-----|--------|
| 1 | Win rate bajo | `max(0, (30 - win_rate)) * 0.5` | 15 | win_rate < 30% |
| 2 | Pipeline estancado | `stale_pct * 0.3` | 20 | Opps sin actividad segun umbral de etapa |
| 3 | Opp vencidas | `overdue_pct * 0.4` | 20 | Opps pasadas de date_deadline |
| 4 | Pipeline bajo | `max(0, (100 - coverage_pct)) * 0.15` | 15 | Solo si hay objetivo mensual |

> **NOTA**: El demo rate NO penaliza. Es informativo pero no obligatorio hacer demo.

**Calculos intermedios**:

```
win_rate = ganadas / total_oportunidades_12m * 100
  → total_oportunidades_12m = TODAS las opp (activas + archivadas) creadas en ultimos 12 meses
  → Si total_oportunidades_12m < 5: win_rate = 50 (dato insuficiente, sin penalizacion)

stale_pct = opp_estancadas / total_opp_pipeline * 100
  → "estancada" = sin actividad segun umbral por etapa:
    - Pte. Asignar consultor: 10 dias
    - Demo propuesta: 10 dias
    - Demo realizada: 10 dias
    - Upselling: 10 dias
    - Presupuesto creado: 10 dias
    - Propuesta enviada: 10 dias
    - En Espera: 20 dias
    - En analisis: 30 dias
    - Otras etapas: stale_days (default 21)

overdue_pct = opp_vencidas / total_opp_pipeline * 100
  → "vencida" = date_deadline < hoy Y opp sigue abierta

coverage_pct = forecast_ponderado / objetivo_mensual * 100
  → Solo se calcula si el usuario proporciona objetivo_mensual
  → Si no hay objetivo: penalizacion = 0
```

### 2.3 Penalizaciones de Gobernanza

| # | Penalizacion | Formula | Max | Cuando |
|---|-------------|---------|-----|--------|
| 1 | Sin fecha cierre | `pct_sin_fecha * 0.15` | 10 | Opps sin date_deadline |
| 2 | Sin revenue | `pct_sin_revenue * 0.15` | 10 | expected_revenue = 0 o vacio |
| 3 | Sin comercial | `count_sin_user * 3` | 10 | Opps sin user_id |
| 4 | Sin actividad 30d | `pct_inactivas * 0.15` | 10 | write_date > 30 dias |

**Calculos intermedios**:

```
pct_sin_fecha = opp_sin_deadline / total_opp_pipeline * 100
pct_sin_revenue = opp_sin_revenue / total_opp_pipeline * 100
count_sin_user = count(opp donde user_id es False o null)
pct_inactivas = opp_inactivas_30d / total_opp_pipeline * 100
```

### 2.4 Clasificacion de Estado

| Score | Estado | Icono |
|-------|--------|-------|
| 80-100 | SALUDABLE | `🟢` |
| 60-79 | EN RIESGO | `🟡` |
| 40-59 | CRITICO | `🟠` |
| 0-39 | EMERGENCIA | `🔴` |

### 2.5 Barra Visual

```
30 caracteres para la barra. Caracteres llenos = round(S / 100 * 30).
Ejemplo (score=75): ██████████████████████▒▒▒▒▒▒▒▒ 75/100
```

---

## 3. Scoring por Comercial (0-100)

Cada comercial recibe un score individual basado en 5 penalizaciones.

### 3.1 Formula

```
Score_comercial = clamp(100 - sum(penalizaciones), 0, 100)
```

### 3.2 Penalizaciones Individuales

| # | Penalizacion | Que Mide | Formula | Max |
|---|-------------|----------|---------|-----|
| 1 | Pipeline | Valor pipeline vs media equipo | Ver 3.3 | 20 |
| 2 | Win rate | Tasa de ganancia vs media equipo | Ver 3.4 | 20 |
| 3 | Gobernanza | Campos incompletos | Ver 3.5 | 15 |
| 4 | Stale | % opp estancadas (por umbral de etapa) | Ver 3.6 | 15 |
| 5 | Actividad | % opp actualizadas | Ver 3.7 | 15 |

> **NOTA**: Demo rate NO penaliza. Se muestra como informativo pero no resta puntos.

### 3.3 Penalizacion Pipeline

```
pipeline_comercial = sum(expected_revenue) de opp activas del comercial
pipeline_media = sum(expected_revenue de todas las activas) / num_comerciales

Si pipeline_media > 0:
  ratio = pipeline_comercial / pipeline_media
  pen_pipeline = max(0, (1 - ratio)) * 20
Sino:
  pen_pipeline = 0
```

### 3.4 Penalizacion Win Rate

```
wr_comercial = ganadas_comercial / total_opp_12m_comercial * 100
wr_media = ganadas_total / total_opp_12m * 100

Si total_opp_12m_comercial < 3:
  pen_wr = 5  (dato insuficiente, penalizacion reducida)
Sino:
  pen_wr = min(20, max(0, (wr_media - wr_comercial)) * 0.5)
```

### 3.5 Penalizacion Gobernanza

```
opp_comercial = opp activas del comercial
sin_fecha = count(opp sin date_deadline) / total_opp_comercial * 100
sin_revenue = count(opp sin expected_revenue) / total_opp_comercial * 100

pen_gov = min(15, (sin_fecha + sin_revenue) * 0.15)
```

### 3.6 Penalizacion Stale

```
Para cada opp del comercial:
  umbral = stale_thresholds[stage_id] (10, 20 o 30 dias segun etapa)
  si ultima_actividad < hoy - umbral: es estancada

stale_pct = estancadas / total_opp_comercial * 100
pen_stale = min(15, stale_pct * 0.2)
```

### 3.7 Penalizacion Actividad

```
inactivas = count(opp del comercial con ultima_actividad < hoy - 30d)
inact_pct = inactivas / total_opp_comercial * 100

pen_actividad = min(15, inact_pct * 0.2)
```

---

## 4. Analisis de Demos

### 4.1 Metricas de Demos

| Metrica | Formula |
|---------|---------|
| Demos este mes | `count(opp donde fecha_demo en mes actual)` |
| Demos por comercial | Agrupar por user_id, contar por mes |
| Tasa de demo | `opp_con_demo / opp_calificadas * 100` |
| Conversion con demo | `ganadas_con_demo / total_opp_12m_con_demo * 100` |
| Conversion sin demo | `ganadas_sin_demo / total_opp_12m_sin_demo * 100` |
| Dias demo→cierre | `avg(date_closed - x_studio_fecha_demo)` para ganadas con demo |

> **NOTA**: La conversion se calcula usando el total de oportunidades de 12 meses
> (misma base que el win rate), separando las que tuvieron demo de las que no.

### 4.2 Determinacion de "Calificada"

```
etapas = crm.stage ordenadas por sequence ASC
min_sequence = etapas[0].sequence
calificada = opp donde stage.sequence > min_sequence
```

No se hardcodea ningun nombre de etapa. La deteccion es dinamica basada en sequence.

### 4.3 Campo No Existe

Si `x_studio_fecha_demo` no existe en los datos:
- Skip completo del analisis de demos
- Penalizaciones de demo = 0 (global y por comercial)
- Aviso en el informe: "Campo x_studio_fecha_demo no encontrado. Analisis de demos omitido."

---

## 5. Prevision Mensual

### 5.1 Oportunidades del Mes

```
opp_mes = opp activas donde date_deadline esta en el mes actual
```

### 5.2 Forecast Ponderado

```
forecast = sum(expected_revenue * probability / 100) para opp_mes
```

### 5.3 Cobertura

```
Si objetivo_mensual > 0:
  cobertura = forecast / objetivo_mensual * 100
Sino:
  cobertura = N/A (no se puede calcular)
```

### 5.4 Tabla de Oportunidades del Mes

Para cada opp_mes, mostrar TODAS (sin limite):
- Nombre de la opp
- Comercial (user_id)
- Revenue esperado
- Probabilidad
- Revenue ponderado (revenue * prob / 100)
- Etapa actual

---

## 6. Problemas de Gobernanza

5 tipos detectados automaticamente:

### 6.1 Sin Fecha de Cierre

```
opp donde date_deadline es False/null
Impacto: No aparece en previsiones mensuales
```

### 6.2 Sin Revenue Esperado

```
opp donde expected_revenue = 0 o es null/False
Impacto: Pipeline sin visibilidad economica
```

### 6.3 Sin Comercial Asignado

```
opp donde user_id es False/null
Impacto: Sin responsable
```

### 6.4 Vencidas y Abiertas

```
opp donde date_deadline < hoy Y sigue activa
Impacto: Pipeline inflado artificialmente
```

### 6.5 Sin Actividad 30+ Dias

```
opp donde ultima_actividad < (hoy - 30 dias)
Impacto: Pipeline muerto, oportunidades fantasma
```

---

## 7. Criterios de Estancamiento por Etapa

Los umbrales de estancamiento varian segun la etapa del pipeline:

| Etapa | Umbral | Justificacion |
|-------|:------:|---------------|
| Pte. Asignar consultor | 10 dias | Debe asignarse rapidamente |
| Demo propuesta | 10 dias | Debe concretarse la fecha |
| Demo realizada | 10 dias | Debe avanzar a presupuesto |
| Upselling | 10 dias | Seguimiento activo necesario |
| Presupuesto creado | 10 dias | Debe enviarse al cliente |
| Propuesta enviada | 10 dias | Debe seguirse para respuesta |
| En Espera | 20 dias | Periodo razonable de espera |
| En analisis | 30 dias | Analisis mas profundo permitido |

---

## 8. Dashboard Extra - Indicadores Adicionales

### 8.1 Pedidos Cerrados

```
Mes actual: opp ganadas con date_closed en mes actual
  → count + sum(expected_revenue)
  → Desglose por comercial

Mes anterior: opp ganadas con date_closed en mes anterior
  → count + sum(expected_revenue)
  → Desglose por comercial
```

### 8.2 Oportunidades Nuevas

```
Mes actual: count(opp en all_12m con create_date en mes actual)
Mes anterior: count(opp en all_12m con create_date en mes anterior)
```

### 8.3 Oportunidades Asignadas por Comercial

```
Por cada comercial:
  Mes actual: count(opp en all_12m con create_date en mes actual Y user_id = comercial)
  Mes anterior: count(opp en all_12m con create_date en mes anterior Y user_id = comercial)
```

---

## 9. Casos Especiales

### 9.1 Sin Oportunidades Activas

```
Si total_opp_activas = 0:
  score = 0
  estado = "EMERGENCIA"
  escenario = "LOW_PIPELINE"
  aviso: "No hay oportunidades activas en el pipeline."
```

### 9.2 Sin Datos de Oportunidades Totales

```
Si total_opp_12m < 5:
  win_rate = 50 (neutro)
  pen_win_rate = 0
  aviso: "Datos insuficientes para calcular win rate (< 5 opp en 12 meses)."
```

### 9.3 Campo x_studio_fecha_demo No Existe

```
Si el campo no aparece en los datos:
  Todas las metricas de demo = 0 o N/A
  Penalizaciones de demo = 0
  Seccion 4 del informe indica que el campo no existe
```

### 9.4 Sin Objetivo Mensual

```
Si no se proporciona objetivo_mensual:
  cobertura = N/A
  pen_pipeline_bajo = 0
  Seccion 3 omite indicador de cobertura
```
