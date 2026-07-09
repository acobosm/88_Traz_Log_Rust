// Seed manual de datos de prueba para localnet.
// Requiere: solana-test-validator / anchor localnet corriendo en localhost:8899
// Uso: node scripts/seed.mjs

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { Connection, Keypair, PublicKey } from '@solana/web3.js'
import { AnchorProvider, Program, BN, utils } from '@coral-xyz/anchor'

const __dirname = dirname(fileURLToPath(import.meta.url))
const RPC_URL = process.env.VITE_RPC_URL ?? 'http://localhost:8899'

const idl = JSON.parse(readFileSync(join(__dirname, '../src/idl/traz_log.json'), 'utf8'))
const PROGRAM_ID = new PublicKey(idl.address)

// ── Wallets ──────────────────────────────────────────────────────────────────

function loadAdminKeypair() {
  const raw = JSON.parse(readFileSync(join(process.env.HOME, '.config/solana/id.json'), 'utf8'))
  return Keypair.fromSecretKey(Uint8Array.from(raw))
}

function loadRoleKeypair(name) {
  const secrets = JSON.parse(readFileSync(join(__dirname, 'seed-keys.local.json'), 'utf8'))
  const decoded = utils.bytes.bs58.decode(secrets[name])
  return Keypair.fromSecretKey(decoded)
}

const admin = loadAdminKeypair()
const base1 = loadRoleKeypair('base1')
const base2 = loadRoleKeypair('base2')
const jefe1 = loadRoleKeypair('jefe1')
const jefe2 = loadRoleKeypair('jefe2')

// Operadores: solo necesitan pubkey, no firman nada en este seed.
const operador1 = new PublicKey('5d5ETPWeQnsu5p6STTCzUZbPXXay6Cs3kcBUKcW8a8PL')
const operador2 = new PublicKey('CwHuZknct1CNXBarMbLxRcs9TdNqkn8hrkHnFe4vQgYv')

function nodeWallet(keypair) {
  return {
    publicKey: keypair.publicKey,
    async signTransaction(tx) {
      tx.partialSign ? tx.partialSign(keypair) : tx.sign([keypair])
      return tx
    },
    async signAllTransactions(txs) {
      for (const tx of txs) tx.partialSign ? tx.partialSign(keypair) : tx.sign([keypair])
      return txs
    },
  }
}

function programAs(keypair) {
  const connection = new Connection(RPC_URL, 'confirmed')
  const provider = new AnchorProvider(connection, nodeWallet(keypair), { commitment: 'confirmed' })
  return new Program(idl, provider)
}

// ── PDAs (mismas derivaciones que useProgram.ts) ────────────────────────────

function toCode(s) {
  const arr = new Array(32).fill(0)
  Buffer.from(s, 'utf8').slice(0, 32).forEach((b, i) => { arr[i] = b })
  return arr
}

function globalStatePda() {
  return PublicKey.findProgramAddressSync([Buffer.from('global')], PROGRAM_ID)[0]
}
function personnelPda(wallet) {
  return PublicKey.findProgramAddressSync([Buffer.from('personnel'), wallet.toBuffer()], PROGRAM_ID)[0]
}
function equipmentPda(code) {
  return PublicKey.findProgramAddressSync([Buffer.from('equipment'), Buffer.from(code)], PROGRAM_ID)[0]
}
function incidentPda(incidentId) {
  const buf = Buffer.alloc(8)
  buf.writeBigUInt64LE(BigInt(incidentId))
  return PublicKey.findProgramAddressSync([Buffer.from('incident'), buf], PROGRAM_ID)[0]
}

// Ejecuta una tx y no falla el seed completo si la cuenta ya existía (rerun idempotente).
async function run(label, fn) {
  try {
    await fn()
    console.log(`OK   ${label}`)
  } catch (e) {
    const msg = e.message ?? String(e)
    if (msg.includes('already in use') || msg.includes('custom program error: 0x0')) {
      console.log(`SKIP ${label} (ya existe)`)
    } else {
      console.error(`FAIL ${label}:`, msg)
    }
  }
}

// ── Seed ─────────────────────────────────────────────────────────────────────

