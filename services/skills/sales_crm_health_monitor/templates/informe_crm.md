# Template: Informe de Salud del CRM

> Usa este template para generar el informe final. Reemplaza los placeholders `{...}` con los datos calculados.

---

```markdown
# Informe de Salud del CRM

**Fecha del analisis**: {fecha_referencia}
**Generado por**: CRM Health Monitor - Enterprise Demo

---

## 1. Dashboard CRM

### CRM Health Score

{icono_estado} **{estado}** {barra_visual} **{score}/100**

| Indicador | Valor | Estado |
|-----------|-------|--------|
| Oportunidades activas | {total_active} | {icono_active} |
| Pipeline total | {total_pipeline_revenue} EUR | - |
| Win rate (12 meses) | {win_rate}% ({total_won} ganadas / {total_all_12m} totales) | {icono_wr} |
| Opp estancadas | {stale_count} ({stale_pct}%) | {icono_stale} |
| Opp vencidas | {overdue_count} ({overdue_pct}%) | {icono_overdue} |
| Tasa de demo | {demo_rate}% | {icono_demo} |
| Opp nuevas este mes | {new_opps_this_month} | - |

**Escenario detectado**: {escenario} - {descripcion_escenario}

{avisos_especiales}

### Pedidos Cerrados

| Periodo | Pedidos | Base Imponible |
|---------|:-------:|---------------:|
| **Mes actual** | {closed_this_month_count} | {closed_this_month_revenue} EUR |
| **Mes anterior** | {closed_prev_month_count} | {closed_prev_month_revenue} EUR |

#### Por Comercial

| Comercial | Pedidos Mes Actual | Importe Mes Actual | Pedidos Mes Anterior | Importe Mes Anterior |
|-----------|:-----------------:|-----------------:|:-------------------:|-------------------:|
| {comercial_1} | {count} | {revenue} EUR | {count_prev} | {revenue_prev} EUR |
| ... | ... | ... | ... | ... |

### Oportunidades Asignadas

| Comercial | Asignadas Mes Actual | Asignadas Mes Anterior |
|-----------|:-------------------:|:---------------------:|
| {comercial_1} | {assigned_this} | {assigned_prev} |
| ... | ... | ... |

---

## 2. Vision del Pipeline

### Por Etapa

| Etapa | Oportunidades | Revenue | Revenue Ponderado | Cierre Este Mes |
|-------|:------------:|--------:|------------------:|:--------------:|
| {etapa_1} | {count} | {revenue} EUR | {weighted} EUR | {closing} |
| {etapa_2} | ... | ... | ... | ... |
| **TOTAL** | **{total}** | **{total_revenue} EUR** | **{total_weighted} EUR** | **{total_closing}** |

### Por Comercial

| Comercial | Opp Activas | Pipeline Ponderado | Ganadas | Perdidas | Win Rate | Cierre Este Mes |
|-----------|:----------:|-----------------:|:------:|:-------:|--------:|:--------------:|
| {comercial_1} | {opp} | {pipeline_weighted} EUR | {won} | {lost} | {wr}% | {closing} |
| {comercial_2} | ... | ... | ... | ... | ... | ... |
| **TOTAL** | **{total}** | **{total_weighted} EUR** | **{total_won}** | **{total_lost}** | **{global_wr}%** | **{total_closing}** |

---

## 3. Prevision del Mes ({mes_actual})

| Indicador | Valor |
|-----------|-------|
| **Oportunidades que cierran este mes** | {opp_mes_count} |
| **Forecast ponderado** | {forecast_weighted} EUR |
| **Objetivo mensual** | {monthly_target} EUR |
| **Cobertura** | {coverage_pct}% |

> Si no hay objetivo mensual, omitir las filas de objetivo y cobertura.

### Detalle de Oportunidades que Cierran Este Mes

> Mostrar TODAS las oportunidades con fecha de cierre en el mes actual, sin limite.

| Oportunidad | Comercial | Revenue | Prob. | Revenue Pond. | Etapa |
|-------------|-----------|--------:|:----:|-------------:|-------|
| {opp_1} | {user} | {rev} EUR | {prob}% | {weighted} EUR | {stage} |
| ... | ... | ... | ... | ... | ... |
| **TOTAL** | | | | **{total_weighted} EUR** | |

---

## 4. Analisis de Demos

> Si el campo x_studio_fecha_demo no existe, mostrar:
> "Campo x_studio_fecha_demo no encontrado en el CRM. Seccion omitida."
> Y no generar el resto de la seccion.

| Indicador | Valor |
|-----------|-------|
| **Demos este mes** | {demos_this_month} |
| **Tasa de demo** | {demo_rate}% ({with_demo}/{total_qualified} calificadas) |
| **Conversion con demo** | {conversion_with_demo}% (ganadas con demo / total opp con demo 12m) |
| **Conversion sin demo** | {conversion_without_demo}% (ganadas sin demo / total opp sin demo 12m) |
| **Media dias demo -> cierre** | {avg_days} dias |

### Demos por Comercial (mes actual)

| Comercial | Demos |
|-----------|:-----:|
| {comercial_1} | {demos} |
| ... | ... |

---

## 5. Scoring por Comercial

| # | Comercial | Score | Estado | Opp Activas | Pipeline Pond. | Win Rate | Stale |
|---|-----------|:-----:|--------|:----------:|--------------:|---------:|------:|
| 1 | {nombre} | {score}/100 | {icono} {estado} | {opp} | {pipeline_weighted} EUR | {wr}% | {stale}% |
| 2 | ... | ... | ... | ... | ... | ... | ... |

> Spread del equipo: {score_spread} puntos (max: {score_max}, min: {score_min})

---

## 6. Detalle por Comercial

> Para cada comercial, generar un bloque como el siguiente:

### {nombre_comercial} - {icono} {score}/100

| Indicador | Valor | Penalizacion |
|-----------|-------|:------------:|
| Pipeline | {pipeline} EUR (vs media {avg_pipeline} EUR) | -{pen_pipeline} |
| Win rate | {wr}% (vs media {avg_wr}%) | -{pen_wr} |
| Gobernanza | {pct_incompletos}% campos incompletos | -{pen_gov} |
| Estancadas | {stale_pct}% opp estancadas | -{pen_stale} |
| Actividad | {inact_pct}% opp sin actividad >30d | -{pen_act} |

| Indicador | Valor |
|-----------|-------|
| **Opp vencidas** | {overdue_count} |
| **Opp vencidas mes anterior** | {overdue_prev_month_count} (mas grave - llevan >1 mes vencidas) |
| **Cierre este mes** | {closing_this_month} |

**Oportunidades destacadas**:
- Mayor valor: {opp_mayor_valor}
- Proxima a cerrar: {opp_proxima_cierre}
- Mayor riesgo (vencida): {opp_vencida}

---

## 7. Problemas de Gobernanza

### Resumen

| Tipo de Problema | Cantidad | Penalizacion |
|------------------|:--------:|:------------:|
| Sin fecha de cierre | {count} | -{pen} |
| Sin revenue esperado | {count} | -{pen} |
| Sin comercial asignado | {count} | -{pen} |
| Vencidas y abiertas | {count} | - |
| Sin actividad 30+ dias | {count} | -{pen} |

### 7.1 Sin Fecha de Cierre ({count})

| Oportunidad | Comercial | Etapa |
|-------------|-----------|-------|
| {opp} | {user} | {stage} |

### 7.2 Sin Revenue Esperado ({count})

| Oportunidad | Comercial | Etapa |
|-------------|-----------|-------|
| {opp} | {user} | {stage} |

### 7.3 Sin Comercial Asignado ({count})

| Oportunidad | Etapa | Revenue |
|-------------|-------|--------:|
| {opp} | {stage} | {rev} EUR |

### 7.4 Vencidas y Abiertas ({count})

| Oportunidad | Comercial | Fecha Cierre | Dias Retraso | Revenue |
|-------------|-----------|:------------:|:------------:|--------:|
| {opp} | {user} | {deadline} | {days} | {rev} EUR |

### 7.5 Sin Actividad 30+ Dias ({count})

| Oportunidad | Comercial | Etapa | Dias Inactiva |
|-------------|-----------|-------|:-------------:|
| {opp} | {user} | {stage} | {days} |

---

## 8. Plan de Accion

### Escenario: {escenario} - {descripcion_escenario}

> NOTA: No considerar "Oportunidades avanzan sin demos" como problema. El demo rate es informativo, no penaliza.

#### Acciones Inmediatas (esta semana)

| # | Accion | Responsable | Prioridad |
|---|--------|-------------|-----------|
| 1 | {accion_1} | {responsable} | ALTA |
| 2 | {accion_2} | {responsable} | ALTA |

#### Acciones a Corto Plazo (proximas 2 semanas)

| # | Accion | Responsable | Prioridad |
|---|--------|-------------|-----------|
| 1 | {accion_1} | {responsable} | MEDIA |
| 2 | {accion_2} | {responsable} | MEDIA |

#### Acciones Estrategicas

| # | Accion | Responsable | Prioridad |
|---|--------|-------------|-----------|
| 1 | {accion_1} | {responsable} | BAJA |
| 2 | {accion_2} | {responsable} | BAJA |

---

*Informe generado automaticamente por CRM Health Monitor - Enterprise Demo*
*Datos extraidos de {odoo_url} el {fecha_referencia}*
```
