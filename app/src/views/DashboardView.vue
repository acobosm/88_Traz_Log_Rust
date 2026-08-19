<template>
  <div class="dashboard">
    <h2>Estado del Sistema</h2>

    <div v-if="loading" class="status">Cargando…</div>

    <div v-else-if="state" class="card-grid">
      <div class="card">
        <span class="label">Sistema</span>
        <span :class="['badge', state.isPaused ? 'badge-red' : 'badge-green']">
          {{ state.isPaused ? 'PAUSADO' : 'ACTIVO' }}
        </span>
      </div>
      <div class="card">
        <span class="label">Próximo ID de incidente</span>
        <span class="value">{{ state.nextIncidentId.toString() }}</span>
      </div>
      <div class="card">
        <span class="label">Admin</span>
        <span class="value mono">{{ shortAdmin }}</span>
      </div>
      <div class="card">
        <span class="label">Programa</span>
        <span class="value mono">{{ shortProgramId }}</span>
      </div>
    </div>

    <div v-else class="status muted">
      No se pudo leer GlobalState. ¿Está corriendo <code>anchor localnet</code>?
    </div>

    <!-- Sección por rol -->
    <section v-if="publicKey && state && (isAdmin || myRole)" class="section" style="margin-top:1.5rem">
      <h2 v-if="isAdmin">Resumen de administración</h2>
      <h2 v-else-if="myRole === 'operator'">Tus equipos bajo custodia</h2>
      <h2 v-else-if="myRole === 'sceneCommander'">Tus incidentes activos</h2>
      <h2 v-else-if="myRole === 'operationalBase'">Resumen de inventario</h2>

      <template v-if="isAdmin">
        <div class="card-grid" style="margin-bottom:1rem">
          <div class="card card-clickable" :class="{ 'card-active': adminSelected === 'operationalBase' }" @click="adminSelected = 'operationalBase'">
            <span class="label">Base Operativa</span><span class="value">{{ personnelRoleCounts.operationalBase }}</span>
          </div>
          <div class="card card-clickable" :class="{ 'card-active': adminSelected === 'sceneCommander' }" @click="adminSelected = 'sceneCommander'">
            <span class="label">Jefe de Escena</span><span class="value">{{ personnelRoleCounts.sceneCommander }}</span>
          </div>
          <div class="card card-clickable" :class="{ 'card-active': adminSelected === 'operator' }" @click="adminSelected = 'operator'">
            <span class="label">Operadores</span><span class="value">{{ personnelRoleCounts.operator }}</span>
          </div>
          <div class="card card-clickable" :class="{ 'card-active': adminSelected === 'incidents' }" @click="adminSelected = 'incidents'">
            <span class="label">Incidentes activos</span><span class="value">{{ activeIncidentsList.length }}</span>
          </div>
        </div>

        <template v-if="adminSelected === 'incidents'">
          <h3 class="subheading">Incidentes activos ({{ activeIncidentsList.length }})</h3>
          <div v-if="activeIncidentsList.length === 0" class="empty">Sin incidentes activos en este momento.</div>
          <div v-else class="table-wrap">
            <table>
              <thead><tr><th>ID</th><th>Descripción</th><th>Riesgo</th><th>Comandante</th></tr></thead>
              <tbody>
                <tr v-for="i in activeIncidentsList" :key="i.publicKey.toBase58()">
                  <td>#{{ i.account.incidentId.toString() }}</td>
                  <td>{{ i.account.description }}</td>
                  <td><span class="risk-badge" :data-level="i.account.riskLevel">{{ i.account.riskLevel }}/5</span></td>
                  <td>{{ personnelMap[i.account.commander?.toBase58?.()] ?? shortKey(i.account.commander) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
        <template v-else>
          <h3 class="subheading">{{ roleLabel(adminSelected) }} ({{ selectedPersonnelList.length }})</h3>
          <div v-if="selectedPersonnelList.length === 0" class="empty">Sin personal registrado en este rol.</div>
          <div v-else class="table-wrap">
            <table>
              <thead><tr><th>Nombre</th><th>Especialidad</th><th>Wallet</th></tr></thead>
              <tbody>
                <tr v-for="p in selectedPersonnelList" :key="p.publicKey.toBase58()">
                  <td>{{ p.account.name }}</td>
                  <td>{{ p.account.specialty }}</td>
                  <td class="mono">{{ shortKey(p.account.wallet) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </template>

      <template v-else-if="myRole === 'operator'">
        <p class="muted" style="margin-bottom:0.75rem">
          {{ myPersonnel.account.activeAssignments }} equipo(s) pendiente(s) de retornar
          <span v-if="myPersonnel.account.currentIncident"> — asignado al incidente #{{ myPersonnel.account.currentIncident.toString() }}</span>
        </p>
        <div v-if="myEquipment.length === 0" class="empty">No tienes equipos bajo custodia en este momento.</div>
        <div v-else class="table-wrap">
          <table>
            <thead><tr><th>Código</th><th>Descripción</th><th>Condición</th></tr></thead>
            <tbody>
              <tr v-for="e in myEquipment" :key="e.publicKey.toBase58()">
                <td class="mono">{{ codeToStr(e.account.code) }}</td>
                <td>{{ e.account.description }}</td>
                <td><span class="badge" :class="conditionClass(e.account.reportedCondition)">{{ formatCondition(e.account.reportedCondition) }}</span></td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>

      <template v-else-if="myRole === 'sceneCommander'">
        <div v-if="myIncidents.length === 0" class="empty">No tienes incidentes activos.</div>
        <div v-else class="table-wrap">
          <table>
            <thead><tr><th>ID</th><th>Descripción</th><th>Coordenadas</th><th>Riesgo</th></tr></thead>
            <tbody>
              <tr v-for="i in myIncidents" :key="i.publicKey.toBase58()">
                <td>#{{ i.account.incidentId.toString() }}</td>
                <td>{{ i.account.description }}</td>
                <td class="mono">{{ i.account.coordinates }}</td>
                <td><span class="risk-badge" :data-level="i.account.riskLevel">{{ i.account.riskLevel }}/5</span></td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>

      <template v-else-if="myRole === 'operationalBase'">
        <div class="card-grid" style="margin-bottom:1rem">
          <div class="card card-clickable" :class="{ 'card-active': baseSelected === 'available' }" @click="baseSelected = 'available'">
            <span class="label">Disponible</span><span class="badge badge-green">{{ equipmentStatusCounts.available }}</span>
          </div>
          <div class="card card-clickable" :class="{ 'card-active': baseSelected === 'inUse' }" @click="baseSelected = 'inUse'">
            <span class="label">En uso</span><span class="badge badge-yellow">{{ equipmentStatusCounts.inUse }}</span>
          </div>
          <div class="card card-clickable" :class="{ 'card-active': baseSelected === 'inRepair' }" @click="baseSelected = 'inRepair'">
            <span class="label">En reparación</span><span class="badge badge-red">{{ equipmentStatusCounts.inRepair }}</span>
          </div>
          <div class="card card-clickable" :class="{ 'card-active': baseSelected === 'lost' }" @click="baseSelected = 'lost'">
            <span class="label">Perdido</span><span class="badge badge-red">{{ equipmentStatusCounts.lost }}</span>
          </div>
          <div class="card card-clickable" :class="{ 'card-active': baseSelected === 'returning' }" @click="baseSelected = 'returning'">
            <span class="label">Retornando</span><span class="badge badge-yellow">{{ equipmentStatusCounts.returning }}</span>
          </div>
        </div>

        <div v-if="!baseSelected" class="empty">Haz clic en una tarjeta para ver el detalle.</div>
        <template v-else>
          <h3 class="subheading">{{ formatStatus({ [baseSelected]: {} }) }} ({{ selectedEquipmentList.length }})</h3>
          <div v-if="selectedEquipmentList.length === 0" class="empty">No hay equipos en este estado.</div>
          <div v-else class="table-wrap">
            <table>
              <thead><tr><th>Código</th><th>Descripción</th><th>Custodio</th></tr></thead>
              <tbody>
                <tr v-for="e in selectedEquipmentList" :key="e.publicKey.toBase58()">
                  <td class="mono">{{ codeToStr(e.account.code) }}</td>
                  <td>{{ e.account.description }}</td>
                  <td>
                    <span v-if="personnelMap[e.account.custodian?.toBase58?.()]">{{ personnelMap[e.account.custodian.toBase58()] }}</span>
                    <span v-else class="mono">{{ shortKey(e.account.custodian) }}</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </template>
    </section>

    <button class="btn-secondary" @click="refresh" style="margin-top:1.5rem">Actualizar</button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useProgram } from '../composables/useProgram'
import type { GlobalStateData } from '../composables/useProgram'
import { useWallet } from '../composables/useWallet'

const { fetchGlobalState, fetchAllPersonnel, fetchAllEquipment, fetchAllIncidents, PROGRAM_ID } = useProgram()
const { publicKey } = useWallet()

const state = ref<GlobalStateData | null>(null)
const loading = ref(false)

const allPersonnel = ref<any[]>([])
const allEquipment = ref<any[]>([])
const allIncidents = ref<any[]>([])

const shortAdmin = ref('')
const shortProgramId = PROGRAM_ID.toBase58().slice(0, 6) + '…' + PROGRAM_ID.toBase58().slice(-4)

const isAdmin = computed(() => {
  if (!publicKey.value || !state.value) return false
  return state.value.admin.toBase58() === publicKey.value.toBase58()
})

const myPersonnel = computed(() => {
  if (!publicKey.value) return null
  const pk = publicKey.value.toBase58()
  return allPersonnel.value.find(p => p.account.wallet.toBase58() === pk) ?? null
})

const myRole = computed(() => myPersonnel.value ? Object.keys(myPersonnel.value.account.role)[0] : null)

const myEquipment = computed(() => {
  if (!publicKey.value) return []
  const pk = publicKey.value.toBase58()
  return allEquipment.value.filter(e =>
    e.account.custodian?.toBase58?.() === pk && Object.keys(e.account.status)[0] === 'inUse'
  )
})

const myIncidents = computed(() => {
  if (!publicKey.value) return []
  const pk = publicKey.value.toBase58()
  return allIncidents.value.filter(i => i.account.commander?.toBase58?.() === pk && i.account.isActive)
})

const activeIncidentsList = computed(() => allIncidents.value.filter(i => i.account.isActive))

const adminSelected = ref<'operationalBase' | 'sceneCommander' | 'operator' | 'incidents'>('incidents')

const selectedPersonnelList = computed(() => {
  if (adminSelected.value === 'incidents') return []
  return allPersonnel.value.filter(p => Object.keys(p.account.role)[0] === adminSelected.value)
})

const baseSelected = ref<'available' | 'inUse' | 'inRepair' | 'lost' | 'returning' | null>(null)

const selectedEquipmentList = computed(() => {
  if (!baseSelected.value) return []
  return allEquipment.value.filter(e => Object.keys(e.account.status)[0] === baseSelected.value)
})

const personnelMap = computed<Record<string, string>>(() => {
  const map: Record<string, string> = {}
  for (const p of allPersonnel.value) {
    map[p.account.wallet.toBase58()] = p.account.name
  }
  return map
})

const equipmentStatusCounts = computed(() => {
  const counts: Record<string, number> = { available: 0, inUse: 0, inRepair: 0, lost: 0, returning: 0 }
  for (const e of allEquipment.value) {
    const k = Object.keys(e.account.status)[0]
    if (k in counts) counts[k]++
  }
  return counts
})

const personnelRoleCounts = computed(() => {
  const counts: Record<string, number> = { operationalBase: 0, sceneCommander: 0, operator: 0 }
  for (const p of allPersonnel.value) {
    const k = Object.keys(p.account.role)[0]
    if (k in counts) counts[k]++
  }
  return counts
})

async function refresh() {
  loading.value = true
  try {
    const [gs, pers, eq, inc] = await Promise.all([
      fetchGlobalState(), fetchAllPersonnel(), fetchAllEquipment(), fetchAllIncidents(),
    ])
    state.value = gs
    allPersonnel.value = pers
    allEquipment.value = eq
    allIncidents.value = inc
    if (state.value) {
      const a = state.value.admin.toBase58()
      shortAdmin.value = a.slice(0, 6) + '…' + a.slice(-4)
    }
  } catch {
    state.value = null
  } finally {
    loading.value = false
  }
}

onMounted(refresh)

// ── Formatters ────────────────────────────────────────────────────────────
function codeToStr(code: number[]): string {
  return String.fromCharCode(...code.filter(b => b !== 0))
}

function formatCondition(c: any): string {
  const map: Record<string, string> = {
    operational: 'Operacional', minorDamage: 'Daño menor',
    criticalDamage: 'Daño crítico', lost: 'Perdido',
  }
  return map[Object.keys(c)[0]] ?? Object.keys(c)[0]
}

function conditionClass(c: any): string {
  const map: Record<string, string> = {
    operational: 'badge-green', minorDamage: 'badge-yellow',
    criticalDamage: 'badge-red', lost: 'badge-red',
  }
  return map[Object.keys(c)[0]] ?? ''
}

function roleLabel(role: string): string {
  const map: Record<string, string> = {
    operationalBase: 'Base Operativa',
    sceneCommander: 'Jefe de Escena', operator: 'Operadores',
  }
  return map[role] ?? role
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

</script>
