<template>
  <div id="app">
    <header>
      <div class="brand">
        <span class="brand-icon">🔥</span>
        <span class="brand-name">FireOPS TrazLog</span>
      </div>

      <nav>
        <button
          v-for="tab in tabs"
          :key="tab.id"
          :class="['nav-btn', { active: currentTab === tab.id }]"
          @click="currentTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </nav>

      <div class="wallet-area">
        <button v-if="!connected" class="btn-connect" @click="connect">
          Conectar Phantom
        </button>
        <div v-else class="wallet-info">
          <span class="wallet-address">{{ shortAddress }}</span>
          <button class="btn-disconnect" @click="disconnect">✕</button>
        </div>
      </div>
    </header>

    <main>
      <div v-if="!connected" class="connect-prompt">
        <p>Conecta tu wallet Phantom para interactuar con el programa.</p>
      </div>
      <template v-else>
        <DashboardView   v-if="currentTab === 'dashboard'" />
        <InventarioView  v-else-if="currentTab === 'inventario'" />
        <AdminView       v-else-if="currentTab === 'admin'" />
        <IncidenteView   v-else-if="currentTab === 'incidente'" />
        <CampoView       v-else-if="currentTab === 'campo'" />
      </template>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useWallet } from './composables/useWallet'
import DashboardView from './views/DashboardView.vue'
import AdminView from './views/AdminView.vue'
import IncidenteView from './views/IncidenteView.vue'
import CampoView from './views/CampoView.vue'
import InventarioView from './views/InventarioView.vue'

const { connected, shortAddress, connect, disconnect } = useWallet()

const currentTab = ref('dashboard')
const tabs = [
  { id: 'dashboard',  label: 'Dashboard' },
  { id: 'inventario', label: 'Inventario' },
  { id: 'incidente',  label: 'Incidente' },
  { id: 'campo',      label: 'Campo' },
  { id: 'admin',      label: 'Admin' },
]
</script>
