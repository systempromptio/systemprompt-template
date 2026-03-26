# Patrones de Diagnostico CRM

Documento de referencia con los 6 escenarios de diagnostico del CRM,
algoritmo de seleccion y acciones recomendadas por escenario.

---

## 1. Escenarios de Diagnostico

### 1.1 HEALTHY_PIPELINE - CRM en Buen Estado

**Condicion**: Score >= 80

**Diagnostico**: El pipeline comercial esta sano. Las oportunidades avanzan, los datos estan completos y el equipo mantiene un ritmo adecuado de demos y cierres.

**Acciones de mantenimiento**:
- Mantener el ritmo actual de actividad comercial
- Revisar oportunidades con fecha de cierre proxima para asegurar seguimiento
- Identificar oportunidades de alto valor que necesiten atencion especial
- Documentar buenas practicas del equipo para replicar

**Recomendaciones**:
- Reunion de pipeline review quincenal
- No se requieren acciones correctivas
- Considerar incrementar objetivo mensual si la cobertura es alta

---

### 1.2 LOW_PIPELINE - Pipeline Insuficiente

**Condicion**: Total opp activas < 10 O cobertura < 50% (si hay objetivo)

**Diagnostico**: No hay suficientes oportunidades en el pipeline para alcanzar los objetivos comerciales. El embudo esta seco y hay riesgo de no cubrir las ventas necesarias.

**Acciones inmediatas** (esta semana):
1. Revisar leads no convertidos: hay leads sin cualificar que podrian ser oportunidades?
2. Activar campanas de generacion de leads (email, LinkedIn, llamadas)
3. Revisar clientes existentes para oportunidades de upsell/cross-sell
4. Recuperar oportunidades perdidas recientemente: alguna se puede reactivar?

**Acciones a corto plazo** (proximas 2 semanas):
1. Planificar campanas de marketing digital o eventos
2. Pedir referencias a clientes satisfechos
3. Revisar segmentacion de mercado: hay segmentos no explorados?
4. Considerar alianzas con partners para generacion de leads

**Acciones estrategicas**:
1. Revisar el proceso de generacion de leads
2. Evaluar si el objetivo mensual es realista con los recursos actuales
3. Considerar contratar SDR (Sales Development Representative)
4. Analizar que canales generan leads de mayor calidad

---

### 1.3 STALE_PIPELINE - Pipeline Inflado con Oportunidades Muertas

**Condicion**: > 40% de opp estancadas (sin actividad > 21 dias)

**Diagnostico**: El pipeline tiene un porcentaje alto de oportunidades sin actividad reciente. Esto infla artificialmente el valor del pipeline y distorsiona las previsiones. Muchas de estas oportunidades probablemente estan muertas.

**Acciones inmediatas** (esta semana):
1. Identificar todas las oportunidades sin actividad > 30 dias
2. Contactar las top 10 oportunidades estancadas de mayor valor
3. Las que no respondan en 5 dias: mover a etapa "Pausa" o marcar como perdidas
4. Limpiar oportunidades claramente muertas (sin contacto > 60 dias)

**Acciones a corto plazo** (proximas 2 semanas):
1. Establecer regla: oportunidad sin actividad > 30 dias = requiere justificacion
2. Implementar recordatorios automaticos de seguimiento en Odoo
3. Crear actividades programadas para todas las oportunidades activas
4. Revisar el proceso de cualificacion: oportunidades mal cualificadas avanzan?

**Acciones estrategicas**:
1. Definir SLA de seguimiento por etapa (ej: nueva = contactar en 48h)
2. Implementar automatizaciones de limpieza periodica
3. Revisar criterios de cualificacion BANT o similar
4. KPI de equipo: % de pipeline activo (no estancado) como objetivo

---

### 1.4 LOW_WIN_RATE - Tasa de Cierre Baja

**Condicion**: Win rate < 15%

**Diagnostico**: Se pierden demasiadas oportunidades. El ratio de conversion es bajo, lo que indica problemas en el proceso de venta, la cualificacion de leads o la competitividad de la oferta.

**Acciones inmediatas** (esta semana):
1. Analizar las ultimas 10 oportunidades perdidas: motivos de perdida
2. Comparar precios con la competencia en las oportunidades perdidas
3. Revisar si las oportunidades perdidas estaban bien cualificadas
4. Hablar con los comerciales sobre objeciones frecuentes

**Acciones a corto plazo** (proximas 2 semanas):
1. Categorizar motivos de perdida: precio, funcionalidad, competencia, timing
2. Reforzar formacion en manejo de objeciones
3. Revisar las propuestas comerciales: son claras y competitivas?
4. Implementar proceso de post-mortem para oportunidades perdidas

