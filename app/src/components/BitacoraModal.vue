<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal">
      <div class="modal-header">
        <h3>Bitácora — {{ equipmentCode }}</h3>
        <div class="modal-header-actions">
          <button class="btn-secondary" :disabled="loading || entries.length === 0" @click="exportPdf">Exportar PDF</button>
          <button class="btn-close" @click="$emit('close')">✕</button>
        </div>
      </div>
      <div class="modal-body">
        <div v-if="loading" class="empty">Cargando…</div>
        <div v-else-if="entries.length === 0" class="empty">Sin reportes registrados para este equipo.</div>
        <div v-else class="table-wrap">
          <table>
            <thead>
              <tr><th>#</th><th>Condición</th><th>Notas</th><th>Operador</th><th>Fecha/Hora</th></tr>
            </thead>
            <tbody>
              <tr v-for="e in entries" :key="e.publicKey.toBase58()">
                <td>{{ e.account.entryIndex.toString() }}</td>
                <td>{{ formatCondition(e.account.condition) }}</td>
                <td>{{ e.account.notes }}</td>
                <td>{{ personnelMap[e.account.operator?.toBase58?.()] ?? shortKey(e.account.operator) }}</td>
                <td>{{ formatTimestamp(e.account.timestamp) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import jsPDF from 'jspdf'
import autoTable from 'jspdf-autotable'

const props = defineProps<{
  equipmentCode: string
  entries: any[]
  loading: boolean
  personnelMap: Record<string, string>
}>()

defineEmits<{ close: [] }>()

function shortKey(pk: any): string {
  const s = pk?.toBase58?.() ?? '—'
  return s === '11111111111111111111111111111111' ? '(sin asignar)' : `${s.slice(0, 4)}…${s.slice(-4)}`
}

function formatCondition(c: any): string {
  const map: Record<string, string> = {
    operational: 'Operacional', minorDamage: 'Daño menor',
    criticalDamage: 'Daño crítico', lost: 'Perdido',
  }
  return map[Object.keys(c)[0]] ?? Object.keys(c)[0]
}

function formatTimestamp(ts: any): string {
  const n = Number(ts.toString())
  if (n === 0) return '—'
  return new Date(n * 1000).toLocaleString()
}

function exportPdf() {
  const doc = new jsPDF()
  doc.setFontSize(14)
  doc.text(`Bitácora — ${props.equipmentCode}`, 14, 16)
  doc.setFontSize(10)
  doc.text(`Generado: ${new Date().toLocaleString()}`, 14, 22)

  autoTable(doc, {
    startY: 28,
    head: [['#', 'Condición', 'Notas', 'Operador', 'Fecha/Hora']],
    body: props.entries.map(e => [
      e.account.entryIndex.toString(),
      formatCondition(e.account.condition),
      e.account.notes,
      props.personnelMap[e.account.operator?.toBase58?.()] ?? shortKey(e.account.operator),
      formatTimestamp(e.account.timestamp),
    ]),
  })

  doc.save(`bitacora_${props.equipmentCode}.pdf`)
}
</script>
