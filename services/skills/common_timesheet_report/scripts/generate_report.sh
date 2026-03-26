#!/bin/bash
# generate_report.sh - Generate weekly/monthly timesheet report from Odoo
# Usage: ./generate_report.sh [start_date]           (weekly)
#        ./generate_report.sh --monthly [YYYY-MM]     (monthly)
#
# Requires: ODOO_URL, ODOO_DB, ODOO_KEY, ODOO_USER
# Output: Markdown formatted report to stdout

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_DIR="$(dirname "$SCRIPT_DIR")"
PILOT_DIR="$(dirname "$SKILL_DIR")/odoo-pilot"
CONFIG="$SCRIPT_DIR/config.json"

# ═══════════════════════════════════════════
# PARSE ARGUMENTS
# ═══════════════════════════════════════════

MONTHLY=false
if [ "${1:-}" = "--monthly" ]; then
    MONTHLY=true
    shift
fi

FORMAT="markdown"
if [ "${1:-}" = "--format" ]; then
    FORMAT="${2:-markdown}"
    shift 2
fi

# ═══════════════════════════════════════════
# DATE CALCULATION
# ═══════════════════════════════════════════

if [ "$MONTHLY" = true ]; then
    if [ -n "${1:-}" ]; then
        YEAR_MONTH="$1"
    else
        # Previous month
        YEAR_MONTH=$(date -d "last month" +%Y-%m)
    fi
    START_DATE="${YEAR_MONTH}-01"
    END_DATE=$(date -d "$START_DATE + 1 month - 1 day" +%Y-%m-%d)
    PREV_START=$(date -d "$START_DATE - 1 month" +%Y-%m-%d)
    PREV_END=$(date -d "$START_DATE - 1 day" +%Y-%m-%d)
    REPORT_TYPE="monthly"
else
    if [ -n "${1:-}" ]; then
        START_DATE="$1"
    else
        DOW=$(date +%u)
        if [ "$DOW" -eq 1 ]; then
            DAYS_BACK=7
        else
            DAYS_BACK=$((DOW - 1 + 7))
        fi
        START_DATE=$(date -d "-${DAYS_BACK} days" +%Y-%m-%d)
    fi
    END_DATE=$(date -d "$START_DATE + 4 days" +%Y-%m-%d)
    PREV_START=$(date -d "$START_DATE - 7 days" +%Y-%m-%d)
    PREV_END=$(date -d "$START_DATE - 1 day" +%Y-%m-%d)
    # For monthly billable context, get the month boundaries
    MONTH_START=$(date -d "$START_DATE" +%Y-%m-01)
    MONTH_END=$(date -d "$MONTH_START + 1 month - 1 day" +%Y-%m-%d)
    REPORT_TYPE="weekly"
fi

# ═══════════════════════════════════════════
# LOAD ENV
# ═══════════════════════════════════════════

if [ -f "$SKILL_DIR/.env" ]; then
    set -a; source "$SKILL_DIR/.env"; set +a
fi

# ═══════════════════════════════════════════
# AUTHENTICATE
# ═══════════════════════════════════════════

eval $(bash "$PILOT_DIR/scripts/auth.sh" 2>/dev/null)

# ═══════════════════════════════════════════
# READ CONFIG
# ═══════════════════════════════════════════

EMPLOYEE_IDS=$(node -e "const c=require('$CONFIG'); console.log(c.employees.map(e=>e.id).join(','))")

# ═══════════════════════════════════════════
# FETCH DATA
# ═══════════════════════════════════════════

# Current period timesheets
DOMAIN="[[\"employee_id\",\"in\",[$EMPLOYEE_IDS]],[\"date\",\">=\",\"$START_DATE\"],[\"date\",\"<=\",\"$END_DATE\"]]"
FIELDS='["employee_id","date","name","unit_amount","project_id","task_id"]'
CURRENT_DATA=$(bash "$PILOT_DIR/scripts/search_records.sh" account.analytic.line "$DOMAIN" "$FIELDS" 1000 2>/dev/null)

