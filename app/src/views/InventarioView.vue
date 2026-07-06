<template>
  <div class="view">

    <div class="inv-header">
      <span class="inv-counts" v-if="!loading">
        {{ personnel.length }} personas · {{ equipment.length }} equipos · {{ incidents.length }} incidentes
      </span>
      <button class="btn-secondary" :disabled="loading" @click="refresh">
        {{ loading ? 'Cargando…' : 'Actualizar' }}
      </button>
    </div>

    <!-- Personal -->
    <section class="section">
      <h2>Personal registrado ({{ personnel.length }})</h2>
      <div v-if="personnel.length === 0" class="empty">Sin registros.</div>
      <div v-else class="table-wrap">
        <table>
          <thead>
            <tr><th>Nombre</th><th>Rol</th><th>Especialidad</th><th>Activo</th><th>Wallet</th></tr>
          </thead>
          <tbody>
            <tr v-for="p in personnel" :key="p.publicKey.toBase58()">
              <td>{{ p.account.name }}</td>
              <td><span class="badge-role">{{ formatRole(p.account.role) }}</span></td>
              <td>{{ p.account.specialty }}</td>
              <td>{{ p.account.isActive ? '✓' : '✗' }}</td>
              <td class="mono">{{ shortKey(p.account.wallet) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <!-- Equipos -->
    <section class="section">
      <h2>Equipos registrados ({{ equipment.length }})</h2>
      <div v-if="equipment.length === 0" class="empty">Sin registros.</div>
      <div v-else class="table-wrap">
        <table>
          <thead>
            <tr><th>Código</th><th>Descripción</th><th>Estado</th><th>Condición</th><th>Consumo (L/h)</th><th>Custodio</th></tr>
          </thead>
          <tbody>
            <tr v-for="e in equipment" :key="e.publicKey.toBase58()">
              <td class="mono">{{ codeToStr(e.account.code) }}</td>
              <td>{{ e.account.description }}</td>
              <td><span :class="statusClass(e.account.status)">{{ formatStatus(e.account.status) }}</span></td>
              <td>{{ formatCondition(e.account.reportedCondition) }}</td>
              <td>{{ e.account.nominalConsumption.toString() }}</td>
              <td class="mono">{{ shortKey(e.account.custodian) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <!-- Incidentes -->
    <section class="section">
      <h2>Incidentes ({{ incidents.length }})</h2>
      <div v-if="incidents.length === 0" class="empty">Sin incidentes registrados.</div>
      <div v-else class="table-wrap">
        <table>
          <thead>
            <tr><th>ID</th><th>Coordenadas</th><th>Riesgo</th><th>Estado</th><th>Comandante</th></tr>
          </thead>
          <tbody>
            <tr v-for="i in incidentsSorted" :key="i.publicKey.toBase58()">
              <td>#{{ i.account.incidentId.toString() }}</td>
              <td>{{ i.account.coordinates }}</td>
              <td>
                <span class="risk-badge" :data-level="i.account.riskLevel">
                  {{ i.account.riskLevel }}/5
                </span>
              </td>
              <td>
                <span :class="i.account.isActive ? 'badge-green' : 'badge-red'">
                  {{ i.account.isActive ? 'Activo' : 'Cerrado' }}
                </span>
              </td>
              <td class="mono">{{ shortKey(i.account.commander) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useProgram } from '../composables/useProgram'

const { fetchAllPersonnel, fetchAllEquipment, fetchAllIncidents } = useProgram()

const personnel = ref<any[]>([])
const equipment = ref<any[]>([])
const incidents = ref<any[]>([])
const loading = ref(false)

const incidentsSorted = computed(() =>
  [...incidents.value].sort((a, b) =>
    Number(a.account.incidentId.toString()) - Number(b.account.incidentId.toString())
  )
)

async function refresh() {
  loading.value = true
  try {
    const [p, e, i] = await Promise.all([
      fetchAllPersonnel(),
      fetchAllEquipment(),
      fetchAllIncidents(),
    ])
    personnel.value = p
    equipment.value = e
    incidents.value = i
  } catch (err) {
    console.error('Error cargando inventario:', err)
  } finally {
    loading.value = false
  }
}

onMounted(refresh)

// ── Formatters ────────────────────────────────────────────────────────────────

function shortKey(pk: any): string {
  const s = pk?.toBase58?.() ?? '—'
  if (s === '11111111111111111111111111111111') return '(sin asignar)'
  return `${s.slice(0, 4)}…${s.slice(-4)}`
}

function codeToStr(code: number[]): string {
  return String.fromCharCode(...code.filter(b => b !== 0))
}

function formatRole(r: any): string {
  const k = Object.keys(r)[0]
  const map: Record<string, string> = {
    admin: 'Admin', operationalBase: 'Base Op.',
    sceneCommander: 'Jefe Escena', operator: 'Operador',
  }
  return map[k] ?? k
}

function formatStatus(s: any): string {
  const k = Object.keys(s)[0]
  const map: Record<string, string> = {
    available: 'Disponible', inUse: 'En uso',
    inRepair: 'Reparación', lost: 'Perdido', returning: 'Retornando',
  }
  return map[k] ?? k
}

function statusClass(s: any): string {
  const k = Object.keys(s)[0]
  const map: Record<string, string> = {
    available: 'badge-green', inUse: 'badge-yellow',
    inRepair: 'badge-red', lost: 'badge-red', returning: 'badge-yellow',
  }
  return map[k] ?? ''
}

function formatCondition(c: any): string {
  const k = Object.keys(c)[0]
  const map: Record<string, string> = {
    operational: 'Operacional', minorDamage: 'Daño menor',
    criticalDamage: 'Daño crítico', lost: 'Perdido',
  }
  return map[k] ?? k
}
</script>
