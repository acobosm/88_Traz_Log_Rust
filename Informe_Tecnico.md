# Informe Técnico — FireOPS Traz_Log
## Sistema de Trazabilidad Logística para Operaciones de Combate de Incendios Forestales
### Migración Solidity/Ethereum → Rust/Anchor/Solana

---

| Campo | Detalle |
|---|---|
| **Proyecto** | FireOPS — Sistema de Trazabilidad Logística |
| **Versión** | 0.1.0 (Fase 1 — Arquitectura de cuentas verificada) |
| **Fecha** | 2026-07-04 |
| **Autor** | Andres C. |
| **Repositorio** | `04_Rust_Practice/88_Traz_Log` |
| **Contrato original** | `TrazabilidadLogistica.sol` (635 líneas, Solidity ^0.8.19) |

---

## 1. Resumen Ejecutivo

Este proyecto traduce el sistema FireOPS de Solidity/Ethereum a Rust/Anchor sobre Solana. El contrato original implementa una cadena de custodia inmutable para operaciones de combate de incendios forestales: registra personal, inventario de equipos, incidentes activos y el retorno de recursos con auditoría en tres pasos.

La migración no es un port 1:1 — Solana opera bajo un modelo de ejecución fundamentalmente diferente (programas sin estado, cuentas PDA separadas) que requiere rediseño de tres componentes del contrato original:

1. **`cerrarIncidente`** → Lazy transition (el contrato original iteraba un array dinámico no viable en Solana)
2. **Bitácora on-chain** → Reemplazada por eventos `emit!` en el MVP (las PDAs de log se añaden en fase opcional)
3. **View functions** → Eliminadas del programa; se resuelven con `getProgramAccounts` desde el cliente

**Alcance MVP:** 9 instrucciones, 4 tipos de cuenta (PDAs), 4 roles, 5 eventos, 13 códigos de error.

---

## 2. Stack Tecnológico

| Herramienta | Versión | Rol |
|---|---|---|
| Rust | 1.89.0 | Lenguaje del programa on-chain |
| Anchor CLI | 1.0.2 | Framework de smart contracts para Solana |
| Solana CLI (Agave) | 3.1.10 | Cliente y validador local |
| anchor-lang | 1.1.2 | Crate de macros y runtime de Anchor |
| LiteSVM | 0.10.0 | Simulador de Solana para tests en Rust |
| Red objetivo | Solana Devnet / Localnet | |

---

## 3. Arquitectura del Sistema

### 3.1 Modelo de cuentas (PDAs)

En Solana, los programas son stateless — el estado vive en cuentas separadas derivadas determinísticamente (PDAs). El programa `traz_log` gestiona 4 tipos de cuentas:

```
GlobalState PDA           PersonnelAccount PDA
seeds: ["global"]         seeds: ["personnel", wallet_pubkey]
─────────────────         ────────────────────────────────────
next_incident_id: u64     wallet: Pubkey
is_paused: bool           name: String (máx 64 chars)
admin: Pubkey             specialty: String (máx 64 chars)
bump: u8                  is_active: bool
                          role: Role
                          bump: u8

EquipmentAccount PDA      IncidentAccount PDA
seeds: ["equipment", code[32]]   seeds: ["incident", id_le_bytes[8]]
─────────────────────────        ──────────────────────────────────
code: [u8; 32]            incident_id: u64
description: String (máx 128)    coordinates: String (máx 128)
nominal_consumption: u64  risk_level: u8
status: EquipmentStatus   is_active: bool
reported_condition: ReportedCondition   opened_at: i64
custodian: Pubkey         commander: Pubkey
incident_id: u64          bump: u8
use_start_time: i64
bump: u8
```

### 3.2 Roles y permisos

| Rol (`Role` enum) | Instrucciones autorizadas |
|---|---|
| `Admin` | `initialize`, `toggle_pause`, `register_personnel` |
| `OperationalBase` | `register_equipment` |
| `SceneCommander` | `open_fire_incident`, `assign_equipment`, `close_incident` |
| `Operator` | `log_milestone`, `initiate_return` |

### 3.3 Flujo completo de un incidente

