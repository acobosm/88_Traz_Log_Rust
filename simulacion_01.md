# Simulación 01 — QA manual en localnet con Phantom

Libreto para probar a mano, navegador + Phantom, la lógica de la Fase 5 (incluye el
adelanto de `LogEntry` y los guardrails `NotIncidentCommander` / `OperatorAlreadyAssigned`).
No usa `scripts/seed.mjs` — todo se hace clic a clic desde la app, como lo haría el
equipo real un día de turno.

Convención de esta guía:
- ✅ **Debe funcionar** — si falla, hay un bug.
- ⛔ **Debe fallar** — si pasa, hay un bug (el guardrail no está funcionando).
- 👁 **Verificar** — qué mirar en la UI para confirmar el resultado antes de seguir.

---

## 0. Prerrequisitos — levantar el entorno

### 0.1 Validador + build + deploy (terminal 1)

Anchor CLI 1.0.2 usa **Surfpool** como validador local por defecto (no el
`solana-test-validator` clásico). Un solo comando levanta el validador, compila y
despliega el programa:

```bash
cd "/home/ebit/projects/0 CodeCrypto Academy/04_Rust_Practice/88_Traz_Log/traz_log"
anchor test --detach
```

`--detach` deja el validador corriendo después de los tests (sin esa flag, Anchor lo
apaga al terminar) y libera la terminal para reutilizarla en el paso 0.5. Confirma que
el Program ID desplegado coincide con `declare_id!` en `lib.rs`
(`13p4xV6WHaPwWzno1F6Z6MeY9b9wLYtEMAnUF8ofdW75`).

### 0.2 Surfpool Studio — ver transacciones en tiempo real

Abre `http://127.0.0.1:18488` en el navegador. Ahí se ven en vivo todas las
transacciones que se vayan generando durante el libreto — útil para verificar cada
paso sin depender solo de la UI de la app.

### 0.3 Frontend (terminal 2)

```bash
cd "/home/ebit/projects/0 CodeCrypto Academy/04_Rust_Practice/88_Traz_Log/app"
npm run dev
```

Abre la URL que imprime Vite (normalmente `http://localhost:5173`).

### 0.4 Phantom apuntando a localnet

En Phantom: **Settings → Developer Settings → Testnet Mode** (activarlo), luego en el
selector de red de arriba elige **Solana** y configura el RPC personalizado a
`http://localhost:8899` (si tu versión de Phantom ya trae "Localhost" como preset, úsalo
directo).

### 0.5 Generar las 5 wallets de prueba (terminal 1, con el validador corriendo en background)

El Admin puede ser tu wallet CLI existente (`~/.config/solana/id.json`, la que ya usa
`Anchor.toml`). Genera las otras 5:

```bash
mkdir -p ~/.config/solana/roles
solana-keygen new --no-bip39-passphrase --outfile ~/.config/solana/roles/jefe1.json
solana-keygen new --no-bip39-passphrase --outfile ~/.config/solana/roles/jefe2.json
solana-keygen new --no-bip39-passphrase --outfile ~/.config/solana/roles/base1.json
solana-keygen new --no-bip39-passphrase --outfile ~/.config/solana/roles/operador1.json
solana-keygen new --no-bip39-passphrase --outfile ~/.config/solana/roles/operador2.json
```

Airdropea SOL a cada una (y a tu wallet Admin si el validador es nuevo):

```bash
solana config set --url http://localhost:8899
solana airdrop 10                                              # Admin (id.json)
solana airdrop 10 $(solana-keygen pubkey ~/.config/solana/roles/jefe1.json)
solana airdrop 10 $(solana-keygen pubkey ~/.config/solana/roles/jefe2.json)
solana airdrop 10 $(solana-keygen pubkey ~/.config/solana/roles/base1.json)
solana airdrop 10 $(solana-keygen pubkey ~/.config/solana/roles/operador1.json)
solana airdrop 10 $(solana-keygen pubkey ~/.config/solana/roles/operador2.json)
```

### 0.6 Importar cada wallet en Phantom

Phantom permite tener varias cuentas bajo la misma extensión y cambiar entre ellas.
Para cada rol (jefe1, jefe2, base1, operador1, operador2):

1. `cat ~/.config/solana/roles/jefe1.json` → copia el arreglo de bytes completo.
2. En Phantom: menú de cuentas → **Add / Connect Wallet → Import Private Key**.
3. Pega el arreglo (Phantom lo detecta como Solana secret key), ponle un nombre
   reconocible (ej. "Jefe1 — Sofía").
4. Repite para las otras 4.

