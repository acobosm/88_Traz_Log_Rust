import { computed } from 'vue'
import { Connection, PublicKey } from '@solana/web3.js'
import { AnchorProvider, Program, BN } from '@coral-xyz/anchor'
import type { Idl } from '@coral-xyz/anchor'
import idlJson from '../idl/traz_log.json'
import { useWallet } from './useWallet'

const PROGRAM_ID = new PublicKey(idlJson.address)
const RPC_URL = import.meta.env.VITE_RPC_URL ?? 'http://localhost:8899'

export type GlobalStateData = {
  admin: PublicKey
  nextIncidentId: BN
  isPaused: boolean
  bump: number
}

function getProvider(wallet: ReturnType<typeof useWallet>['adapter']) {
  const connection = new Connection(RPC_URL, 'confirmed')
  return new AnchorProvider(connection, wallet as any, { commitment: 'confirmed' })
}

export function useProgram() {
  const { adapter, connected } = useWallet()

  const program = computed(() => {
    if (!connected.value) return null
    const provider = getProvider(adapter)
    return new Program(idlJson as Idl, provider)
  })

  // ── PDAs ──────────────────────────────────────────────────────────────────

  function globalStatePda(): PublicKey {
    return PublicKey.findProgramAddressSync([Buffer.from('global')], PROGRAM_ID)[0]
  }

  function personnelPda(wallet: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('personnel'), wallet.toBuffer()],
      PROGRAM_ID,
    )[0]
  }

  function equipmentPda(code: Uint8Array): PublicKey {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('equipment'), Buffer.from(code)],
      PROGRAM_ID,
    )[0]
  }

  function incidentPda(incidentId: number): PublicKey {
    const buf = Buffer.alloc(8)
    buf.writeBigUInt64LE(BigInt(incidentId))
    return PublicKey.findProgramAddressSync(
      [Buffer.from('incident'), buf],
      PROGRAM_ID,
    )[0]
  }

  // ── Lecturas ──────────────────────────────────────────────────────────────

  async function fetchGlobalState(): Promise<GlobalStateData | null> {
    if (!program.value) return null
    const pda = globalStatePda()
    return (program.value.account as any).globalState.fetch(pda)
  }

  async function fetchIncident(incidentId: number) {
    if (!program.value) return null
    const pda = incidentPda(incidentId)
    return (program.value.account as any).incidentAccount.fetch(pda)
  }

  async function fetchEquipment(code: Uint8Array) {
    if (!program.value) return null
    const pda = equipmentPda(code)
    return (program.value.account as any).equipmentAccount.fetch(pda)
  }

  // ── Instrucciones ─────────────────────────────────────────────────────────

  async function togglePause() {
    if (!program.value) throw new Error('No conectado')
    return (program.value.methods as any)
      .togglePause()
      .accounts({ globalState: globalStatePda() })
      .rpc()
  }

  async function registerPersonnel(
    wallet: PublicKey,
    name: string,
    specialty: string,
    role: object,
  ) {
    if (!program.value) throw new Error('No conectado')
    return (program.value.methods as any)
      .registerPersonnel(name, specialty, role)
      .accounts({
        globalState: globalStatePda(),
        newPersonnel: personnelPda(wallet),
        wallet,
      })
      .rpc()
  }

  async function openFireIncident(
    incidentId: number,
    coordinates: string,
    riskLevel: number,
  ) {
    if (!program.value) throw new Error('No conectado')
    const { publicKey: pk } = useWallet()
    if (!pk.value) throw new Error('Sin wallet')
    return (program.value.methods as any)
      .openFireIncident(new BN(incidentId), coordinates, riskLevel)
      .accounts({
        globalState: globalStatePda(),
        signerPersonnel: personnelPda(pk.value),
        incident: incidentPda(incidentId),
      })
      .rpc()
  }

  async function closeIncident(incidentId: number) {
    if (!program.value) throw new Error('No conectado')
    const { publicKey: pk } = useWallet()
    if (!pk.value) throw new Error('Sin wallet')
    return (program.value.methods as any)
      .closeIncident(new BN(incidentId))
      .accounts({
        globalState: globalStatePda(),
        signerPersonnel: personnelPda(pk.value),
        incident: incidentPda(incidentId),
      })
      .rpc()
  }

  return {
    program,
    PROGRAM_ID,
    globalStatePda,
    personnelPda,
    equipmentPda,
    incidentPda,
    fetchGlobalState,
    fetchIncident,
    fetchEquipment,
    togglePause,
    registerPersonnel,
    openFireIncident,
    closeIncident,
  }
}