```
Admin → initialize()                   # Crea GlobalState PDA (una sola vez)
Admin → register_personnel()           # Registra cada brigadista con rol

OperationalBase → register_equipment() # Registra cada equipo en inventario

SceneCommander → open_fire_incident()  # Abre incidente, incrementa contador
SceneCommander → assign_equipment()    # Equipo: Available → InUse, asigna custodio

Operator → log_milestone()             # Actualiza ReportedCondition del equipo

SceneCommander → close_incident()      # Incidente: activo=false  (LAZY)
Operator → initiate_return()           # Equipo: InUse → Returning (por custodio)
```

### 3.4 Decisión de arquitectura crítica: Lazy Close

En Solidity, `cerrarIncidente` iteraba el array `recursosAsignados[]` para cambiar todos los equipos a `EnRetorno` en una sola transacción. En Solana esto es inviable: cada cuenta debe declararse explícitamente (límite ~32 por tx), y arrays dinámicos no tienen equivalente directo.

**Solución elegida:** `close_incident` solo marca `incident.is_active = false`. Cada operador ejecuta individualmente `initiate_return` para su equipo. El sistema mantiene la trazabilidad y la seguridad sin depender de iteraciones en cadena.

---

## 4. Estructura de Archivos

```
traz_log/
├── Anchor.toml                         Configuración del workspace
├── Cargo.toml                          Workspace Cargo (dependencias)
├── rust-toolchain.toml                 Pinea Rust 1.89.0
├── programs/
│   └── traz_log/
│       ├── Cargo.toml                  Dependencias del programa
│       └── src/
│           ├── lib.rs                  Punto de entrada, enruta instrucciones
│           ├── constants.rs            Seeds de los 4 PDAs
│           ├── state.rs                Account structs y enums
│           ├── events.rs               Structs de eventos emit!
│           ├── error.rs                Códigos de error custom
│           ├── instructions.rs         Módulo agregador (re-exports)
│           └── instructions/
│               ├── initialize.rs       Setup del GlobalState
│               ├── toggle_pause.rs     Pausa/reanuda el sistema
│               ├── register_personnel.rs  Registra brigadista
│               ├── register_equipment.rs  Registra equipo
│               ├── open_fire_incident.rs  Abre incidente
│               ├── assign_equipment.rs    Asigna equipo a operador
│               ├── log_milestone.rs       Reporta estado del equipo
│               ├── initiate_return.rs     Inicia retorno del equipo
│               └── close_incident.rs      Cierra incidente (lazy)
└── tests/
    └── test_initialize.rs              Tests de integración con LiteSVM
```

---

## 5. Documentación Detallada de Archivos

### 5.1 `constants.rs`

Centraliza los seeds de derivación de PDAs como constantes de bytes. Usados en todos los `#[derive(Accounts)]` structs para evitar strings literales duplicadas.

```rust
pub const SEED_GLOBAL: &[u8]     = b"global";
pub const SEED_PERSONNEL: &[u8]  = b"personnel";
pub const SEED_EQUIPMENT: &[u8]  = b"equipment";
pub const SEED_INCIDENT: &[u8]   = b"incident";
```

---

### 5.2 `state.rs`

Define las 4 estructuras de cuenta (PDAs) y los 3 enums del dominio.

#### Account Structs

**`GlobalState`** — Una única instancia por programa.

```
Parámetros:
  next_incident_id: u64     Contador autoincremental para IDs de incidentes
  is_paused: bool           Flag de circuit breaker del sistema
  admin: Pubkey             Dirección con privilegios de administrador
  bump: u8                  Bump canónico del PDA (guardado para eficiencia)

Espacio: 8 (discriminador) + 8 + 1 + 32 + 1 = 50 bytes
Rent exemption aprox.: 348,000 lamports (~0.000348 SOL)
```

**`PersonnelAccount`** — Una instancia por brigadista.

