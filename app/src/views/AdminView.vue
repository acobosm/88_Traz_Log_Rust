<template>
  <div class="view">

    <!-- Toggle Pause -->
    <section class="section">
      <h2>Estado del sistema</h2>
      <div class="row-actions">
        <button class="btn-primary" :disabled="busy.pause" @click="handleTogglePause">
          {{ busy.pause ? 'Enviando…' : 'Toggle Pause' }}
        </button>
        <span v-if="feedback.pause" :class="feedbackClass(feedback.pause)">{{ feedback.pause }}</span>
      </div>
    </section>

    <!-- Registrar Personal -->
    <section class="section">
      <h2>Registrar personal</h2>
      <form class="form-grid" @submit.prevent="handleRegisterPersonnel">
        <label>Wallet del miembro
          <input v-model="personnel.wallet" placeholder="Pubkey (base58)" required />
        </label>
        <label>Nombre
          <input v-model="personnel.name" placeholder="Nombre completo" required />
        </label>
        <label>Especialidad
          <input v-model="personnel.specialty" placeholder="Ej: Rescate, Logística" required />
        </label>
        <label>Rol
          <select v-model="personnel.role">
            <option value="admin">Admin</option>
            <option value="operationalBase">OperationalBase</option>
            <option value="sceneCommander">SceneCommander</option>
            <option value="operator">Operator</option>
          </select>
        </label>
        <div class="form-footer">
          <button type="submit" class="btn-primary" :disabled="busy.personnel">
            {{ busy.personnel ? 'Enviando…' : 'Registrar' }}
          </button>
          <span v-if="feedback.personnel" :class="feedbackClass(feedback.personnel)">{{ feedback.personnel }}</span>
        </div>
      </form>
    </section>

    <!-- Registrar Equipo -->
    <section class="section">
      <h2>Registrar equipo</h2>
      <form class="form-grid" @submit.prevent="handleRegisterEquipment">
        <label>Código (max 32 caracteres)
          <input v-model="equipment.code" placeholder="Ej: BOMBA-01" maxlength="32" required />
        </label>
        <label>Descripción
          <input v-model="equipment.description" placeholder="Descripción del equipo" required />
        </label>
        <label>Consumo nominal (L/h)
          <input v-model.number="equipment.consumption" type="number" min="0" required />
        </label>
        <div class="form-footer">
          <button type="submit" class="btn-primary" :disabled="busy.equipment">
            {{ busy.equipment ? 'Enviando…' : 'Registrar' }}
          </button>
          <span v-if="feedback.equipment" :class="feedbackClass(feedback.equipment)">{{ feedback.equipment }}</span>
        </div>
      </form>
    </section>

  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import { PublicKey } from '@solana/web3.js'
import { useProgram, toCode } from '../composables/useProgram'

const { togglePause, registerPersonnel, registerEquipment } = useProgram()

const busy = reactive({ pause: false, personnel: false, equipment: false })
const feedback = reactive({ pause: '', personnel: '', equipment: '' })

const personnel = reactive({ wallet: '', name: '', specialty: '', role: 'operator' })
const equipment = reactive({ code: '', description: '', consumption: 0 })

function feedbackClass(msg: string) {
  return msg.startsWith('✓') ? 'feedback-ok' : 'feedback-err'
}

async function handleTogglePause() {
  busy.pause = true; feedback.pause = ''
  try {
    await togglePause()
    feedback.pause = '✓ Transacción enviada'
  } catch (e: any) {
    feedback.pause = '✗ ' + (e.message ?? e)
  } finally { busy.pause = false }
}

async function handleRegisterPersonnel() {
  busy.personnel = true; feedback.personnel = ''
  try {
    const wallet = new PublicKey(personnel.wallet)
    const roleVariant = { [personnel.role]: {} }
    await registerPersonnel(wallet, personnel.name, personnel.specialty, roleVariant)
    feedback.personnel = '✓ Personal registrado'
    personnel.wallet = ''; personnel.name = ''; personnel.specialty = ''
  } catch (e: any) {
    feedback.personnel = '✗ ' + (e.message ?? e)
  } finally { busy.personnel = false }
}

async function handleRegisterEquipment() {
  busy.equipment = true; feedback.equipment = ''
  try {
    await registerEquipment(toCode(equipment.code), equipment.description, equipment.consumption)
    feedback.equipment = '✓ Equipo registrado'
    equipment.code = ''; equipment.description = ''; equipment.consumption = 0
  } catch (e: any) {
    feedback.equipment = '✗ ' + (e.message ?? e)
  } finally { busy.equipment = false }
}
</script>
