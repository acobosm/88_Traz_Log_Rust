# FireOPS — Sistema de Trazabilidad Logística (Rust/Anchor)

Port del sistema FireOPS de Solidity/Ethereum a **Rust/Anchor en Solana**.  
Sistema de cadena de custodia para operaciones de combate de incendios forestales: gestiona personal, equipos, incidentes y retorno de recursos con firma digital.

---

## Requisitos previos

| Herramienta | Versión | Notas |
|---|---|---|
| Rust | 1.75+ | vía rustup |
| Solana CLI (Agave) | 3.1.10 | cliente de Solana |
| Anchor CLI | 1.0.2 | framework de smart contracts |
| Node.js | 18+ | para tests TypeScript y frontend |
| npm | 9+ | incluido con Node.js |
| Phantom Wallet | última | extensión de navegador, solo para demo en devnet |

---

## Instalación paso a paso

### 1. Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup update stable
```

Verificar: `rustc --version`

---

### 2. Solana CLI (Agave)

```bash
sh -c "$(curl -sSfL https://release.anza.xyz/v3.1.10/install)"
```

Agregar al PATH (agregar al final de `~/.bashrc` o `~/.zshrc`):

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

Aplicar sin reiniciar: `source ~/.bashrc`

Verificar: `solana --version`  → `solana-cli 3.1.10`

---

### 3. Anchor CLI

```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 1.0.2
avm use 1.0.2
```

Verificar: `anchor --version` → `anchor-cli 1.0.2`

> En caso de error de compilación, asegurarse de tener las dependencias del sistema:  
> **Ubuntu/Debian:** `sudo apt-get install -y pkg-config build-essential libudev-dev`

---

### 4. Clonar el repositorio

```bash
git clone <URL_DEL_REPO>
cd 88_Traz_Log
```

---

### 5. Instalar dependencias del proyecto Anchor

```bash
cd traz_log
npm install
```

---

## Ejecutar el proyecto en local (localnet)

Anchor CLI 1.0.2 usa **Surfpool** como validador local por defecto (no el
`solana-test-validator` clásico) para `anchor test` y `anchor localnet`. Surfpool
incluye **Studio**, una interfaz web para ver las transacciones en tiempo real, en
`http://127.0.0.1:18488`.

### Opción A — Todo en un comando (recomendado)

```bash
cd traz_log
anchor test
```

Anchor levanta Surfpool, despliega el programa, ejecuta todos los tests TypeScript y apaga el validador al terminar. Usa `anchor test --detach` si quieres que el validador (y el Studio) sigan corriendo después de los tests, para seguir explorando el estado a mano.

### Opción B — Validador separado (útil para explorar el estado)

```bash
# Terminal 1: levantar validador + build + deploy, y dejarlo corriendo
cd traz_log
anchor test --detach

# Terminal 2: frontend u otros comandos
```

Abre `http://127.0.0.1:18488` para ver el Studio con las transacciones en vivo.

Si por algún motivo necesitas el `solana-test-validator` clásico en vez de Surfpool, usa `--validator legacy` en `anchor test` o `anchor localnet`.

Resultado esperado: todos los tests en verde con `passing`.

---

## Demo en Devnet con Phantom Wallet

### 1. Configurar wallet para devnet