# Previous period timesheets (for comparison)
PREV_DOMAIN="[[\"employee_id\",\"in\",[$EMPLOYEE_IDS]],[\"date\",\">=\",\"$PREV_START\"],[\"date\",\"<=\",\"$PREV_END\"]]"
PREV_DATA=$(bash "$PILOT_DIR/scripts/search_records.sh" account.analytic.line "$PREV_DOMAIN" '["employee_id","unit_amount","project_id"]' 1000 2>/dev/null)

# Absences in the period
ABSENCE_DOMAIN="[[\"employee_id\",\"in\",[$EMPLOYEE_IDS]],[\"date_from\",\"<=\",\"$END_DATE 23:59:59\"],[\"date_to\",\">=\",\"$START_DATE 00:00:00\"],[\"state\",\"in\",[\"validate\",\"confirm\"]]]"
ABSENCE_DATA=$(bash "$PILOT_DIR/scripts/search_records.sh" hr.leave "$ABSENCE_DOMAIN" '["employee_id","holiday_status_id","date_from","date_to","number_of_days","state"]' 100 2>/dev/null)

# Internal project IDs (for billable classification)
INTERNAL_PROJECTS=$(bash "$PILOT_DIR/scripts/search_records.sh" project.project '["|",["is_internal_project","=",true],["name","ilike","interno"]]' '["id","name"]' 200 2>/dev/null)

# Monthly billable data for Elena (if weekly report, get full month context)
MONTHLY_BILLABLE=""
if [ "$REPORT_TYPE" = "weekly" ]; then
    MONTH_DOMAIN="[[\"employee_id\",\"=\",206],[\"date\",\">=\",\"$MONTH_START\"],[\"date\",\"<=\",\"$MONTH_END\"]]"
    MONTHLY_BILLABLE=$(bash "$PILOT_DIR/scripts/search_records.sh" account.analytic.line "$MONTH_DOMAIN" '["project_id","unit_amount"]' 500 2>/dev/null)
fi

# ═══════════════════════════════════════════
# GENERATE REPORT
# ═══════════════════════════════════════════

node -e '
const config = require(process.argv[1]);
const current = JSON.parse(process.argv[2]);
const prev = JSON.parse(process.argv[3]);
const absences = JSON.parse(process.argv[4]);
const internalProjects = JSON.parse(process.argv[5]);
const monthlyBillable = process.argv[6] ? JSON.parse(process.argv[6]) : [];
const startDate = process.argv[7];
const endDate = process.argv[8];
const reportType = process.argv[9];
const monthStart = process.argv[10] || "";

// Build internal project ID set
const internalIds = new Set(internalProjects.map(p => p.id));
const internalPatterns = config.internal_project_patterns.map(p => new RegExp(p, "i"));
function isInternal(projId, projName) {
    if (!projId) return true;
    if (internalIds.has(projId)) return true;
    return internalPatterns.some(p => p.test(projName));
}

// Build vague patterns
const vaguePatterns = config.thresholds.vague_entry_patterns.map(p => new RegExp(p, "i"));
function isVague(desc) {
    if (!desc || desc.trim().length <= 2) return true;
    return vaguePatterns.some(p => p.test(desc.trim()));
}

// Get working days in period (exclude weekends)
function getWorkingDays(start, end) {
    const days = [];
    let d = new Date(start + "T12:00:00");
    const e = new Date(end + "T12:00:00");
    while (d <= e) {
        const dow = d.getDay();
        if (dow >= 1 && dow <= 5) {
            days.push(d.toISOString().split("T")[0]);
        }
        d.setDate(d.getDate() + 1);
    }
    return days;
}

// Count absence days for employee in period
function getAbsenceDays(empId) {
    let days = 0;
    absences.filter(a => a.employee_id[0] === empId).forEach(a => {
        const aStart = new Date(a.date_from.split(" ")[0] + "T12:00:00");
        const aEnd = new Date(a.date_to.split(" ")[0] + "T12:00:00");
        const pStart = new Date(startDate + "T12:00:00");
        const pEnd = new Date(endDate + "T12:00:00");
        // Count overlap working days
        let d = new Date(Math.max(aStart, pStart));
        const e = new Date(Math.min(aEnd, pEnd));
        while (d <= e) {
            const dow = d.getDay();
            if (dow >= 1 && dow <= 5) days++;
            d.setDate(d.getDate() + 1);
        }
    });
    return days;
}

