# CLAUDE.md — FireOPS Traz_Log (Rust/Anchor)

## Descripción del proyecto
Traducción de `TrazabilidadLogistica.sol` (Solidity/Foundry) a Rust/Anchor + Phantom Wallet.
Sistema de trazabilidad logística para operaciones de combate de incendios forestales.

**Contrato original:** `03 Ethereum Practice/.../88_Traz_Log/contracts/TrazabilidadLogistica.sol` (635 líneas)
**Directorio Rust:** `/home/ebit/projects/0 CodeCrypto Academy/04_Rust_Practice/88_Traz_Log/`
**Programa Anchor:** `traz_log` (subdirectorio `88_Traz_Log/traz_log/`)
**Plazo de entrega final:** 28 Ago 2026

---

## Stack
- Anchor CLI 1.0.2
- Solana CLI 3.1.10 (Agave)
- Validador local: **Surfpool** (default de Anchor 1.0.2 para `anchor test`/`anchor localnet`, flag `--validator surfpool|legacy`). Studio en `http://127.0.0.1:18488` para ver transacciones en tiempo real. `legacy` cae a `solana-test-validator` clásico.
- Tests: Rust + LiteSVM 0.10 (Anchor 1.0 no usa TypeScript para tests)
- Frontend: Vue.js + `@solana/wallet-adapter-vue` + Phantom
- Red objetivo: Solana Devnet

---

## Convenciones de código y git

**Todo el código en inglés:** funciones, structs, enums, variantes, eventos, errores, campos, seeds.
Los comentarios explicativos de lógica de negocio pueden estar en español.

**Ramas:** inglés, snake_case, prefijo numérico — `{N}_{descripcion_en_ingles}`

**Commits:** inglés, estilo imperativo corto
Ejemplos: `feat: add register_personnel instruction`, `test: add toggle_pause happy path`

---

## Alcance: MVP ampliado

Scope reducido respecto al contrato original. Análisis completo en `Reference/`.

### 8 instrucciones incluidas

| Instrucción Rust | Rol requerido | Equivalente Solidity |
|---|---|---|
| `toggle_pause` | Admin | `pause()` / `unpause()` |
| `register_personnel` | Admin o OperationalBase | `registrarPersonal` |
| `register_equipment` | OperationalBase | `registrarInsumo` |
| `open_fire_incident` | SceneCommander | `abrirEventoIncendio` |
| `assign_equipment` | SceneCommander | `asignarInsumo` |
| `log_milestone` | Operator (custodian) | `registrarHito` |
| `initiate_return` | Operator (custodian) | `iniciarRetorno` |
| `close_incident` | SceneCommander | `cerrarIncidente` |

### Excluido del MVP (fase opcional si sobran días)
- Handshake de 3 pasos: `register_audit` + `sign_release`
- ~~Bitácora on-chain: `LogEntry` PDAs — posible en `7_onchain_log`~~ → **adelantada e implementada en Fase 5** (ver estado actual)
- `register_equipment_batch` (multi-tx)
- `log_tactical_note`
- `update_fire_risk`
- `register_audit_report`
- Roles `Auditor` y `Consultant`

---

## Nombres de código — mapeo Solidity → Rust

### Accounts (PDAs)

| Rust struct | Seeds | Equivalente Solidity |
|---|---|---|
| `GlobalState` | `[b"global"]` | variables globales del contrato |
| `PersonnelAccount` | `[b"personnel", wallet_pubkey.as_ref()]` | `mapping(address => Personal)` |
| `EquipmentAccount` | `[b"equipment", code[32]]` | `mapping(bytes32 => Insumo)` |
| `IncidentAccount` | `[b"incident", &id.to_le_bytes()]` | `mapping(uint256 => EventoIncendio)` |
| `LogEntry` (adelanto `7_onchain_log`) | `[b"log", equipment_code[32], entry_index.to_le_bytes()]` | Bitácora histórica, sin equivalente 1:1 en el contrato original |

*Tamaños exactos en bytes se calculan en Fase 1 antes de `anchor build`.*

### Enums

| Rust | Variantes | Equivalente Solidity |
|---|---|---|
| `Role` | `Admin`, `OperationalBase`, `SceneCommander`, `Operator` | roles de `AccessControl` |
| `EquipmentStatus` | `Available`, `InUse`, `InRepair`, `Lost`, `Returning` | `EstadoInsumo` |
| `ReportedCondition` | `Operational`, `MinorDamage`, `CriticalDamage`, `Lost` | `EstadoReportado` |