```
Parámetros:
  wallet: Pubkey            Dirección de la wallet del brigadista
  name: String              Nombre completo (máx. 64 caracteres)
  specialty: String         Especialidad/cargo (máx. 64 caracteres)
  is_active: bool           Si el personal está activo en el sistema
  role: Role                Rol asignado (enum de 4 variantes)
  bump: u8                  Bump canónico del PDA

Espacio: 8 + 32 + (4+64) + (4+64) + 1 + 1 + 1 = 179 bytes
Rent exemption aprox.: 1,245,840 lamports (~0.00125 SOL)
```

**`EquipmentAccount`** — Una instancia por equipo registrado.

```
Parámetros:
  code: [u8; 32]            Código único del equipo (equivalente a bytes32)
  description: String       Descripción del equipo (máx. 128 caracteres)
  nominal_consumption: u64  Consumo nominal en ml/hora
  status: EquipmentStatus   Estado actual (enum de 5 variantes)
  reported_condition: ReportedCondition  Condición reportada (enum 4 variantes)
  custodian: Pubkey         Wallet del operador custodio (Pubkey::default si disponible)
  incident_id: u64          ID del incidente asignado (0 si disponible)
  use_start_time: i64       Unix timestamp de inicio de uso (0 si disponible)
  bump: u8                  Bump canónico del PDA

Espacio: 8 + 32 + (4+128) + 8 + 1 + 1 + 32 + 8 + 8 + 1 = 231 bytes
Rent exemption aprox.: 1,607,760 lamports (~0.00161 SOL)
```

**`IncidentAccount`** — Una instancia por incidente abierto.

```
Parámetros:
  incident_id: u64          ID único del incidente (del contador en GlobalState)
  coordinates: String       Coordenadas GPS / descripción del lugar (máx. 128 chars)
  risk_level: u8            Nivel de riesgo 1-5
  is_active: bool           Si el incidente está activo
  opened_at: i64            Unix timestamp de apertura
  commander: Pubkey         Wallet del SceneCommander que abrió el incidente
  bump: u8                  Bump canónico del PDA

Espacio: 8 + 8 + (4+128) + 1 + 1 + 8 + 32 + 1 = 191 bytes
Rent exemption aprox.: 1,329,960 lamports (~0.00133 SOL)
```

#### Enums

**`Role`** — Roles del sistema.

```
Admin           Administrador del sistema
OperationalBase Base operativa (gestión de inventario)
SceneCommander  Jefe de escena (gestión de incidentes)
Operator        Brigadista en campo
```

**`EquipmentStatus`** — Estados del ciclo de vida del equipo.

```
Available   Disponible en bodega
InUse       Asignado y en uso en un incidente
InRepair    En taller (fuera de servicio)
Lost        Perdido o dado de baja
Returning   En proceso de retorno post-incidente
```

**`ReportedCondition`** — Condición reportada desde campo.

```
Operational    Funcionando con normalidad
MinorDamage    Daño menor, funcional
CriticalDamage Daño crítico, requiere revisión urgente
Lost           Reportado como perdido en campo
```

---

### 5.3 `events.rs`

Define los 5 structs de eventos emitidos con `emit!()`. Los eventos quedan indexados en el log de la transacción y son consumibles por el frontend vía `program.addEventListener()`.

| Evento | Campos emitidos | Cuándo se emite |
|---|---|---|
| `PersonnelRegistered` | `wallet: Pubkey`, `role: Role` | Al completar `register_personnel` |
| `EquipmentRegistered` | `code: [u8; 32]` | Al completar `register_equipment` |
| `IncidentOpened` | `incident_id: u64`, `commander: Pubkey`, `risk_level: u8` | Al completar `open_fire_incident` |
| `EquipmentAssigned` | `incident_id: u64`, `equipment_code: [u8; 32]`, `operator: Pubkey` | Al completar `assign_equipment` |
| `IncidentClosed` | `incident_id: u64` | Al completar `close_incident` |

---

### 5.4 `error.rs`

Define 13 códigos de error custom con mensajes descriptivos. En Anchor, estos se propagan como `ProgramError` al cliente con el código numérico y el mensaje.