1. Instalar la extensión [Phantom Wallet](https://phantom.app) en el navegador
2. En Phantom → Settings → Developer Settings → activar **Testnet Mode**
3. Cambiar la red a **Solana Devnet**

### 2. Obtener SOL de prueba

```bash
solana config set --url devnet
solana airdrop 2
```

O desde el faucet web: https://faucet.solana.com

### 3. Desplegar en devnet

```bash
cd traz_log
anchor deploy --provider.cluster devnet
```

El Program ID se actualizará automáticamente en `Anchor.toml` y en `traz_log/src/lib.rs`.

### 4. Iniciar el frontend

```bash
cd traz_log/app
npm install
npm run dev
```

Abrir `http://localhost:5173`, conectar Phantom y seleccionar el rol correspondiente.

---

## Estructura del proyecto

```
88_Traz_Log/
├── traz_log/                          # Workspace de Anchor
│   ├── programs/
│   │   └── traz_log/
│   │       └── src/
│   │           └── lib.rs             # Programa Rust principal
│   ├── tests/
│   │   └── traz_log.ts                # Tests de integración TypeScript
│   ├── app/                           # Frontend Vue.js + Phantom
│   ├── migrations/
│   │   └── deploy.ts
│   ├── Anchor.toml                    # Configuración del workspace
│   ├── Cargo.toml
│   └── package.json
├── Reference/                         # Documentos de análisis (solo local, no en repo)
├── CLAUDE.md                          # Contexto del proyecto para asistente IA
├── README.md                          # Este archivo
└── .gitignore
```

---

## Arquitectura

### 4 tipos de cuentas (PDAs)

| Cuenta | Seeds | Descripción |
|---|---|---|
| `GlobalState` | `[b"global"]` | Contador de incidentes, flag de pausa, admin |
| `PersonnelAccount` | `[b"personnel", wallet]` | Brigadista con rol asignado |
| `EquipmentAccount` | `[b"equipment", code[32]]` | Equipo con estado, custodio y consumo nominal |
| `IncidentAccount` | `[b"incident", id[8]]` | Incendio con coordenadas, nivel de riesgo y estado activo |

### 4 roles

| Rol | Permisos |
|---|---|
| `Admin` | Registro de personal, pausa del sistema |
| `OperationalBase` | Registro de equipos, auditoría |
| `SceneCommander` | Apertura/cierre de incidentes, asignación de equipos |
| `Operator` | Reporte de hitos, retorno de equipos |

### 8 instrucciones (MVP)

```
toggle_pause          → alterna pausa del sistema
register_personnel    → registra brigadista con rol
register_equipment    → registra equipo en inventario
open_fire_incident    → abre nuevo incidente con coordenadas GPS
assign_equipment      → asigna equipo a operador en un incidente
log_milestone         → reporta estado del equipo desde campo
initiate_return       → operador inicia retorno del equipo
close_incident        → cierra incidente (lazy: equipos transicionan individualmente)
```

### Flujo completo de un incidente

```
Admin registra personal con roles
BaseOperativa carga inventario de equipos
SceneCommander abre incidente → asigna equipos → operadores registran hitos
SceneCommander cierra incidente (marca activo = false)
Cada Operator ejecuta initiate_return para sus equipos
```

---

## Ramas de desarrollo

| Rama | Contenido |
|---|---|
| `main` | Código estable, recibe merges de cada fase completada |
| `0_anchor_fundamentals` | Setup del workspace, tutoriales base |
| `1_account_architecture` | Definición de structs, enums y PDAs |
| `2_registration_and_inventory` | toggle_pause, register_personnel, register_equipment |
| `3_incident_management` | open_fire_incident, assign_equipment, log_milestone |
| `4_return_and_close` | initiate_return, close_incident, tests E2E |
| `5_phantom_frontend` | Frontend Vue.js con Phantom wallet |
| `6_devnet_deploy` | Deploy en Solana Devnet + QA |
| `7_onchain_log` *(opcional)* | Bitácora histórica on-chain con LogEntry PDAs |

---

## Proyecto original en Solidity

Este proyecto es la traducción de:  
`TrazabilidadLogistica.sol` — sistema FireOPS desarrollado con Foundry/Solidity para Ethereum.  
El análisis de migración completo (mapeo de patrones, rediseño de componentes, estimación de esfuerzo) está disponible en el directorio `Reference/` del repositorio local.

---

## Troubleshooting

**`anchor build` falla con error de versión de Rust:**
```bash
rustup update stable
rustup override set stable
```

**Surfpool / validador local no inicia:**
```bash
anchor test --detach --validator legacy   # fuerza solana-test-validator clásico
```
Si el puerto `18488` (Studio) o `8899`/`8900` ya están en uso por una corrida previa que quedó colgada (ej. apagón abrupto), mata el proceso residual antes de reintentar.

**Error `Program failed to deploy` en devnet:**
Verificar saldo: `solana balance --url devnet`  
Si es 0, hacer airdrop: `solana airdrop 2 --url devnet`

**Phantom no conecta al frontend:**
Confirmar que Phantom está en red **Devnet** y que el programa está desplegado en devnet con `anchor deploy`.
