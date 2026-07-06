<template>
  <div class="view">

    <!-- Log Milestone -->
    <section class="section">
      <h2>Reportar condición de equipo</h2>
      <form class="form-grid" @submit.prevent="handleLogMilestone">
        <label>Código del equipo
          <input v-model="milestone.code" placeholder="Ej: BOMBA-01" maxlength="32" required />
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
      <form class="form-grid" @submit.prevent="handleInitiateReturn">
        <label>Código del equipo
          <input v-model="ret.code" placeholder="Ej: BOMBA-01" maxlength="32" required />
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
        <label>Código del equipo
          <input v-model="query.code" placeholder="Ej: BOMBA-01" maxlength="32" required />
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
          <span class="value">{{ formatStatus(equipmentData.status) }}</span>
        </div>
        <div class="card">
          <span class="label">Condición</span>
          <span class="value">{{ formatCondition(equipmentData.reportedCondition) }}</span>
        </div>
        <div class="card">
          <span class="label">Custodio</span>
          <span class="value mono">{{ shortKey(equipmentData.custodian) }}</span>
        </div>
        <div class="card">
          <span class="label">Incidente</span>
          <span class="value">{{ equipmentData.incidentId.toString() }}</span>
        </div>
      </div>
      <p v-if="feedback.query" :class="feedbackClass(feedback.query)">{{ feedback.query }}</p>
    </section>

  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useProgram, toCode } from '../composables/useProgram'

const { logMilestone, initiateReturn, fetchEquipment } = useProgram()

const busy = reactive({ milestone: false, ret: false, query: false })
const feedback = reactive({ milestone: '', ret: '', query: '' })

const milestone = reactive({ code: '', condition: 'operational', notes: '' })
const ret = reactive({ code: '' })
const query = reactive({ code: '' })
const equipmentData = ref<any>(null)

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

async function handleLogMilestone() {
  busy.milestone = true; feedback.milestone = ''
  try {
    const conditionVariant = { [milestone.condition]: {} }
    await logMilestone(toCode(milestone.code), milestone.notes, conditionVariant)
    feedback.milestone = '✓ Condición reportada'
    milestone.code = ''; milestone.notes = ''
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
  } catch (e: any) {
    feedback.ret = '✗ ' + (e.message ?? e)
  } finally { busy.ret = false }
}

async function handleFetchEquipment() {
  busy.query = true; feedback.query = ''; equipmentData.value = null
  try {
    equipmentData.value = await fetchEquipment(toCode(query.code))
    if (!equipmentData.value) feedback.query = '✗ Equipo no encontrado'
  } catch {
    feedback.query = '✗ Equipo no encontrado'
  } finally { busy.query = false }
}
</script>