// Expected hours for working days minus absences
function expectedHours(empId) {
    const workDays = getWorkingDays(startDate, endDate);
    const absDays = getAbsenceDays(empId);
    const effectiveDays = workDays.length - absDays;
    // Average hours per day
    const avgPerDay = config.company.weekly_hours / 5;
    return effectiveDays * avgPerDay;
}

// Score emoji
function scoreEmoji(pct) {
    if (pct >= config.thresholds.green_pct) return "🟢";
    if (pct >= config.thresholds.yellow_pct) return "🟡";
    return "🔴";
}

// Previous period totals by employee
const prevByEmp = {};
prev.forEach(e => {
    const id = e.employee_id[0];
    prevByEmp[id] = (prevByEmp[id] || 0) + e.unit_amount;
});

// Filter entries with hours > 0
const entries = current.filter(e => e.unit_amount > 0);

// Group by employee
const byEmployee = {};
entries.forEach(e => {
    const empId = e.employee_id[0];
    if (!byEmployee[empId]) {
        byEmployee[empId] = { name: e.employee_id[1], entries: [], total: 0, billable: 0, internal: 0 };
    }
    byEmployee[empId].entries.push(e);
    byEmployee[empId].total += e.unit_amount;
    if (isInternal(e.project_id ? e.project_id[0] : null, e.project_id ? e.project_id[1] : "")) {
        byEmployee[empId].internal += e.unit_amount;
    } else {
        byEmployee[empId].billable += e.unit_amount;
    }
});

// Elena monthly billable tracking
let elenaMonthlyBillable = 0;
if (monthlyBillable.length > 0) {
    monthlyBillable.forEach(e => {
        if (e.unit_amount > 0 && e.project_id && !isInternal(e.project_id[0], e.project_id[1])) {
            elenaMonthlyBillable += e.unit_amount;
        }
    });
}

const workingDays = getWorkingDays(startDate, endDate);
const dayNames = { 1: "Lun", 2: "Mar", 3: "Mié", 4: "Jue", 5: "Vie" };

// ═══════════ RENDER REPORT ═══════════

if (reportType === "monthly") {
    console.log("📊 **Resumen Mensual de Timesheets**");
} else {
    console.log("📊 **Resumen Semanal de Timesheets**");
}
console.log("Período: " + startDate + " → " + endDate);
console.log("");

let grandTotal = 0;
let grandBillable = 0;
const alerts = [];

const sortedEmployees = Object.keys(byEmployee).sort((a, b) =>
    byEmployee[a].name.localeCompare(byEmployee[b].name)
);