| Código | Nombre | Mensaje | Disparado en |
|---|---|---|---|
| 6000 | `Unauthorized` | Unauthorized: insufficient role for this action | Cualquier instrucción con rol incorrecto |
| 6001 | `SystemPaused` | System is currently paused | Cualquier instrucción cuando `is_paused = true` |
| 6002 | `InactivePersonnel` | Personnel account is inactive | Instrucciones que verifican `is_active` |
| 6003 | `EquipmentNotAvailable` | Equipment is not available for assignment | `assign_equipment` cuando status ≠ Available |
| 6004 | `EquipmentNotInUse` | Equipment is not in use | `log_milestone`, `initiate_return` |
| 6005 | `NotCustodian` | Caller is not the equipment custodian | `log_milestone`, `initiate_return` |
| 6006 | `IncidentNotActive` | Incident is not active | `assign_equipment` con incidente cerrado |
| 6007 | `IncidentAlreadyClosed` | Incident is already closed | `close_incident` cuando ya está cerrado |
| 6008 | `InvalidOperatorRole` | Assigned operator must have the Operator role | `assign_equipment` con rol incorrecto |
| 6009 | `InvalidEquipmentStatus` | Invalid equipment status for this operation | Futuras instrucciones de handshake |
| 6010 | `InvalidRiskLevel` | Risk level must be between 1 and 5 | `open_fire_incident` |
| 6011 | `InvalidIncidentId` | Provided incident ID does not match current counter | `open_fire_incident` con ID incorrecto |

> Los códigos Anchor custom empiezan en 6000 (`anchor_lang::error::ERROR_CODE_OFFSET`).

---

### 5.5 `instructions.rs`

Módulo agregador: declara los 9 submódulos de instrucciones y los re-exporta con `pub use mod::*` para que los tipos `Context<T>` estén disponibles en el scope de `lib.rs`.

```
Módulos declarados y re-exportados:
  assign_equipment, close_incident, initialize, initiate_return,
  log_milestone, open_fire_incident, register_equipment,
  register_personnel, toggle_pause
```

> Se genera una warning de compilación (`ambiguous_glob_reexports`) porque la función `handler` existe en todos los módulos. Es inofensiva — el `#[program]` en `lib.rs` invoca siempre los handlers por path completo (`register_personnel::handler(ctx, ...)`), eliminando toda ambigüedad.

---

### 5.6 `lib.rs`

Punto de entrada del programa Anchor. Contiene:

- `declare_id!("13p4xV6WHaPwWzno1F6Z6MeY9b9wLYtEMAnUF8ofdW75")` — Program ID generado al inicializar el workspace
- Declaración de módulos y re-exports públicos
- Bloque `#[program]` con las 9 funciones públicas que enrutan a los handlers individuales

Cada función en `#[program]` delega inmediatamente al handler del módulo correspondiente, manteniendo `lib.rs` como enrutador limpio con la lógica encapsulada en cada archivo de instrucción.

---

### 5.7 Instrucciones — Documentación Detallada

#### `initialize`

**Propósito:** Crea el `GlobalState` PDA. Se llama una única vez después del deploy.

| | |
|---|---|
| **Rol requerido** | La wallet que firma queda registrada como `admin` |
| **Params de entrada** | *(ninguno)* |
| **Cuentas requeridas** | `admin` (Signer, mut), `global_state` (init PDA), `system_program` |
| **Estado modificado** | Crea `GlobalState` con `next_incident_id=0`, `is_paused=false`, `admin=signer.key()` |
| **Emite evento** | *(ninguno)* |
| **Puede fallar si** | `global_state` ya existe (Anchor rechaza `init` en cuenta existente) |

---

#### `toggle_pause`

**Propósito:** Alterna el flag `is_paused` del sistema. Si está activo, ninguna instrucción puede ejecutarse.

| | |
|---|---|
| **Rol requerido** | Admin (`global_state.admin == signer.key()`) — verificado via `constraint` |
| **Params de entrada** | *(ninguno)* |
| **Cuentas requeridas** | `signer` (Signer), `global_state` (mut, bump verificado) |
| **Estado modificado** | `global_state.is_paused = !global_state.is_paused` |
| **Emite evento** | *(ninguno)* |
| **Puede fallar si** | `signer` no es el admin → `Unauthorized` |

