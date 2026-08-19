<template>
  <div class="view">

    <!-- Log Milestone -->
    <section class="section">
      <h2>Reportar condición de equipo</h2>
      <div v-if="myEquipment.length === 0" class="empty">No tienes equipos bajo custodia en este momento.</div>
      <form v-else class="form-grid" @submit.prevent="handleLogMilestone">
        <label>Equipo bajo tu custodia
          <select v-model="milestone.code" required>
            <option value="" disabled>— selecciona un equipo —</option>
            <option v-for="e in myEquipment" :key="e.publicKey.toBase58()" :value="codeToStr(e.account.code)">
              {{ codeToStr(e.account.code) }} — {{ e.account.description }}
            </option>
          </select>
        </label>
        <label>Condición
          <select v-model="milestone.condition">
            <option value="operational">Operacional</option>
            <option value="minorDamage">Daño menor</option>
            <option value="criticalDamage">Daño crítico</option>
            <option value="lost">Perdido</option>
          </select>
        </label>
        <label>Notas
          <input v-model="milestone.notes" placeholder="Observaciones de campo" required />
        </label>
        <div class="form-footer">
          <button type="submit" class="btn-primary" :disabled="busy.milestone">
            {{ busy.milestone ? 'Enviando…' : 'Reportar' }}
          </button>
          <span v-if="feedback.milestone" :class="feedbackClass(feedback.milestone)">{{ feedback.milestone }}</span>
        </div>
      </form>
    </section>

    <!-- Initiate Return -->
    <section class="section">
      <h2>Iniciar retorno de equipo</h2>
      <div v-if="myEquipment.length === 0" class="empty">No tienes equipos bajo custodia en este momento.</div>
      <form v-else class="form-grid" @submit.prevent="handleInitiateReturn">
        <label>Equipo bajo tu custodia
          <select v-model="ret.code" required>
            <option value="" disabled>— selecciona un equipo —</option>
            <option v-for="e in myEquipment" :key="e.publicKey.toBase58()" :value="codeToStr(e.account.code)">
              {{ codeToStr(e.account.code) }} — {{ e.account.description }}
            </option>
          </select>
        </label>
        <div class="form-footer">
          <button type="submit" class="btn-primary" :disabled="busy.ret">
            {{ busy.ret ? 'Enviando…' : 'Iniciar retorno' }}
          </button>
          <span v-if="feedback.ret" :class="feedbackClass(feedback.ret)">{{ feedback.ret }}</span>
        </div>
      </form>
    </section>

    <!-- Consultar equipo -->
    <section class="section">
      <h2>Consultar estado de equipo</h2>
      <form class="form-grid" @submit.prevent="handleFetchEquipment">
        <label>Equipo
          <select v-model="query.code" required>
            <option value="" disabled>— selecciona un equipo —</option>
            <option v-for="e in allEquipment" :key="e.publicKey.toBase58()" :value="codeToStr(e.account.code)">
              {{ codeToStr(e.account.code) }} — {{ e.account.description }}
            </option>
          </select>
        </label>
        <div class="form-footer">
          <button type="submit" class="btn-secondary" :disabled="busy.query">
            {{ busy.query ? 'Consultando…' : 'Consultar' }}
          </button>
        </div>
      </form>
      <div v-if="equipmentData" class="card-grid" style="margin-top:1rem">
        <div class="card">
          <span class="label">Estado</span>
          <span class="badge" :class="statusClass(equipmentData.status)">{{ formatStatus(equipmentData.status) }}</span>
        </div>
        <div class="card">
          <span class="label">Condición</span>
          <span class="badge" :class="conditionClass(equipmentData.reportedCondition)">{{ formatCondition(equipmentData.reportedCondition) }}</span>
        </div>
        <div class="card">
          <span class="label">Custodio</span>
          <span v-if="personnelMap[equipmentData.custodian?.toBase58?.()]" class="value value-compact">
            {{ personnelMap[equipmentData.custodian.toBase58()] }}
            <span class="muted mono" style="font-size:0.75rem"> · {{ shortKey(equipmentData.custodian) }}</span>
          </span>
          <span v-else class="value value-compact mono">{{ shortKey(equipmentData.custodian) }}</span>
        </div>
        <div class="card">
          <span class="label">Incidente</span>
          <span class="value value-compact">
            {{ Object.keys(equipmentData.status)[0] === 'available'
              ? '—'
              : `#${equipmentData.incidentId.toString()} — ${incidentMap[equipmentData.incidentId.toString()] ?? '?'}` }}
          </span>
        </div>
      </div>
      <div v-if="equipmentData" style="margin-top:1.5rem">
        <h3 class="subheading">Bitácora</h3>
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
                <td><span class="badge" :class="conditionClass(e.account.condition)">{{ formatCondition(e.account.condition) }}</span></td>
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
      <p v-if="feedback.query" :class="feedbackClass(feedback.query)">{{ feedback.query }}</p>
    </section>

  </div>