### Eventos `emit!`

| Rust event | Equivalente Solidity |
|---|---|
| `PersonnelRegistered` | `InsumoRegistrado` |
| `EquipmentRegistered` | `InsumoRegistrado` |
| `IncidentOpened` | `IncendioIniciado` |
| `EquipmentAssigned` | `InsumoAsignado` |
| `IncidentClosed` | `IncendioCerrado` |

---

## Decisiones de arquitectura tomadas

| Decisión | Opción elegida | Razón |
|---|---|---|
| `close_incident` | **Lazy transition** | Solo marca `active = false`. Cada operador llama `initiate_return` individualmente. Evita iterar array dinámico y el límite de ~32 accounts por tx de Solana. |
| Bitácora | **`LogEntry` PDA (adelantado a Fase 5)** | `log_milestone` ahora crea un PDA histórico por reporte (`seeds = ["log", code, entry_index]`) además de emitir el evento. Se adelantó `7_onchain_log` porque el costo de implementarlo era bajo y da queryabilidad real desde el frontend. |
| `register_equipment_batch` | **Eliminado en MVP** | Reemplazado por registros individuales. El batch de 51 items requiere `signAllTransactions` — complejidad no esencial para el MVP. |
| RBAC | **Campo `role: Role` en `PersonnelAccount`** | Sin `AccessControl` de OZ. Verificación via `constraint =` en cada Accounts struct. |
| Roles activos | **4 roles** | Admin, OperationalBase, SceneCommander, Operator. Auditor y Consultant fuera del MVP. |
| `tx.origin` de Solidity | **Eliminado** | En Solana el signer es siempre explícito. Elimina la vulnerabilidad Media del contrato original. |
| Doble compromiso de operador | **Campos `current_incident`/`active_assignments` en `PersonnelAccount`** | El contrato Solidity original no impedía que un operador quedara custodio en dos incidentes activos a la vez. Se detectó en QA manual de Fase 5 y se corrigió con error `OperatorAlreadyAssigned`. |
| Suplantación de comando en `assign_equipment` | **`constraint = incident.commander == signer.key()`** | Un `SceneCommander` podía asignar equipo a un incidente que no había abierto él. Corregido con error `NotIncidentCommander`. |

---

## Plan de fases y ramas git

| # | Rama | Contenido | Fechas |
|---|---|---|---|
| 0 | `0_anchor_fundamentals` | Tutoriales counter + escrow, `anchor init traz_log`, `.gitignore`, `anchor build` verde | Jul 4–13 |
| 1 | `1_account_architecture` | 4 account structs con tamaños, enums, seeds, 8 stubs vacíos que compilan | Jul 13–18 |
| 2 | `2_registration_and_inventory` | `toggle_pause`, `register_personnel`, `register_equipment` + tests TS | Jul 18–22 |
| 3 | `3_incident_management` | `open_fire_incident`, `assign_equipment`, `log_milestone` + tests TS | Jul 22–29 |
| 4 | `4_return_and_close` | `initiate_return`, `close_incident` + tests TS + flujo E2E completo | Jul 29–Aug 5 |
| 5 | `5_phantom_frontend` | Vue.js reescrito: wallet adapter, paneles por rol, `getProgramAccounts`, event listeners. Incluye adelanto de `LogEntry` PDAs (bitácora on-chain) | Aug 5–17 |
| 6 | `6_devnet_deploy` | `anchor deploy devnet`, QA manual con Phantom, verificar rent, correcciones UX | Aug 17–22 |
| ~~*(opt)*~~ | ~~`7_onchain_log`~~ | **Absorbida en Fase 5** — ya no requiere rama propia | — |

**Criterio de "done" por fase:**
- Fases 0–4: `anchor test` pasa en verde (localnet)
- Fases 5–6: demo funcional manual con Phantom

### Flujo git por fase
```
main → crear rama {N}_{name}
         → desarrollar + testear
         → push a ghp y glp  (libre, cualquier fecha)
         → merge a main
         → push main a ghp y glp
```

---

## Remotos git

| Nombre | Destino | Restricción |
|---|---|---|
| `ghp` | GitHub personal | Libre, cualquier fecha |
| `glp` | GitLab personal | Libre, cualquier fecha |
| `gla` | GitLab academia | **Solo desde agosto 3, 2026** |