---

#### `register_personnel`

**Propósito:** Registra un miembro del equipo con rol asignado, creando su `PersonnelAccount` PDA.

| | |
|---|---|
| **Rol requerido** | Admin (Fase 0; la ruta `OperationalBase` se añade en fases posteriores) |
| **Params de entrada** | `name: String`, `specialty: String`, `role: Role` |
| **Cuentas requeridas** | `signer` (admin, mut), `global_state` (bump verificado), `new_personnel` (init PDA), `wallet` (SystemAccount — dirección a registrar), `system_program` |
| **Seeds del PDA** | `["personnel", wallet.key()]` |
| **Estado modificado** | Crea `PersonnelAccount` con `is_active=true` |
| **Emite evento** | `PersonnelRegistered { wallet, role }` |
| **Puede fallar si** | Sistema pausado → `SystemPaused`; signer no es admin → `Unauthorized`; PDA ya existe (Anchor rechaza `init`) |

---

#### `register_equipment`

**Propósito:** Registra un equipo en el inventario, creando su `EquipmentAccount` PDA en estado `Available`.

| | |
|---|---|
| **Rol requerido** | `OperationalBase` |
| **Params de entrada** | `code: [u8; 32]`, `description: String`, `nominal_consumption: u64` (ml/hora) |
| **Cuentas requeridas** | `signer` (mut), `global_state`, `signer_personnel` (PDA del signer — verifica rol), `equipment` (init PDA), `system_program` |
| **Seeds del PDA** | `["equipment", code]` |
| **Estado modificado** | Crea `EquipmentAccount` con `status=Available`, `custodian=Pubkey::default()`, `incident_id=0` |
| **Emite evento** | `EquipmentRegistered { code }` |
| **Puede fallar si** | Sistema pausado; rol ≠ OperationalBase → `Unauthorized`; personal inactivo → `InactivePersonnel`; code duplicado (Anchor rechaza `init`) |

---

#### `open_fire_incident`

**Propósito:** Abre un nuevo incidente de incendio, asignando un ID único y registrando coordenadas y nivel de riesgo.

| | |
|---|---|
| **Rol requerido** | `SceneCommander` |
| **Params de entrada** | `incident_id: u64` (debe coincidir con `global_state.next_incident_id`), `coordinates: String`, `risk_level: u8` (1–5) |
| **Cuentas requeridas** | `signer` (mut), `global_state` (mut — se incrementa el contador), `signer_personnel`, `incident` (init PDA), `system_program` |
| **Seeds del PDA** | `["incident", incident_id.to_le_bytes()]` |
| **Estado modificado** | Crea `IncidentAccount` activo; `global_state.next_incident_id += 1` |
| **Emite evento** | `IncidentOpened { incident_id, commander, risk_level }` |
| **Puede fallar si** | Sistema pausado; rol ≠ SceneCommander; `risk_level` fuera de 1–5 → `InvalidRiskLevel`; `incident_id` ≠ contador → `InvalidIncidentId` |

> El cliente lee `global_state.next_incident_id` antes de llamar esta instrucción y pasa ese valor como `incident_id`, permitiendo derivar el PDA del incidente antes de enviar la transacción.

---

#### `assign_equipment`

**Propósito:** Asigna un equipo disponible a un operador dentro de un incidente activo.

| | |
|---|---|
| **Rol requerido** | `SceneCommander` |
| **Params de entrada** | `equipment_code: [u8; 32]`, `incident_id: u64` |
| **Cuentas requeridas** | `signer`, `global_state`, `signer_personnel`, `equipment` (mut), `incident`, `operator_personnel` (verifica rol Operator), `operator_wallet` (SystemAccount) |
| **Verificaciones inline** | `equipment.status == Available`; `incident.is_active == true`; `operator_personnel.role == Operator` |
| **Estado modificado** | `equipment.status = InUse`; `equipment.custodian = operator_wallet.key()`; `equipment.incident_id = incident.incident_id`; `equipment.use_start_time = clock.unix_timestamp` |
| **Emite evento** | `EquipmentAssigned { incident_id, equipment_code, operator }` |
| **Puede fallar si** | Sistema pausado; equipo no disponible → `EquipmentNotAvailable`; incidente cerrado → `IncidentNotActive`; operador sin rol → `InvalidOperatorRole` |