</template>

<script setup lang="ts">
import { reactive, ref, computed, onMounted } from 'vue'
import { useProgram, toCode } from '../composables/useProgram'
import { useWallet } from '../composables/useWallet'

const {
  logMilestone, initiateReturn, fetchEquipment, fetchAllEquipment,
  fetchAllPersonnel, fetchAllIncidents, fetchLogEntries,
} = useProgram()
const { publicKey } = useWallet()

const busy = reactive({ milestone: false, ret: false, query: false })
const feedback = reactive({ milestone: '', ret: '', query: '' })

const milestone = reactive({ code: '', condition: 'operational', notes: '' })
const ret = reactive({ code: '' })
const query = reactive({ code: '' })
const equipmentData = ref<any>(null)
const allEquipment = ref<any[]>([])
const allPersonnel = ref<any[]>([])
const allIncidents = ref<any[]>([])
const logEntries = ref<any[]>([])
const loadingLog = ref(false)

const myEquipment = computed(() => {
  if (!publicKey.value) return []
  const myPk = publicKey.value.toBase58()
  return allEquipment.value.filter(e => {
    const custodian = e.account.custodian?.toBase58?.() ?? ''
    return custodian === myPk && Object.keys(e.account.status)[0] === 'inUse'
  })
})

const personnelMap = computed<Record<string, string>>(() => {
  const map: Record<string, string> = {}
  for (const p of allPersonnel.value) {
    map[p.account.wallet.toBase58()] = p.account.name
  }
  return map
})

const incidentMap = computed<Record<string, string>>(() => {
  const map: Record<string, string> = {}
  for (const i of allIncidents.value) {
    map[i.account.incidentId.toString()] = i.account.description
  }
  return map
})

onMounted(async () => {
  const [eq, pers, inc] = await Promise.all([
    fetchAllEquipment(), fetchAllPersonnel(), fetchAllIncidents(),
  ])
  allEquipment.value = eq
  allPersonnel.value = pers
  allIncidents.value = inc
})

// ── Formatters ─────────────────────────────────────────────────────────────
function codeToStr(code: number[]): string {
  return String.fromCharCode(...code.filter(b => b !== 0))
}

function feedbackClass(msg: string) {
  return msg.startsWith('✓') ? 'feedback-ok' : 'feedback-err'
}

function shortKey(pk: any): string {
  const s = pk?.toBase58?.() ?? pk?.toString?.() ?? '—'
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

function conditionClass(c: any): string {
  const map: Record<string, string> = {
    operational: 'badge-green', minorDamage: 'badge-yellow',
    criticalDamage: 'badge-red', lost: 'badge-red',
  }
  return map[Object.keys(c)[0]] ?? ''
}

function formatTimestamp(ts: any): string {
  const n = Number(ts.toString())
  if (n === 0) return '—'
  return new Date(n * 1000).toLocaleString()
}

// ── Handlers ───────────────────────────────────────────────────────────────
async function handleLogMilestone() {
  busy.milestone = true; feedback.milestone = ''
  try {
    const conditionVariant = { [milestone.condition]: {} }
    await logMilestone(toCode(milestone.code), milestone.notes, conditionVariant)
    feedback.milestone = '✓ Condición reportada'
    milestone.code = ''; milestone.notes = ''
    allEquipment.value = await fetchAllEquipment()
  } catch (e: any) {
    feedback.milestone = '✗ ' + (e.message ?? e)
  } finally { busy.milestone = false }
}

async function handleInitiateReturn() {
  busy.ret = true; feedback.ret = ''
  try {
    await initiateReturn(toCode(ret.code))
    feedback.ret = '✓ Retorno iniciado'
    ret.code = ''
    allEquipment.value = await fetchAllEquipment()
  } catch (e: any) {
    feedback.ret = '✗ ' + (e.message ?? e)
  } finally { busy.ret = false }
}

async function handleFetchEquipment() {
  busy.query = true; feedback.query = ''; equipmentData.value = null; logEntries.value = []
  try {
    const code = toCode(query.code)
    equipmentData.value = await fetchEquipment(code)
    if (!equipmentData.value) {
      feedback.query = '✗ Equipo no encontrado'
    } else {
      loadingLog.value = true
      try {
        const entries = await fetchLogEntries(code)
        logEntries.value = [...entries].sort(
          (a, b) => Number(b.account.entryIndex.toString()) - Number(a.account.entryIndex.toString())
        )
      } finally { loadingLog.value = false }
    }
  } catch {
    feedback.query = '✗ Equipo no encontrado'
  } finally { busy.query = false }
}
</script>