Tu wallet Admin (CLI) también puedes importarla a Phantom si quieres operar el panel
Admin desde el navegador, o dejar esa parte solo por CLI/tests — para este libreto sí
la necesitas en Phantom porque `initialize` y `register_personnel` se hacen desde
**AdminView**.

---

## 1. Reparto de personajes

| Rol en la app | Nombre en la historia | Wallet | Usada en |
|---|---|---|---|
| Admin | — (tu wallet CLI) | `id.json` | Inicializar, registrar personal |
| SceneCommander | **Sofía Ramírez** — Jefe de Escena 1 | `jefe1.json` | Incidente #0, luego #2 |
| SceneCommander | **Andrés Paredes** — Jefe de Escena 2 | `jefe2.json` | Incidente #1 |
| OperationalBase | **Marta Chávez** — Base Operativa | `base1.json` | Registra todo el equipo |
| Operator | **Diego Salazar** — Operador 1 | `operador1.json` | Incidente #0 → #2 |
| Operator | **Valeria Núñez** — Operador 2 | `operador2.json` | Incidente #1 |

Equipo que se registrará (todo por Marta, en **Admin → Registrar equipo**):

| Código | Descripción | Consumo (L/h) | Usado en |
|---|---|---|---|
| `BOMBA-01` | Motobomba 01 | 4000 | Incidente #0 |
| `RADIO-01` | Radio Motorola 01 | 0 | Incidente #0 |
| `GPS-01` | GPS Garmin 01 | 0 | Incidente #0 |
| `BOMBA-02` | Motobomba 02 | 4000 | Incidente #1 |
| `VEHICULO4X4-01` | Vehículo 4x4 01 | 0 | Incidente #2 |
| `BATEFUEGOS-01` | Batefuegos 01 | 0 | Incidente #2 |

> Nota: una vez que un equipo pasa a `Returning` no hay instrucción que lo regrese a
> `Available` (fuera del alcance del MVP). Por eso el incidente #2 usa equipo nuevo en
> vez de reciclar `BOMBA-01`.

> Nota 2: el programa no obliga a cerrar un incidente antes de abrir otro (no hay
> límite de incidentes activos simultáneos). En este libreto cerramos el #0 antes de
> abrir el #2 solo por realismo — Sofía no puede comandar dos incendios a la vez.

---

## 2. Fase 0 — Setup (wallet Admin)

Conecta Phantom con la wallet **Admin**. En la parte superior de la APP, ve a la pestaña **Admin**.

1. ✅ **Initialize Sistema** — botón "Initialize Sistema". Debe quedar como admin tu wallet.
   👁 Verifica en **Dashboard**: estado "Activo", admin = tu pubkey.
2. ✅ Registrar personal, uno por uno (**Registrar personal**):
   - Wallet de Sofía, nombre "Sofía Ramírez", especialidad "Logística", rol `SceneCommander`.
   - Wallet de Andrés, nombre "Andrés Paredes", especialidad "Logística", rol `SceneCommander`.
   - Wallet de Marta, nombre "Marta Chávez", especialidad "Base Operativa", rol `OperationalBase`.
   - Wallet de Diego, nombre "Diego Salazar", especialidad "Rescate", rol `Operator`.
   - Wallet de Valeria, nombre "Valeria Núñez", especialidad "Rescate", rol `Operator`.
   👁 Verifica en **Inventario** → tabla de personal: 5 filas con los roles correctos.

---

## 3. Fase 1 — Inventario inicial (wallet Marta / Base Operativa)

Cambia Phantom a **Marta**. Ve a **Admin → Registrar equipo** y registra:

1. ✅ `BOMBA-01` — "Motobomba 01" — 4000
2. ✅ `RADIO-01` — "Radio Motorola 01" — 0
3. ✅ `GPS-01` — "GPS Garmin 01" — 0
4. ✅ `BOMBA-02` — "Motobomba 02" — 4000

(`VEHICULO4X4-01` y `BATEFUEGOS-01` los registras más adelante, en la Fase 9, cuando
surja el tercer incendio — así también probamos que se puede seguir dando de alta
inventario en cualquier momento, no solo al principio.)

👁 Verifica en **Inventario**: 4 equipos, todos en estado "Disponible".

---

## 4. Fase 2 — Se abre el incidente #0 (wallet Sofía)

08:30 — Sofía llega al primer llamado del día.

1. Cambia Phantom a **Sofía**. Ve a **Incidente → Abrir incidente**.
2. ✅ Descripción: "Incendio forestal sector Lumbisí". Coordenadas:
   `-0.22427525812065774, -78.49952508364493`. Riesgo: `3`.
   👁 Debe aparecer como incidente `#0` en el panel superior de la vista Incidente.