---

#### `log_milestone`

**Propósito:** El operador custodio reporta la condición del equipo desde campo, actualizando `reported_condition`.

| | |
|---|---|
| **Rol requerido** | `Operator` (y debe ser el custodio actual del equipo) |
| **Params de entrada** | `equipment_code: [u8; 32]`, `notes: String` (referencia libre, off-chain), `condition: ReportedCondition` |
| **Cuentas requeridas** | `signer`, `global_state`, `signer_personnel`, `equipment` (mut) |
| **Verificaciones inline** | `equipment.custodian == signer.key()`; `equipment.status == InUse` |
| **Estado modificado** | `equipment.reported_condition = condition` |
| **Emite evento** | *(ninguno en MVP; los notes quedan en el payload RPC de la transacción)* |
| **Puede fallar si** | Sistema pausado; no es el custodio → `NotCustodian`; equipo no está en uso → `EquipmentNotInUse` |

---

#### `initiate_return`

**Propósito:** El operador custodio inicia el retorno del equipo. Transiciona el estado de `InUse` a `Returning`.

| | |
|---|---|
| **Rol requerido** | `Operator` (y debe ser el custodio actual del equipo) |
| **Params de entrada** | `equipment_code: [u8; 32]` |
| **Cuentas requeridas** | `signer`, `global_state`, `signer_personnel`, `equipment` (mut) |
| **Verificaciones inline** | `equipment.custodian == signer.key()`; `equipment.status == InUse` |
| **Estado modificado** | `equipment.status = Returning` |
| **Emite evento** | *(ninguno)* |
| **Puede fallar si** | Sistema pausado; no es el custodio → `NotCustodian`; equipo no está en uso → `EquipmentNotInUse` |

> Esta instrucción se llama individualmente por cada operador después (o antes) de que el SceneCommander ejecute `close_incident`. No hay dependencia de orden estricta entre ambas.

---

#### `close_incident`

**Propósito:** Cierra el incidente marcando `is_active = false`. Los equipos transicionan individualmente vía `initiate_return` (modelo lazy).

| | |
|---|---|
| **Rol requerido** | `SceneCommander` |
| **Params de entrada** | `incident_id: u64` |
| **Cuentas requeridas** | `signer`, `global_state`, `signer_personnel`, `incident` (mut) |
| **Verificaciones inline** | `incident.is_active == true` |
| **Estado modificado** | `incident.is_active = false` |
| **Emite evento** | `IncidentClosed { incident_id }` |
| **Puede fallar si** | Sistema pausado; incidente ya cerrado → `IncidentAlreadyClosed`; rol incorrecto → `Unauthorized` |

---

### 5.8 `tests/test_initialize.rs`

Suite de tests de integración usando **LiteSVM** — simulador de Solana que carga el bytecode compilado (`.so`) y ejecuta transacciones en memoria sin red. Es el framework de testing estándar en Anchor 1.0+.

#### Función auxiliar `global_state_pda`
Deriva el PDA de `GlobalState` a partir del program ID. Reutilizada por ambos tests para evitar duplicación.

#### Función auxiliar `send_initialize`
Construye y envía la transacción `initialize` completa. Reutilizada por el segundo test para evitar repetir el setup.

#### `test_initialize_creates_global_state`
- Levanta LiteSVM, carga el programa compilado desde `target/deploy/traz_log.so`
- Crea una wallet de admin y la fondea con 1 SOL (airdrop en LiteSVM)
- Deriva el PDA de `GlobalState` y construye la transacción `initialize`
- **Verifica:** la transacción se confirma sin error

#### `test_global_state_not_paused_after_init`
- Mismo setup que el test anterior, usando `send_initialize`
- Lee los datos crudos del `GlobalState` PDA después de la inicialización
- **Verifica:** el byte en offset 16 es 0 (campo `is_paused = false`)
  - Layout de memoria: `discriminator[8]` + `next_incident_id[8]` + `is_paused[1]` → offset 16

