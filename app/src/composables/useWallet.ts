import { ref, computed } from 'vue'
import { PhantomWalletAdapter } from '@solana/wallet-adapter-phantom'
import type { PublicKey } from '@solana/web3.js'

const adapter = new PhantomWalletAdapter()
const connected = ref(false)
const publicKey = ref<PublicKey | null>(null)

adapter.on('connect', (pk: PublicKey) => {
  connected.value = true
  publicKey.value = pk
})

adapter.on('disconnect', () => {
  connected.value = false
  publicKey.value = null
})

export function useWallet() {
  const shortAddress = computed(() => {
    if (!publicKey.value) return ''
    const s = publicKey.value.toBase58()
    return `${s.slice(0, 4)}…${s.slice(-4)}`
  })

  async function connect() {
    await adapter.connect()
  }

  async function disconnect() {
    await adapter.disconnect()
  }

  return { adapter, connected, publicKey, shortAddress, connect, disconnect }
}