async function main() {
  const adminProgram = programAs(admin)

  // 1. Initialize (si hace falta)
  try {
    await adminProgram.account.globalState.fetch(globalStatePda())
    console.log('SKIP initialize (ya existe)')
  } catch {
    await run('initialize', () =>
      adminProgram.methods.initialize()
        .accounts({ admin: admin.publicKey, globalState: globalStatePda() })
        .rpc()
    )
  }

  // 2. Personal
  const personnel = [
    { wallet: base1.publicKey, name: 'Base Operativa 1', specialty: 'Base Operativa Norte', role: 'operationalBase' },
    { wallet: base2.publicKey, name: 'Base Operativa 2', specialty: 'Base Operativa Sur', role: 'operationalBase' },
    { wallet: jefe1.publicKey, name: 'Jefe de Escena 01', specialty: 'Logistica', role: 'sceneCommander' },
    { wallet: jefe2.publicKey, name: 'Jefe de Escena 02', specialty: 'Logística', role: 'sceneCommander' },
    { wallet: operador1, name: 'Operador 01', specialty: 'Rescate', role: 'operator' },
    { wallet: operador2, name: 'Operador 02', specialty: 'Rescate', role: 'operator' },
  ]
  for (const p of personnel) {
    await run(`registerPersonnel ${p.name}`, () =>
      adminProgram.methods.registerPersonnel(p.name, p.specialty, { [p.role]: {} })
        .accounts({ globalState: globalStatePda(), newPersonnel: personnelPda(p.wallet), wallet: p.wallet })
        .rpc()
    )
  }

  // 3. Equipamiento (firmado por la base operativa correspondiente)
  const equipment = [
    { code: 'BOMBA-01', description: 'Motobomba 01', consumption: 4000, signer: base1 },
    { code: 'BOMBA-02', description: 'Motobomba 02', consumption: 4000, signer: base2 },
    { code: 'GPS-01', description: 'GPS Garmin 01', consumption: 0, signer: base1 },
    { code: 'GPS-02', description: 'GPS Garmin 02', consumption: 0, signer: base2 },
    { code: 'RADIO-01', description: 'Radio Motorola 01', consumption: 0, signer: base1 },
    { code: 'RADIO-02', description: 'Radio Motorola 02', consumption: 0, signer: base2 },
    { code: 'BATEFUEGOS-01', description: 'BateFuegos 01', consumption: 0, signer: base1 },
    { code: 'BATEFUEGOS-02', description: 'BateFuegos 02', consumption: 0, signer: base2 },
    { code: 'VEHICULO4X4-01', description: 'Vehículo 4x4 01', consumption: 0, signer: base1 },
    { code: 'CISTERNA-01', description: 'Camión Cisterna 01', consumption: 0, signer: base2 },
  ]
  for (const eq of equipment) {
    const program = programAs(eq.signer)
    const code = toCode(eq.code)
    await run(`registerEquipment ${eq.code}`, () =>
      program.methods.registerEquipment(code, eq.description, new BN(eq.consumption))
        .accounts({
          globalState: globalStatePda(),
          signerPersonnel: personnelPda(eq.signer.publicKey),
          equipment: equipmentPda(code),
        })
        .rpc()
    )
  }

  // 4. Incidentes (firmados por el jefe de escena correspondiente)
  const globalState = await adminProgram.account.globalState.fetch(globalStatePda())
  let nextId = Number(globalState.nextIncidentId.toString())

  const incidents = [
    { coordinates: '-0.22427525812065774, -78.49952508364493', riskLevel: 3, signer: jefe1 },
    { coordinates: '-0.20215294825985808, -78.47531785829314', riskLevel: 1, signer: jefe2 },
  ]
  for (const inc of incidents) {
    const program = programAs(inc.signer)
    const id = nextId
    await run(`openFireIncident #${id}`, async () => {
      await program.methods.openFireIncident(new BN(id), inc.coordinates, inc.riskLevel)
        .accounts({
          globalState: globalStatePda(),
          signerPersonnel: personnelPda(inc.signer.publicKey),
          incident: incidentPda(id),
        })
        .rpc()
      nextId += 1
    })
  }

  console.log('\nSeed completo.')
}

main().catch((e) => {
  console.error('Error fatal en el seed:', e)
  process.exit(1)
})