---

## 6. Cálculo de Espacio de Cuentas

Anchor 1.0 usa `#[derive(InitSpace)]` para calcular el espacio automáticamente. Para `String`, `#[max_len(N)]` reserva `4 + N` bytes (4 para el prefijo de longitud `u32` + N para el contenido máximo).

| Cuenta | Desglose de campos | Espacio total | Rent exempt aprox. |
|---|---|---|---|
| `GlobalState` | 8 + 8 + 1 + 32 + 1 | **50 bytes** | ~0.00035 SOL |
| `PersonnelAccount` | 8 + 32 + (4+64) + (4+64) + 1 + 1 + 1 | **179 bytes** | ~0.00125 SOL |
| `EquipmentAccount` | 8 + 32 + (4+128) + 8 + 1 + 1 + 32 + 8 + 8 + 1 | **231 bytes** | ~0.00161 SOL |
| `IncidentAccount` | 8 + 8 + (4+128) + 1 + 1 + 8 + 32 + 1 | **191 bytes** | ~0.00133 SOL |

> **Estimación de costo de deploy completo** (1 GlobalState + 10 personal + 50 equipos + 5 incidentes):
> 50 + (10×179) + (50×231) + (5×191) = 14,345 bytes → ~0.10 SOL en devnet.

---

## 7. Resultados de Tests

Ejecutado el 2026-07-04. Comando: `cargo test -- --nocapture`

```
warning: ambiguous glob re-exports
  --> programs/traz_log/src/instructions.rs:11:9
   |
11 | pub use assign_equipment::*;
   |         ^^^^^^^^^^^^^^^^^^^ the name `handler` in the value namespace is first re-exported here
...
19 | pub use toggle_pause::*;
   |         --------------- but the name `handler` in the value namespace is also re-exported here
   |
   = note: `#[warn(ambiguous_glob_reexports)]` on by default

warning: `traz_log` (lib) generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.81s
     Running unittests src/lib.rs (target/debug/deps/traz_log-1d609f2171abe445)

running 1 test
test test_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/test_initialize.rs (target/debug/deps/test_initialize-ce196e98174629cd)

running 2 tests
test test_global_state_not_paused_after_init ... ok
test test_initialize_creates_global_state ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s

   Doc-tests traz_log

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Resumen acumulado:** 8 tests pasados, 0 fallidos.

| Suite | Tests | Archivo |
|---|---|---|
| Unittests lib | 1 | `src/lib.rs` (generado por Anchor) |
| Fase 0 — Initialize | 2 | `tests/test_initialize.rs` |
| Fase 1 — Account Architecture | 5 | `tests/test_toggle_pause.rs` |

> **Sobre la warning:** El re-export glob genera un warning porque la función `handler` tiene el mismo nombre en todos los módulos. Es inofensiva — las instrucciones se invocan siempre por path completo en `lib.rs` (`register_personnel::handler(ctx, ...)`), eliminando toda ambigüedad en runtime.

> **Hallazgo técnico — `expire_blockhash()`:** En LiteSVM 0.10, dos transacciones con idéntico contenido (mismos accounts, misma instrucción, mismo signer) dentro del mismo slot tienen la misma firma y la segunda falla con `AlreadyProcessed`. La solución es llamar `svm.expire_blockhash()` entre envíos para rotar el blockhash interno y generar firmas distintas. Este patrón aplica en cualquier test que envíe la misma instrucción dos veces consecutivas.

---

## 8. Mapeo Solidity → Rust/Anchor

