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

    <button class="btn-secondary" @click="refresh">Actualizar</button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useProgram } from '../composables/useProgram'
import type { GlobalStateData } from '../composables/useProgram'

const { fetchGlobalState, PROGRAM_ID } = useProgram()

const state = ref<GlobalStateData | null>(null)
const loading = ref(false)

const shortAdmin = ref('')
const shortProgramId = PROGRAM_ID.toBase58().slice(0, 6) + '…' + PROGRAM_ID.toBase58().slice(-4)

async function refresh() {
  loading.value = true
  try {
    state.value = await fetchGlobalState()
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
</script>