*URLs a configurar cuando estén listos los repos remotos. Avisar a Claude para ejecutar `git remote add`.*

## Estrategia de push a `gla`

Desarrollo real en los repos personales (`ghp`/`glp`), push a `gla` incremental por rama para simular una progresión de ~30 días. El calendario de fechas específico no se versiona en este archivo — se gestiona aparte para evitar que quede desactualizado.

---

## Estado actual del proyecto

- [x] Análisis del contrato original completado (`Reference/`)
- [x] MVP scope definido
- [x] `git init` ejecutado en `88_Traz_Log/`
- [x] `CLAUDE.md` creado
- [x] `.gitignore` creado
- [x] `README.md` creado
- [x] `Informe_Tecnico.md` creado y actualizado
- [x] `anchor init traz_log` ejecutado
- [x] 9 instrucciones implementadas en `programs/traz_log/src/instructions/`
- [x] 5 account structs (incluye `LogEntry`) + 3 enums + 5 eventos + 14 errores en `state.rs`, `events.rs`, `error.rs`
- [x] Remoto `ghp` → `https://github.com/acobosm/88_Traz_Log_Rust.git`
- [x] Remoto `glp` → `https://gitlab.com/acobosm1/web3-blockchain/88_traz_log_rust.git`
- [x] Remoto `gla` → `https://gitlab.codecrypto.academy/andres.cobos/88_traz_log_rust.git` (configurado 2026-08-14)
- [x] Fase 0 completada — 3 tests en verde (`test_initialize.rs`)
- [x] Fase 1 completada — 5 tests en verde (`test_toggle_pause.rs`)
- [x] Fase 2 completada — 10 tests en verde (`test_register_personnel.rs`, `test_register_equipment.rs`)
- [x] Fase 3 completada — 15 tests en verde (`test_open_fire_incident.rs`, `test_assign_equipment.rs`, `test_log_milestone.rs`)
- [x] Fase 4 completada — 10 tests + 1 E2E en verde (`test_initiate_return.rs`, `test_close_incident.rs`, `test_e2e_full_flow.rs`)
- [x] Fase 5 completada — Vue 3 + Vite + Phantom: scaffold, `useWallet`, `useProgram`, 5 vistas (Dashboard, Inventario, Admin, Incidente, Campo), las 9 instrucciones + 7 lecturas conectadas, build limpio, flujo completo validado en localnet
- [x] Fase 5 (adelanto) — Guardrails `NotIncidentCommander` y `OperatorAlreadyAssigned` en `assign_equipment` (2 tests nuevos), bitácora on-chain `LogEntry` (`log_milestone` crea PDA histórico, 4 tests en `test_log_entry.rs`), panel de incidente + modal de bitácora en `IncidenteView.vue`
- [x] Rama `main` creada (2026-08-19) y sincronizada en `ghp`/`glp`/`gla` — primera vez en el proyecto
- [x] Fase 5.5 — QA manual completo con Phantom (libreto `simulacion_01.md`, 13/13 fases). Los 5 hallazgos de QA resueltos: #1 y #2 el 2026-08-15; #3 el 2026-08-19 en rama `5_1_equipment_status_view_fixes` (mergeada a `main`); #4 (dashboard por rol) y #5 (bitácora en Inventario + export PDF) el 2026-08-19 en rama `5_2_role_dashboard_and_log_export` (mergeada a `main`)
- [ ] Fase 6 — Deploy Devnet + QA end-to-end (bloqueada hasta terminar validación manual en localnet con Phantom)

**Fase 5.5 cerrada — los 5 hallazgos de QA resueltos y mergeados a `main`. Lista para pasar a Fase 6 (devnet deploy)**
**Tests acumulados: 44 pasando, 0 fallidos — backend Rust/Anchor 100% verificado**
**Adelanto respecto al cronograma: ~35 días**

---

## Reglas del proyecto

- Todo identificador de código en inglés (funciones, structs, enums, variantes, eventos, errores, campos, seeds)
- Comentarios de lógica de negocio pueden estar en español
- Ramas y commits: en inglés (ver sección "Convenciones de código y git")
- No implementar funciones fuera del scope MVP sin discutirlo primero
- Cada fase se testea completamente en localnet antes de mergear a `main`
- Al reanudar sesión: leer este archivo + verificar estado actual antes de escribir código
- Referencia de seguridad del contrato original: `Reference/report.md` sección 7
