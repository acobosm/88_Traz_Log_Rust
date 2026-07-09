<template>
  <div class="view">

    <!-- Panel del incidente activo -->
    <section class="section">
      <div class="inv-header">
        <h2>Panel del incidente</h2>
        <button class="btn-secondary" :disabled="loadingPanel" @click="loadPanel">
          {{ loadingPanel ? 'Cargando…' : 'Actualizar' }}
        </button>
      </div>
      <div v-if="activeIncidents.length === 0" class="empty">Sin incidentes activos.</div>
      <template v-else>
        <label class="panel-select-label">
          Selecciona incidente
          <select v-model.number="selectedIncidentId">
            <option v-for="i in activeIncidents" :key="i.publicKey.toBase58()" :value="Number(i.account.incidentId.toString())">
              #{{ i.account.incidentId.toString() }} — {{ i.account.coordinates }} · Riesgo {{ i.account.riskLevel }}/5
            </option>
          </select>
        </label>
        <div v-if="incidentEquipment.length === 0" class="empty" style="margin-top:0.75rem">Sin equipos asignados a este incidente.</div>
        <div v-else class="table-wrap" style="margin-top:0.75rem">
          <table>
            <thead>
              <tr><th>Código</th><th>Descripción</th><th>Estado</th><th>Condición</th><th>Custodio</th><th>Acciones</th></tr>
            </thead>
            <tbody>
              <tr v-for="e in incidentEquipment" :key="e.publicKey.toBase58()">
                <td class="mono">{{ codeToStr(e.account.code) }}</td>
                <td>{{ e.account.description }}</td>
                <td><span class="badge" :class="statusClass(e.account.status)">{{ formatStatus(e.account.status) }}</span></td>
                <td>{{ formatCondition(e.account.reportedCondition) }}</td>
                <td>
                  <span v-if="personnelMap[e.account.custodian?.toBase58?.()]">
                    {{ personnelMap[e.account.custodian.toBase58()] }}
                    <span class="muted mono" style="font-size:0.75rem"> · {{ shortKey(e.account.custodian) }}</span>
                  </span>
                  <span v-else class="mono">{{ shortKey(e.account.custodian) }}</span>
                </td>
                <td>
                  <button class="btn-log" @click="openLog(e)">Bitácora</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>
    </section>

    <!-- Abrir incidente -->
    <section class="section">
      <h2>Abrir incidente</h2>
      <form class="form-grid" @submit.prevent="handleOpenIncident">
        <label>Descripción
          <input v-model="open.description" placeholder="Ej: Incendio forestal sector norte" maxlength="64" required />
        </label>
        <label>Coordenadas
          <input v-model="open.coordinates" placeholder="Ej: 9.934,-84.082" required />
        </label>
        <label>Nivel de riesgo (1–5)
          <input v-model.number="open.riskLevel" type="number" min="1" max="5" required />
        </label>
        <div class="form-footer">
          <button type="submit" class="btn-primary" :disabled="busy.open">
            {{ busy.open ? 'Enviando…' : 'Abrir incidente' }}
          </button>
          <span v-if="feedback.open" :class="feedbackClass(feedback.open)">{{ feedback.open }}</span>
        </div>
      </form>
    </section>

    <!-- Asignar equipo -->
    <section class="section">
      <h2>Asignar equipo a operador</h2>
      <div v-if="activeIncidents.length === 0" class="empty">No hay incidentes activos. Abre uno primero.</div>
      <form v-else class="form-grid" @submit.prevent="handleAssignEquipment">
        <label>Equipo disponible
          <select v-model="assign.code" required>
            <option value="" disabled>— selecciona un equipo —</option>
            <option v-for="e in availableEquipment" :key="e.publicKey.toBase58()" :value="codeToStr(e.account.code)">
              {{ codeToStr(e.account.code) }} — {{ e.account.description }}
            </option>
          </select>
        </label>
        <label>Incidente
          <select v-model.number="assign.incidentId" required>
            <option v-for="i in activeIncidents" :key="i.publicKey.toBase58()" :value="Number(i.account.incidentId.toString())">
              #{{ i.account.incidentId.toString() }} — {{ i.account.coordinates }}
            </option>
          </select>
        </label>
        <label>Operador
          <select v-model="assign.operatorWallet" required>
            <option value="" disabled>— selecciona un operador —</option>
            <option v-for="p in operators" :key="p.publicKey.toBase58()" :value="p.account.wallet.toBase58()">
              {{ p.account.name }} ({{ p.account.wallet.toBase58().slice(0,4) }}…{{ p.account.wallet.toBase58().slice(-4) }})
            </option>
          </select>
        </label>
        <div class="form-footer">
          <button type="submit" class="btn-primary" :disabled="busy.assign">
            {{ busy.assign ? 'Enviando…' : 'Asignar' }}
          </button>
          <span v-if="feedback.assign" :class="feedbackClass(feedback.assign)">{{ feedback.assign }}</span>
        </div>
      </form>
    </section>

    <!-- Modal bitácora -->
    <div v-if="showLogModal" class="modal-overlay" @click.self="showLogModal = false">
      <div class="modal">
        <div class="modal-header">
          <h3>Bitácora — {{ selectedEquipmentCode }}</h3>
          <button class="btn-close" @click="showLogModal = false">✕</button>
        </div>
        <div class="modal-body">
          <div v-if="loadingLog" class="empty">Cargando…</div>
          <div v-else-if="logEntries.length === 0" class="empty">Sin reportes registrados para este equipo.</div>
          <div v-else class="table-wrap">
            <table>
              <thead>
                <tr><th>#</th><th>Condición</th><th>Notas</th><th>Operador</th><th>Fecha/Hora</th></tr>
              </thead>
              <tbody>
                <tr v-for="e in logEntries" :key="e.publicKey.toBase58()">
                  <td>{{ e.account.entryIndex.toString() }}</td>
                  <td>{{ formatCondition(e.account.condition) }}</td>
                  <td>{{ e.account.notes }}</td>
                  <td>
                    {{ personnelMap[e.account.operator?.toBase58?.()] ?? shortKey(e.account.operator) }}
                  </td>
                  <td>{{ formatTimestamp(e.account.timestamp) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>

    <!-- Cerrar incidente -->
    <section class="section">
      <h2>Cerrar incidente</h2>
      <div v-if="activeIncidents.length === 0" class="empty">Sin incidentes activos.</div>
      <form v-else class="form-grid" @submit.prevent="handleCloseIncident">
        <label>Incidente
          <select v-model.number="close.incidentId" required>
            <option v-for="i in activeIncidents" :key="i.publicKey.toBase58()" :value="Number(i.account.incidentId.toString())">
              #{{ i.account.incidentId.toString() }} — {{ i.account.coordinates }}
            </option>
          </select>
        </label>
        <div class="form-footer">
          <button type="submit" class="btn-danger" :disabled="busy.close">
            {{ busy.close ? 'Enviando…' : 'Cerrar incidente' }}
          </button>
          <span v-if="feedback.close" :class="feedbackClass(feedback.close)">{{ feedback.close }}</span>
        </div>
      </form>
    </section>

  </div>
</template>

<script setup lang="ts">
import { reactive, ref, computed, onMounted } from 'vue'
import { PublicKey } from '@solana/web3.js'
import { useProgram, toCode } from '../composables/useProgram'

const {
  fetchGlobalState, fetchAllEquipment, fetchAllIncidents, fetchAllPersonnel,
  fetchLogEntries, openFireIncident, assignEquipment, closeIncident,
} = useProgram()

const busy = reactive({ open: false, assign: false, close: false })
const feedback = reactive({ open: '', assign: '', close: '' })

const open = reactive({ description: '', coordinates: '', riskLevel: 3 })
const assign = reactive({ code: '', incidentId: 0, operatorWallet: '' })
const close = reactive({ incidentId: 0 })

// ── Panel ─────────────────────────────────────────────────────────────────
const allEquipment = ref<any[]>([])
const allIncidents = ref<any[]>([])
const allPersonnel = ref<any[]>([])
const operators = ref<any[]>([])
const loadingPanel = ref(false)
const selectedIncidentId = ref(0)

const personnelMap = computed<Record<string, string>>(() => {
  const map: Record<string, string> = {}
  for (const p of allPersonnel.value) {
    map[p.account.wallet.toBase58()] = p.account.name
  }
  return map
})

const activeIncidents = computed(() =>
  allIncidents.value.filter(i => i.account.isActive)
)

const availableEquipment = computed(() =>
  allEquipment.value.filter(e => Object.keys(e.account.status)[0] === 'available')
)

const incidentEquipment = computed(() =>
  allEquipment.value.filter(e =>
    Number(e.account.incidentId.toString()) === selectedIncidentId.value &&
    Object.keys(e.account.status)[0] !== 'available'
  )
)

async function loadPanel() {
  loadingPanel.value = true
  try {
    const [eq, inc, pers] = await Promise.all([
      fetchAllEquipment(),
      fetchAllIncidents(),
      fetchAllPersonnel(),
    ])
    allEquipment.value = eq
    allIncidents.value = inc
    allPersonnel.value = pers
    operators.value = pers.filter(p => Object.keys(p.account.role)[0] === 'operator')
    if (activeIncidents.value.length > 0) {
      selectedIncidentId.value = Number(activeIncidents.value[0].account.incidentId.toString())
      assign.incidentId = selectedIncidentId.value
      close.incidentId = selectedIncidentId.value
    }
  } finally {
    loadingPanel.value = false
  }
}

onMounted(loadPanel)

// ── Modal bitácora ─────────────────────────────────────────────────────────
const showLogModal = ref(false)
const selectedEquipmentCode = ref('')
const logEntries = ref<any[]>([])
const loadingLog = ref(false)

async function openLog(equipment: any) {
  selectedEquipmentCode.value = codeToStr(equipment.account.code)
  showLogModal.value = true
  loadingLog.value = true
  try {
    logEntries.value = await fetchLogEntries(equipment.account.code)
  } finally {
    loadingLog.value = false
  }
}

// ── Formatters ─────────────────────────────────────────────────────────────
function codeToStr(code: number[]): string {
  return String.fromCharCode(...code.filter(b => b !== 0))
}

function shortKey(pk: any): string {
  const s = pk?.toBase58?.() ?? '—'
  return s === '11111111111111111111111111111111' ? '(sin asignar)' : `${s.slice(0, 4)}…${s.slice(-4)}`
}

function formatStatus(s: any): string {
  const map: Record<string, string> = {
    available: 'Disponible', inUse: 'En uso',
    inRepair: 'En reparación', lost: 'Perdido', returning: 'Retornando',
  }
  return map[Object.keys(s)[0]] ?? Object.keys(s)[0]
}

function formatCondition(c: any): string {
  const map: Record<string, string> = {
    operational: 'Operacional', minorDamage: 'Daño menor',
    criticalDamage: 'Daño crítico', lost: 'Perdido',
  }
  return map[Object.keys(c)[0]] ?? Object.keys(c)[0]
}

function statusClass(s: any): string {
  const map: Record<string, string> = {
    available: 'badge-green', inUse: 'badge-yellow',
    inRepair: 'badge-red', lost: 'badge-red', returning: 'badge-yellow',
  }
  return map[Object.keys(s)[0]] ?? ''
}

function formatTimestamp(ts: any): string {
  const n = Number(ts.toString())
  if (n === 0) return '—'
  return new Date(n * 1000).toLocaleString()
}

function feedbackClass(msg: string) {
  return msg.startsWith('✓') ? 'feedback-ok' : 'feedback-err'
}

// ── Handlers ───────────────────────────────────────────────────────────────
async function handleOpenIncident() {
  busy.open = true; feedback.open = ''
  try {
    const gs = await fetchGlobalState()
    const incidentId = gs ? Number(gs.nextIncidentId.toString()) : 0
    await openFireIncident(incidentId, open.description, open.coordinates, open.riskLevel)
    feedback.open = `✓ Incidente #${incidentId} abierto`
    open.description = ''; open.coordinates = ''
    await loadPanel()
  } catch (e: any) {
    feedback.open = '✗ ' + (e.message ?? e)
  } finally { busy.open = false }
}

async function handleAssignEquipment() {
  busy.assign = true; feedback.assign = ''
  try {
    const operatorWallet = new PublicKey(assign.operatorWallet)
    await assignEquipment(toCode(assign.code), assign.incidentId, operatorWallet)
    feedback.assign = `✓ ${assign.code} asignado`
    assign.code = ''; assign.operatorWallet = ''
    await loadPanel()
  } catch (e: any) {
    feedback.assign = '✗ ' + (e.message ?? e)
  } finally { busy.assign = false }
}

async function handleCloseIncident() {
  busy.close = true; feedback.close = ''
  try {
    await closeIncident(close.incidentId)
    feedback.close = `✓ Incidente #${close.incidentId} cerrado`
    await loadPanel()
  } catch (e: any) {
    feedback.close = '✗ ' + (e.message ?? e)
  } finally { busy.close = false }
}
</script>