sortedEmployees.forEach(empId => {
    const emp = byEmployee[empId];
    const empConfig = config.employees.find(e => e.id === parseInt(empId));
    grandTotal += emp.total;
    grandBillable += emp.billable;

    const expected = expectedHours(parseInt(empId));
    const pct = expected > 0 ? (emp.total / expected * 100) : 0;
    const absDays = getAbsenceDays(parseInt(empId));
    const prevTotal = prevByEmp[parseInt(empId)] || 0;
    const delta = emp.total - prevTotal;
    const deltaStr = delta >= 0 ? "+" + delta.toFixed(1) : delta.toFixed(1);

    // Score
    const emoji = scoreEmoji(pct);

    console.log("━━━━━━━━━━━━━━━━━━━━━━━━");
    console.log(emoji + " **" + (empConfig ? empConfig.short_name : emp.name) + "** — " + emp.total.toFixed(1) + "h / " + expected.toFixed(1) + "h (" + pct.toFixed(0) + "%) " + deltaStr + "h vs anterior");

    if (absDays > 0) {
        const absTypes = absences.filter(a => a.employee_id[0] === parseInt(empId))
            .map(a => a.holiday_status_id[1]).join(", ");
        console.log("🏖️ Ausencias: " + absDays + " día(s) — " + absTypes);
    }
    console.log("");

    // Billable vs Internal
    const billPct = emp.total > 0 ? (emp.billable / emp.total * 100) : 0;
    console.log("💰 Facturable: " + emp.billable.toFixed(1) + "h (" + billPct.toFixed(0) + "%) | Interno: " + emp.internal.toFixed(1) + "h (" + (100 - billPct).toFixed(0) + "%)");

    // Elena monthly billable target
    if (parseInt(empId) === 206 && empConfig && empConfig.billable_target_monthly && reportType === "weekly") {
        const target = empConfig.billable_target_monthly;
        const monthPct = (elenaMonthlyBillable / target * 100).toFixed(0);
        console.log("🎯 Objetivo mensual facturable: " + elenaMonthlyBillable.toFixed(1) + "h / " + target + "h (" + monthPct + "%)");
    }
    console.log("");

    // Weekly summary - deduce what they worked on from descriptions
    const themes = {};
    emp.entries.forEach(e => {
        const desc = (e.name || "").trim();
        if (!desc || desc === "/" || desc === "." || desc === "-") return;
        if (/descanso/i.test(desc)) return; // skip breaks
        const taskCat = e.task_id ? e.task_id[1].trim() : "";
        // Group by task category for theming
        const key = taskCat || "General";
        if (!themes[key]) themes[key] = [];
        themes[key].push(desc);
    });

    // Build concise summary: top activities with specific details
    const summaryParts = [];
    const sortedThemes = Object.keys(themes).sort((a, b) => themes[b].length - themes[a].length);
    sortedThemes.slice(0, 5).forEach(theme => {
        const items = themes[theme];
        // Get unique, non-generic descriptions
        const unique = [...new Set(items.filter(d => d.length > 3))].slice(0, 3);
        if (unique.length > 0) {
            summaryParts.push("**" + theme + "**: " + unique.join(", "));
        }
    });
    if (summaryParts.length > 0) {
        console.log("📋 *Resumen:* " + summaryParts.join(" · "));
        console.log("");
    }

    // Group by project
    const byProject = {};
    emp.entries.forEach(e => {
        const projName = e.project_id ? e.project_id[1] : "Sin proyecto";
        const projId = e.project_id ? e.project_id[0] : 0;
        if (!byProject[projName]) {
            byProject[projName] = { tasks: {}, total: 0, internal: isInternal(projId, projName) };
        }
        byProject[projName].total += e.unit_amount;
        const taskName = e.task_id ? e.task_id[1].trim() : "Sin tarea";
        if (!byProject[projName].tasks[taskName]) {
            byProject[projName].tasks[taskName] = 0;
        }
        byProject[projName].tasks[taskName] += e.unit_amount;
    });

    const sortedProjects = Object.keys(byProject).sort((a, b) => byProject[b].total - byProject[a].total);
    sortedProjects.forEach(projName => {
        const proj = byProject[projName];
        const projPct = (proj.total / emp.total * 100).toFixed(0);
        const marker = proj.internal ? "🏠" : "💼";
        console.log(marker + " " + projName + " — " + proj.total.toFixed(1) + "h (" + projPct + "%)");

        const sortedTasks = Object.keys(proj.tasks).sort((a, b) => proj.tasks[b] - proj.tasks[a]);
        sortedTasks.forEach(taskName => {
            const taskHours = proj.tasks[taskName];
            const taskPct = (taskHours / emp.total * 100).toFixed(0);
            console.log("  • " + taskName + ": " + taskHours.toFixed(1) + "h");

            // Task concentration alert
            if (taskPct > config.thresholds.task_concentration_pct) {
                alerts.push("⚡ " + (empConfig ? empConfig.short_name : emp.name) + ": \"" + taskName + "\" ocupa " + taskPct + "% del tiempo total");
            }
        });
    });
    console.log("");

    // Daily distribution
    const byDay = {};
    emp.entries.forEach(e => {
        if (!byDay[e.date]) byDay[e.date] = 0;
        byDay[e.date] += e.unit_amount;
    });

    const loggedDays = new Set(Object.keys(byDay));
    const absDates = new Set();
    absences.filter(a => a.employee_id[0] === parseInt(empId)).forEach(a => {
        let d = new Date(a.date_from.split(" ")[0] + "T12:00:00");
        const e = new Date(a.date_to.split(" ")[0] + "T12:00:00");
        while (d <= e) { absDates.add(d.toISOString().split("T")[0]); d.setDate(d.getDate() + 1); }
    });

    const dayLine = workingDays.map(d => {
        const dow = new Date(d + "T12:00:00").getDay();
        const name = dayNames[dow] || d;
        if (absDates.has(d)) return name + ":🏖️";
        if (!loggedDays.has(d)) return name + ":❌";
        return name + ":" + byDay[d].toFixed(1) + "h";
    }).join(" | ");
    console.log("📅 " + dayLine);

    // Missing days alert
    const missingDays = workingDays.filter(d => !loggedDays.has(d) && !absDates.has(d));
    if (missingDays.length > 0) {
        alerts.push("❌ " + (empConfig ? empConfig.short_name : emp.name) + ": sin horas registradas " + missingDays.length + " día(s)");
    }

    // Low hours alert
    if (emp.total < config.thresholds.low_hours_weekly && reportType === "weekly" && absDays < 3) {
        alerts.push("⚠️ " + (empConfig ? empConfig.short_name : emp.name) + ": solo " + emp.total.toFixed(1) + "h (esperadas " + expected.toFixed(1) + "h)");
    }

    // Excessive daily hours alert
    const maxDaily = config.thresholds.max_hours_daily || 9;
    Object.entries(byDay).forEach(([day, hours]) => {
        if (hours > maxDaily) {
            const dow = new Date(day + "T12:00:00").getDay();
            const dayName = dayNames[dow] || day;
            alerts.push("🕐 " + (empConfig ? empConfig.short_name : emp.name) + ": " + hours.toFixed(1) + "h el " + dayName + " " + day + " (máx " + maxDaily + "h)");
        }
    });

    // Vague entries
    const vagueEntries = emp.entries.filter(e => isVague(e.name));
    if (vagueEntries.length > 0) {
        const vagueHours = vagueEntries.reduce((s, e) => s + e.unit_amount, 0);
        alerts.push("📝 " + (empConfig ? empConfig.short_name : emp.name) + ": " + vagueEntries.length + " entrada(s) vagas (" + vagueHours.toFixed(1) + "h) — descripciones genéricas tipo \"/\" o \"Gestión\"");
    }

    console.log("");
});