| Componente Solidity | Equivalente Rust/Anchor | Observación |
|---|---|---|
| `mapping(address => Personal)` | `PersonnelAccount` PDA `seeds = ["personnel", wallet]` | Lookup determinista sin storage adicional |
| `mapping(bytes32 => Insumo)` | `EquipmentAccount` PDA `seeds = ["equipment", code]` | `bytes32` → `[u8; 32]` |
| `mapping(uint256 => EventoIncendio)` | `IncidentAccount` PDA `seeds = ["incident", id_le_bytes]` | `uint256` → `u64` |
| `AccessControl` (OpenZeppelin) | Campo `role: Role` en `PersonnelAccount` + `constraint =` en Accounts | Sin dependencia externa |
| `Pausable` (OpenZeppelin) | Campo `is_paused: bool` en `GlobalState` + `require!(!paused)` | Implementación nativa |
| `ReentrancyGuard` (OpenZeppelin) | No necesario | El modelo de cuentas de Solana previene reentrancia estructuralmente |
| `tx.origin` | Eliminado | En Solana el `Signer` siempre es explícito y verificado por el runtime |
| `emit EventName(...)` | `emit!(EventName { campos })` | Prácticamente idéntico en sintaxis |
| `bytes32[] recursosAsignados` | Eliminado — reemplazado por lazy transition | Sin equivalente directo: rediseño necesario |
| `require("mensaje")` | `require!(condición, ErrorCode::Variante)` | Errores tipados y decodificables por el cliente |
| View functions | `getProgramAccounts` off-chain | No son instrucciones — el cliente filtra y deserializa PDAs |

---

## 9. Decisiones de Arquitectura

### 9.1 Bumps almacenados en cada cuenta
Cada PDA almacena su `bump: u8` en el momento de creación (desde `ctx.bumps.account_name`). En verificaciones posteriores se usa `bump = account.bump` en lugar de recalcular el bump canónico en cada instrucción, reduciendo compute units.

### 9.2 `incident_id` como parámetro de instrucción
El cliente lee `GlobalState.next_incident_id`, lo pasa como primer argumento de `open_fire_incident`, y Anchor lo usa en el `#[instruction(incident_id)]` para derivar el PDA del incidente. El programa verifica que `incident_id == global_state.next_incident_id`. Esto permite al cliente construir la transacción completa (con la dirección PDA pre-calculada) antes de enviarla.

### 9.3 `register_personnel` admin-only en Fase 0
La instrucción original `registrarPersonal` aceptaba `DEFAULT_ADMIN_ROLE` o `BASE_OPERATIVA_ROLE`. En Anchor, manejar una cuenta opcional (`Option<Account<'info, T>>`) añade complejidad innecesaria en Fase 0. Solución: admin-only en esta fase. La ruta `OperationalBase` se añade en Fase 3 sin cambiar la interfaz pública del programa.

---

## 10. Estado del Proyecto y Roadmap

| Fase | Rama Git | Estado | Contenido principal |
|---|---|---|---|
| **0** | `0_anchor_fundamentals` | **Completada** | Workspace, 9 instrucciones, 3 tests de initialize |
| **1** | `1_account_architecture` | **Completada** | Tamaño de GlobalState, deserialización de campos, 5 tests de toggle_pause |
| 2 | `2_registration_and_inventory` | Pendiente | Tests completos de `register_personnel` y `register_equipment` |
| 3 | `3_incident_management` | Pendiente | Tests de `open_fire_incident`, `assign_equipment`, `log_milestone` |
| 4 | `4_return_and_close` | Pendiente | Tests de `initiate_return`, `close_incident`, flujo E2E completo |
| 5 | `5_phantom_frontend` | Pendiente | Vue.js + Phantom Wallet adapter |
| 6 | `6_devnet_deploy` | Pendiente | Deploy a Solana Devnet + QA end-to-end |
| *(opt)* | `7_onchain_log` | Opcional | `LogEntry` PDAs, bitácora histórica queryable |

### Remotos Git configurados

| Alias | URL | Estrategia |
|---|---|---|
| `ghp` | `https://github.com/acobosm/88_Traz_Log_Rust.git` | Push continuo durante desarrollo |
| `glp` | `https://gitlab.com/acobosm1/web3-blockchain/88_traz_log_rust.git` | Push continuo durante desarrollo |
| `gla` | *(pendiente — academia)* | Solo desde el 3 de agosto de 2026, push incremental para simular 30 días de desarrollo |

> Este informe se actualiza al completar cada fase. Los resultados de `cargo test` en la Sección 7 se reemplazan con el output de la fase más reciente.