3. ✅ En **Asignar equipo a operador**: equipo `BOMBA-01`, incidente `#0`, operador Diego.
   👁 Panel del incidente #0 → `BOMBA-01` aparece "En uso", custodio = Diego.

---

## 5. Fase 3 — Se abre el incidente #1 (wallet Andrés)

08:55 — Casi al mismo tiempo, un segundo foco activa a Andrés.

1. Cambia Phantom a **Andrés**. **Incidente → Abrir incidente**.
2. ✅ Descripción: "Incendio forestal sector Cumbayá". Coordenadas:
   `-0.20215294825985808, -78.47531785829314`. Riesgo: `1`.
   👁 Debe crearse como incidente `#1`.

---

## 6. Fase 4 — Error humano #1: Sofía se equivoca de incidente ⛔→✅

09:10 — Sofía sigue conectada, cansada de la llamada anterior, y en el selector de
"Asignar equipo a operador" no se fija bien en cuál incidente tiene seleccionado.

1. Sigue con Phantom en **Sofía**. En **Asignar equipo a operador**:
   equipo `RADIO-01`, incidente **`#1`** (el de Andrés, no el suyo), operador Diego.
   ⛔ **Debe fallar** con `NotIncidentCommander` ("Only the commander who opened this
   incident can assign equipment to it"). Sofía no comanda el #1.
   👁 Verifica el mensaje de error en rojo bajo el formulario.
2. Sofía revisa, se da cuenta del error, y corrige: mismo equipo `RADIO-01`,
   incidente **`#0`** (el suyo), operador Diego.
   ✅ Ahora sí debe asignarse correctamente.
   👁 Panel del incidente #0 → `RADIO-01` también "En uso" bajo Diego.

---

## 7. Fase 5 — Andrés asigna su propio equipo (wallet Andrés)

09:20 — Andrés, en su incidente, asigna equipo a su operadora.

1. Cambia Phantom a **Andrés**. **Asignar equipo a operador**: equipo `BOMBA-02`,
   incidente `#1`, operadora Valeria.
   ✅ Debe funcionar. Valeria queda comprometida con el incidente #1.

---

## 8. Fase 6 — Error humano #2: Sofía pide un refuerzo que ya está ocupado ⛔→✅

10:00 — Sofía necesita más manos y, sin comunicarse antes con Andrés, intenta jalar
a Valeria para su propio incidente.

1. Cambia Phantom a **Sofía**. **Asignar equipo a operador**: equipo `GPS-01`,
   incidente `#0`, operadora **Valeria**.
   ⛔ **Debe fallar** con `OperatorAlreadyAssigned` ("Operator is already assigned to
   a different active incident"). Valeria sigue comprometida con el #1.
   👁 Verifica el mensaje de error.
2. Sofía entiende que Valeria no está libre y en su lugar asigna el mismo `GPS-01`
   a **Diego** (su propio operador, que ya tiene otros dos equipos bajo el mismo
   incidente #0 — esto confirma que un operador puede acumular varios equipos
   *dentro del mismo incidente* sin problema).
   ✅ Debe funcionar.
   👁 Panel del incidente #0 → Diego ahora aparece como custodio de `BOMBA-01`,
   `RADIO-01` y `GPS-01`.

---

## 9. Fase 7 — Reportes de campo (bitácora on-chain)

Mediodía — cada operador reporta el estado real de su equipo. Esta es la primera
prueba de la bitácora `LogEntry` adelantada en Fase 5.

1. Cambia Phantom a **Diego**. Ve a **Campo → Reportar condición de equipo**.
   - ✅ Equipo `BOMBA-01`, condición **Operacional**, notas: "Funcionó sin novedad
     toda la mañana".
   - ✅ Equipo `RADIO-01`, condición **Daño menor**, notas: "Antena doblada al caer,
     sigue transmitiendo".
   👁 Ve a **Incidente → Panel del incidente** (incidente #0) y haz clic en
   "Bitácora" sobre `RADIO-01`: debe verse la entrada `#0` con condición "Daño menor"
   y la nota exacta.
2. Cambia Phantom a **Valeria**. **Campo → Reportar condición de equipo**.
   - ✅ Equipo `BOMBA-02`, condición **Daño crítico**, notas: "Motor se sobrecalentó
     y dejó de bombear agua".
   👁 Panel del incidente #1 → botón "Bitácora" sobre `BOMBA-02` → entrada `#0` con
   "Daño crítico".

(`GPS-01` lo dejamos sin reportar a propósito — no todo equipo necesita bitácora si
no hubo novedad, solo demuestra que reportar es opcional por ítem.)

---

## 10. Fase 8 — Retorno y cierre del incidente #0

13:00 — El incendio de Lumbisí queda controlado. Diego retorna su equipo y Sofía
cierra el incidente.

1. Cambia Phantom a **Diego**. **Campo → Iniciar retorno de equipo**, uno por uno:
   - ✅ `RADIO-01` → Returning
   - ✅ `GPS-01` → Returning
   - ✅ `BOMBA-01` → Returning
   👁 Después del tercero, Diego queda libre (`active_assignments = 0`,
   `current_incident = None` — no hay forma de verlo directo en la UI, pero se
   confirma indirectamente en la Fase 11 cuando puede tomar equipo nuevo).
2. Cambia Phantom a **Sofía**. **Incidente → Cerrar incidente**, selecciona `#0`.
   ✅ Debe cerrarse. 👁 En **Inventario** el incidente #0 pasa a "Cerrado".

---

## 11. Fase 9 — Nuevo inventario para el tercer incidente (wallet Marta)

15:00 — Surge un tercer foco. Como el equipo de Diego quedó en `Returning` (no se
puede reciclar), Marta da de alta equipo nuevo.

1. Cambia Phantom a **Marta**. **Admin → Registrar equipo**:
   - ✅ `VEHICULO4X4-01` — "Vehículo 4x4 01" — 0
   - ✅ `BATEFUEGOS-01` — "Batefuegos 01" — 0

---

## 12. Fase 10 — Incidente #2 (wallet Sofía)

15:15 — Sofía, ya libre del incidente #0, atiende el tercer llamado.

1. Cambia Phantom a **Sofía**. **Incidente → Abrir incidente**: descripción
   "Incendio forestal sector Tumbaco", coordenadas `-0.2100,-78.4000`, riesgo `4`.
   ✅ Debe crearse como incidente `#2` (el `#1` de Andrés sigue activo en paralelo).
2. **Asignar equipo a operador**: `VEHICULO4X4-01`, incidente `#2`, operador **Diego**.
   ✅ Debe funcionar — Diego estaba libre desde la Fase 8.
3. **Asignar equipo a operador**: `BATEFUEGOS-01`, incidente `#2`, operador Diego.
   ✅ Debe funcionar (mismo operador, mismo incidente, sin problema).

---

## 13. Fase 11 — Cierre del incidente #2

17:00 — Incendio controlado sin daños al equipo.

1. Cambia Phantom a **Diego**. **Campo → Reportar condición**: `VEHICULO4X4-01`,
   condición **Operacional**, notas: "Sin novedad".
   ✅
2. **Campo → Iniciar retorno**: `VEHICULO4X4-01` ✅, luego `BATEFUEGOS-01` ✅.
3. Cambia Phantom a **Sofía**. **Incidente → Cerrar incidente** `#2`.
   ✅

---

## 14. Fase 12 — Cierre del incidente #1 (wallet Andrés / Valeria)

17:30 — El incidente de Andrés, el más pequeño (riesgo 1), se cierra al final del día.

1. Cambia Phantom a **Valeria**. **Campo → Iniciar retorno**: `BOMBA-02`.
   ✅ (aunque ya estaba en "Daño crítico", el retorno solo cambia el `status`, no la
   condición reportada — ambos campos son independientes).
2. Cambia Phantom a **Andrés**. **Incidente → Cerrar incidente** `#1`.
   ✅

---

## 15. Checklist final — qué quedó demostrado

- [ ] Guardrail `NotIncidentCommander` disparado y luego corregido (Fase 4)
- [ ] Guardrail `OperatorAlreadyAssigned` disparado y luego corregido (Fase 6)
- [ ] Un operador con varios equipos bajo el mismo incidente (Fase 6, Diego)
- [ ] Bitácora (`LogEntry`) con condición **Operacional** (Fase 7, Fase 11)
- [ ] Bitácora con **Daño menor** (Fase 7, `RADIO-01`)
- [ ] Bitácora con **Daño crítico** (Fase 7, `BOMBA-02`)
- [ ] Equipo sin bitácora reportada, retornado igual sin bloquear el flujo (`GPS-01`)
- [ ] Un operador liberado (`initiate_return` hasta 0 asignaciones) puede tomar
      equipo en un incidente nuevo (Fase 8 → Fase 10, Diego)
- [ ] 3 incidentes abiertos, 3 cerrados, sin reciclar equipo `Returning`
- [ ] Panel de incidente y modal de bitácora reflejan los datos reales on-chain en
      cada paso (no solo el resultado del test automatizado)

Si todos los ✅ pasaron donde debían pasar y todos los ⛔ fallaron donde debían
fallar, la Fase 5.5 queda validada en localnet y se puede pasar a planear la Fase 6
(devnet).