// Handle employees with no entries
config.employees.forEach(empConfig => {
    if (!byEmployee[empConfig.id]) {
        const absDays = getAbsenceDays(empConfig.id);
        const expected = expectedHours(empConfig.id);
        if (absDays < workingDays.length) {
            console.log("━━━━━━━━━━━━━━━━━━━━━━━━");
            console.log("🔴 **" + empConfig.short_name + "** — 0.0h / " + expected.toFixed(1) + "h (0%)");
            if (absDays > 0) console.log("🏖️ Ausencias: " + absDays + " día(s)");
            alerts.push("🚨 " + empConfig.short_name + ": SIN HORAS REGISTRADAS en todo el período");
            console.log("");
        }
    }
});

// Summary
console.log("━━━━━━━━━━━━━━━━━━━━━━━━");
const billablePct = grandTotal > 0 ? (grandBillable / grandTotal * 100) : 0;
const grandExpected = config.employees.reduce((s, e) => s + expectedHours(e.id), 0);
const grandPct = grandExpected > 0 ? (grandTotal / grandExpected * 100) : 0;
const grandEmoji = scoreEmoji(grandPct);
console.log(grandEmoji + " **Total Marketing: " + grandTotal.toFixed(1) + "h / " + grandExpected.toFixed(1) + "h (" + grandPct.toFixed(0) + "%)** — 💼 " + grandBillable.toFixed(1) + "h facturable (" + (grandTotal > 0 ? (grandBillable/grandTotal*100).toFixed(0) : 0) + "%) / 🏠 " + (grandTotal - grandBillable).toFixed(1) + "h interno");

// Alerts section
if (alerts.length > 0) {
    console.log("");
    console.log("**⚠️ Alertas:**");
    alerts.forEach(a => console.log(a));
}
' "$CONFIG" "$CURRENT_DATA" "$PREV_DATA" "$ABSENCE_DATA" "$INTERNAL_PROJECTS" "$MONTHLY_BILLABLE" "$START_DATE" "$END_DATE" "$REPORT_TYPE" "${MONTH_START:-}"