**Acciones estrategicas**:
1. Mejorar criterios de cualificacion para filtrar mejor los leads
2. Revisar la propuesta de valor: esta alineada con las necesidades del mercado?
3. Considerar ajustes de pricing o packaging
4. Analizar win rate por segmento/sector para focalizar esfuerzos

---

### 1.5 GOVERNANCE_ISSUES - Datos de Baja Calidad

**Condicion**: Penalizacion de gobernanza > 15 puntos

**Diagnostico**: Los datos del CRM estan incompletos o desactualizados. Faltan campos criticos como fecha de cierre, revenue esperado o comercial asignado. Esto impide tener visibilidad real del pipeline y hacer previsiones fiables.

**Acciones inmediatas** (esta semana):
1. Generar lista de oportunidades con datos incompletos
2. Asignar tarea de limpieza: cada comercial completa sus oportunidades
3. Las oportunidades sin comercial: asignar responsable inmediatamente
4. Completar expected_revenue en las oportunidades de mayor probabilidad

**Acciones a corto plazo** (proximas 2 semanas):
1. Establecer campos obligatorios en Odoo para avanzar de etapa
2. Hacer pipeline review semanal donde se revisen datos incompletos
3. Crear dashboard en Odoo con indicador de completitud de datos
4. Definir minimo de informacion requerida por etapa

**Acciones estrategicas**:
1. Implementar reglas de automatizacion en Odoo (alertas de campos vacios)
2. KPI de equipo: % de completitud de datos como objetivo
3. Formar al equipo sobre la importancia de los datos para previsiones
4. Considerar gamificacion: ranking de completitud por comercial

---

### 1.6 UNBALANCED_TEAM - Rendimiento Desigual

**Condicion**: Diferencia entre score maximo y minimo del equipo > 40 puntos

**Diagnostico**: Hay una gran disparidad en el rendimiento individual de los comerciales. Algunos comerciales tienen scores altos mientras otros estan muy por debajo. Esto puede indicar problemas de formacion, motivacion, asignacion de territorios o carga de trabajo desigual.

**Acciones inmediatas** (esta semana):
1. Identificar los comerciales con score mas bajo
2. Reunion 1:1 con los comerciales de menor rendimiento para entender causas
3. Revisar si la asignacion de oportunidades es equitativa
4. Verificar si los comerciales de bajo score tienen bloqueos especificos

**Acciones a corto plazo** (proximas 2 semanas):
1. Emparejar comerciales de alto y bajo rendimiento (mentoring)
2. Compartir mejores practicas del comercial top en reunion de equipo
3. Revisar la distribucion de territorios/cuentas
4. Ajustar objetivos individuales si la carga es desigual

**Acciones estrategicas**:
1. Programa de formacion continua para el equipo
2. Revisar proceso de onboarding de nuevos comerciales
3. Evaluar si la estructura del equipo es adecuada
4. Considerar especializacion por sector o tamano de cuenta

---

## 2. Algoritmo de Seleccion de Escenario

```
1. Si total_opp_activas < 10 O (objetivo > 0 Y cobertura < 50):
   → LOW_PIPELINE

2. Si score >= 80:
   → HEALTHY_PIPELINE

3. Si stale_pct > 40:
   → STALE_PIPELINE

4. Si total_opp_12m >= 5 Y win_rate < 15:
   → LOW_WIN_RATE

5. Si pen_gobernanza > 15:
   → GOVERNANCE_ISSUES

6. Si num_comerciales >= 3 Y (score_max - score_min) > 40:
   → UNBALANCED_TEAM

7. Else:
   → Evaluar combinacion. Reportar los 2-3 problemas principales.
```

> **NOTA**: El escenario NO_DEMOS ha sido eliminado. El demo rate es informativo, no es un problema que requiera accion correctiva.

**Nota**: Los escenarios se evaluan en orden de prioridad. Solo se reporta
el escenario principal, pero el informe detalla todos los problemas encontrados.

---

## 3. Detalle de Metricas por Escenario

### 3.1 Metricas a Destacar por Escenario

| Escenario | Metricas Clave |
|-----------|---------------|
| HEALTHY_PIPELINE | Win rate, forecast, cobertura |
| LOW_PIPELINE | Total opps, valor pipeline, cobertura |
| STALE_PIPELINE | % estancadas, lista top estancadas |
| LOW_WIN_RATE | Win rate, motivos perdida, conversion demo |
| GOVERNANCE_ISSUES | Campos vacios por tipo, lista opps incompletas |
| UNBALANCED_TEAM | Tabla scores, spread, top/bottom comerciales |
