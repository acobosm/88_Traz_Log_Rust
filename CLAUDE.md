# CLAUDE.md — FireOPS Traz_Log (Rust/Anchor)

## Descripción del proyecto
Traducción de `TrazabilidadLogistica.sol` (Solidity/Foundry) a Rust/Anchor + Phantom Wallet.
Sistema de trazabilidad logística para operaciones de combate de incendios forestales.

**Contrato original:** `03 Ethereum Practice/.../88_Traz_Log/contracts/TrazabilidadLogistica.sol` (635 líneas)
**Directorio Rust:** `/home/ebit/projects/0 CodeCrypto Academy/04_Rust_Practice/88_Traz_Log/`
**Programa Anchor:** `traz_log` (subdirectorio `88_Traz_Log/traz_log/`)
**Plazo:** 50 días calendario — Jul 4 al Aug 22, 2026

---

## Stack
- Anchor CLI 1.0.2
- Solana CLI 3.1.10 (Agave)
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
- Bitácora on-chain: `LogEntry` PDAs — posible en `7_onchain_log`
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
| Bitácora | **`emit!` (MVP)** | Sin `LogEntry` PDAs. La bitácora histórica queryable puede agregarse en `7_onchain_log`. |
| `register_equipment_batch` | **Eliminado en MVP** | Reemplazado por registros individuales. El batch de 51 items requiere `signAllTransactions` — complejidad no esencial para el MVP. |
| RBAC | **Campo `role: Role` en `PersonnelAccount`** | Sin `AccessControl` de OZ. Verificación via `constraint =` en cada Accounts struct. |
| Roles activos | **4 roles** | Admin, OperationalBase, SceneCommander, Operator. Auditor y Consultant fuera del MVP. |
| `tx.origin` de Solidity | **Eliminado** | En Solana el signer es siempre explícito. Elimina la vulnerabilidad Media del contrato original. |

---

## Plan de fases y ramas git

| # | Rama | Contenido | Fechas |
|---|---|---|---|
| 0 | `0_anchor_fundamentals` | Tutoriales counter + escrow, `anchor init traz_log`, `.gitignore`, `anchor build` verde | Jul 4–13 |
| 1 | `1_account_architecture` | 4 account structs con tamaños, enums, seeds, 8 stubs vacíos que compilan | Jul 13–18 |
| 2 | `2_registration_and_inventory` | `toggle_pause`, `register_personnel`, `register_equipment` + tests TS | Jul 18–22 |
| 3 | `3_incident_management` | `open_fire_incident`, `assign_equipment`, `log_milestone` + tests TS | Jul 22–29 |
| 4 | `4_return_and_close` | `initiate_return`, `close_incident` + tests TS + flujo E2E completo | Jul 29–Aug 5 |
| 5 | `5_phantom_frontend` | Vue.js reescrito: wallet adapter, paneles por rol, `getProgramAccounts`, event listeners | Aug 5–17 |
| 6 | `6_devnet_deploy` | `anchor deploy devnet`, QA manual con Phantom, verificar rent, correcciones UX | Aug 17–22 |
| *(opt)* | `7_onchain_log` | `LogEntry` PDAs, bitácora histórica queryable | Si sobran días |

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

## Estrategia de push a `gla` (simula desarrollo de 30 días)

Por acuerdo con los profesores: desarrollo real en julio (repos personales), push a `gla` desde el 3 de agosto distribuido para simular progresión de 30 días.

| Fecha push | Qué sube a `gla` | Simula en el "proyecto 30 días" |
|---|---|---|
| Aug 3 | `0_anchor_fundamentals` + `main` | Semana 1: setup |
| Aug 6 | `1_account_architecture` + `main` | Días 7–10: arquitectura |
| Aug 9 | `2_registration_and_inventory` + `main` | Días 10–14: registro |
| Aug 12 | `3_incident_management` + `main` | Días 14–21: incidentes |
| Aug 15 | `4_return_and_close` + `main` | Días 21–25: retorno/cierre |
| Aug 19 | `5_phantom_frontend` + `main` | Días 25–29: frontend |
| Aug 22 | `6_devnet_deploy` + `main` | Días 29–30: deploy |

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
- [x] 4 account structs + 3 enums + 5 eventos + 13 errores en `state.rs`, `events.rs`, `error.rs`
- [x] Remoto `ghp` → `https://github.com/acobosm/88_Traz_Log_Rust.git`
- [x] Remoto `glp` → `https://gitlab.com/acobosm1/web3-blockchain/88_traz_log_rust.git`
- [ ] Remoto `gla` → pendiente (configurar desde agosto 3, 2026)
- [x] Fase 0 completada — 3 tests en verde (`test_initialize.rs`)
- [x] Fase 1 completada — 5 tests en verde (`test_toggle_pause.rs`)
- [x] Fase 2 completada — 10 tests en verde (`test_register_personnel.rs`, `test_register_equipment.rs`)
- [x] Fase 3 completada — 15 tests en verde (`test_open_fire_incident.rs`, `test_assign_equipment.rs`, `test_log_milestone.rs`)
- [x] Fase 4 completada — 10 tests + 1 E2E en verde (`test_initiate_return.rs`, `test_close_incident.rs`, `test_e2e_full_flow.rs`)
- [ ] Fase 5 — Frontend Vue.js + Phantom
- [ ] Fase 6 — Deploy Devnet + QA

**Fase actual: 5_phantom_frontend — pendiente**
**Tests acumulados: 43 pasando, 0 fallidos — backend Rust/Anchor 100% verificado**
**Adelanto respecto al cronograma: ~30 días**

---

## Reglas del proyecto

- Todo identificador de código en inglés (funciones, structs, enums, variantes, eventos, errores, campos, seeds)
- Comentarios de lógica de negocio pueden estar en español
- Ramas y commits: en inglés (ver sección "Convenciones de código y git")
- No implementar funciones fuera del scope MVP sin discutirlo primero
- Cada fase se testea completamente en localnet antes de mergear a `main`
- Al reanudar sesión: leer este archivo + verificar estado actual antes de escribir código
- Referencia de seguridad del contrato original: `Reference/report.md` sección 7
