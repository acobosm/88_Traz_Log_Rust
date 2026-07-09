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

export function toCode(s: string): number[] {
  const arr = new Array<number>(32).fill(0)
  const bytes = new TextEncoder().encode(s)
  bytes.slice(0, 32).forEach((b, i) => { arr[i] = b })
  return arr
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

  function equipmentPda(code: number[]): PublicKey {
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
    return (program.value.account as any).globalState.fetch(globalStatePda())
  }

  async function fetchIncident(incidentId: number) {
    if (!program.value) return null
    return (program.value.account as any).incidentAccount.fetch(incidentPda(incidentId))
  }

  async function fetchEquipment(code: number[]) {
    if (!program.value) return null
    return (program.value.account as any).equipmentAccount.fetch(equipmentPda(code))
  }

  async function fetchLogEntries(code: number[]): Promise<{ publicKey: PublicKey; account: any }[]> {
    if (!program.value) return []
    const codeStr = String.fromCharCode(...code.filter(b => b !== 0))
    const all = await (program.value.account as any).logEntry.all()
    return (all as any[])
      .filter(e => String.fromCharCode(...e.account.equipmentCode.filter((b: number) => b !== 0)) === codeStr)
      .sort((a, b) => Number(a.account.entryIndex.toString()) - Number(b.account.entryIndex.toString()))
  }

  async function fetchAllPersonnel(): Promise<{ publicKey: PublicKey; account: any }[]> {
    if (!program.value) return []
    return (program.value.account as any).personnelAccount.all()
  }

  async function fetchAllEquipment(): Promise<{ publicKey: PublicKey; account: any }[]> {
    if (!program.value) return []
    return (program.value.account as any).equipmentAccount.all()
  }

  async function fetchAllIncidents(): Promise<{ publicKey: PublicKey; account: any }[]> {
    if (!program.value) return []
    return (program.value.account as any).incidentAccount.all()
  }

  // ── Instrucciones ─────────────────────────────────────────────────────────

  async function initialize() {
    if (!program.value) throw new Error('No conectado')
    const { publicKey: pk } = useWallet()
    if (!pk.value) throw new Error('Sin wallet')
    return (program.value.methods as any)
      .initialize()
      .accounts({ admin: pk.value, globalState: globalStatePda() })
      .rpc()
  }

  async function togglePause() {
    if (!program.value) throw new Error('No conectado')
    return (program.value.methods as any)
      .togglePause()
      .accounts({ globalState: globalStatePda() })
      .rpc()
  }

  async function registerPersonnel(wallet: PublicKey, name: string, specialty: string, role: object) {
    if (!program.value) throw new Error('No conectado')
    return (program.value.methods as any)
      .registerPersonnel(name, specialty, role)
      .accounts({ globalState: globalStatePda(), newPersonnel: personnelPda(wallet), wallet })
      .rpc()
  }

  async function registerEquipment(code: number[], description: string, nominalConsumption: number) {
    if (!program.value) throw new Error('No conectado')
    const { publicKey: pk } = useWallet()
    if (!pk.value) throw new Error('Sin wallet')
    return (program.value.methods as any)
      .registerEquipment(code, description, new BN(nominalConsumption))
      .accounts({
        globalState: globalStatePda(),
        signerPersonnel: personnelPda(pk.value),
        equipment: equipmentPda(code),
      })
      .rpc()
  }

  async function openFireIncident(incidentId: number, description: string, coordinates: string, riskLevel: number) {
    if (!program.value) throw new Error('No conectado')
    const { publicKey: pk } = useWallet()
    if (!pk.value) throw new Error('Sin wallet')
    return (program.value.methods as any)
      .openFireIncident(new BN(incidentId), description, coordinates, riskLevel)
      .accounts({
        globalState: globalStatePda(),
        signerPersonnel: personnelPda(pk.value),
        incident: incidentPda(incidentId),
      })
      .rpc()
  }

  async function assignEquipment(code: number[], incidentId: number, operatorWallet: PublicKey) {
    if (!program.value) throw new Error('No conectado')
    const { publicKey: pk } = useWallet()
    if (!pk.value) throw new Error('Sin wallet')
    return (program.value.methods as any)
      .assignEquipment(code, new BN(incidentId))
      .accounts({
        globalState: globalStatePda(),
        signerPersonnel: personnelPda(pk.value),
        equipment: equipmentPda(code),
        incident: incidentPda(incidentId),
        operatorPersonnel: personnelPda(operatorWallet),
        operatorWallet,
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

  async function logMilestone(code: number[], notes: string, condition: object) {
    if (!program.value) throw new Error('No conectado')
    const { publicKey: pk } = useWallet()
    if (!pk.value) throw new Error('Sin wallet')
    return (program.value.methods as any)
      .logMilestone(code, notes, condition)
      .accounts({
        globalState: globalStatePda(),
        signerPersonnel: personnelPda(pk.value),
        equipment: equipmentPda(code),
      })
      .rpc()
  }

  async function initiateReturn(code: number[]) {
    if (!program.value) throw new Error('No conectado')
    const { publicKey: pk } = useWallet()
    if (!pk.value) throw new Error('Sin wallet')
    return (program.value.methods as any)
      .initiateReturn(code)
      .accounts({
        globalState: globalStatePda(),
        signerPersonnel: personnelPda(pk.value),
        equipment: equipmentPda(code),
      })
      .rpc()
  }

  return {
    program,
    PROGRAM_ID,
    globalStatePda,
    initialize,
    personnelPda,
    equipmentPda,
    incidentPda,
    fetchGlobalState,
    fetchIncident,
    fetchEquipment,
    fetchAllPersonnel,
    fetchAllEquipment,
    fetchAllIncidents,
    fetchLogEntries,
    togglePause,
    registerPersonnel,
    registerEquipment,
    openFireIncident,
    assignEquipment,
    closeIncident,
    logMilestone,
    initiateReturn,
  }
}
