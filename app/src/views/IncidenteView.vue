<template>
  <div class="view">

    <!-- Abrir incidente -->
    <section class="section">
      <h2>Abrir incidente</h2>
      <form class="form-grid" @submit.prevent="handleOpenIncident">
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
      <form class="form-grid" @submit.prevent="handleAssignEquipment">
        <label>Código del equipo
          <input v-model="assign.code" placeholder="Ej: BOMBA-01" maxlength="32" required />
        </label>
        <label>ID de incidente
          <input v-model.number="assign.incidentId" type="number" min="0" required />
        </label>
        <label>Wallet del operador
          <input v-model="assign.operatorWallet" placeholder="Pubkey (base58)" required />
        </label>
        <div class="form-footer">
          <button type="submit" class="btn-primary" :disabled="busy.assign">
            {{ busy.assign ? 'Enviando…' : 'Asignar' }}
          </button>
          <span v-if="feedback.assign" :class="feedbackClass(feedback.assign)">{{ feedback.assign }}</span>
        </div>
      </form>
    </section>

    <!-- Cerrar incidente -->
    <section class="section">
      <h2>Cerrar incidente</h2>
      <form class="form-grid" @submit.prevent="handleCloseIncident">
        <label>ID de incidente
          <input v-model.number="close.incidentId" type="number" min="0" required />
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
import { reactive } from 'vue'
import { PublicKey } from '@solana/web3.js'
import { useProgram, toCode } from '../composables/useProgram'

const { fetchGlobalState, openFireIncident, assignEquipment, closeIncident } = useProgram()

const busy = reactive({ open: false, assign: false, close: false })
const feedback = reactive({ open: '', assign: '', close: '' })

const open = reactive({ coordinates: '', riskLevel: 3 })
const assign = reactive({ code: '', incidentId: 0, operatorWallet: '' })
const close = reactive({ incidentId: 0 })

function feedbackClass(msg: string) {
  return msg.startsWith('✓') ? 'feedback-ok' : 'feedback-err'
}

async function handleOpenIncident() {
  busy.open = true; feedback.open = ''
  try {
    const gs = await fetchGlobalState()
    const incidentId = gs ? Number(gs.nextIncidentId.toString()) : 0
    await openFireIncident(incidentId, open.coordinates, open.riskLevel)
    feedback.open = `✓ Incidente #${incidentId} abierto`
    open.coordinates = ''
  } catch (e: any) {
    feedback.open = '✗ ' + (e.message ?? e)
  } finally { busy.open = false }
}

async function handleAssignEquipment() {
  busy.assign = true; feedback.assign = ''
  try {
    const operatorWallet = new PublicKey(assign.operatorWallet)
    await assignEquipment(toCode(assign.code), assign.incidentId, operatorWallet)
    feedback.assign = '✓ Equipo asignado'
    assign.code = ''; assign.operatorWallet = ''
  } catch (e: any) {
    feedback.assign = '✗ ' + (e.message ?? e)
  } finally { busy.assign = false }
}

async function handleCloseIncident() {
  busy.close = true; feedback.close = ''
  try {
    await closeIncident(close.incidentId)
    feedback.close = `✓ Incidente #${close.incidentId} cerrado`
  } catch (e: any) {
    feedback.close = '✗ ' + (e.message ?? e)
  } finally { busy.close = false }
}
</script>
