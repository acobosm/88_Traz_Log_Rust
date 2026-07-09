/// Flujo E2E completo: inicialización → registro de personal y equipo →
/// apertura de incidente → asignación → reporte de condición → cierre de
/// incidente → retorno de equipo.
///
/// Valida el ciclo de vida íntegro tal como lo haría el contrato Solidity
/// original (`TrazabilidadLogistica.sol`) en Ethereum, ahora en Solana/Anchor.
use {
    anchor_lang::{
        solana_program::{instruction::Instruction, pubkey::Pubkey, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    traz_log::{
        EquipmentStatus, GlobalState, IncidentAccount, EquipmentAccount,
        LogEntry, ReportedCondition, Role,
    },
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn global_state_pda(p: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"global"], p).0
}
fn personnel_pda(w: &Pubkey, p: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"personnel", w.as_ref()], p).0
}
fn equipment_pda(c: &[u8; 32], p: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"equipment", c.as_ref()], p).0
}
fn incident_pda(id: u64, p: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"incident", &id.to_le_bytes()], p).0
}
fn log_entry_pda(code: &[u8; 32], entry_index: u64, p: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"log", code.as_ref(), &entry_index.to_le_bytes()], p).0
}

fn to_code(s: &str) -> [u8; 32] {
    let mut c = [0u8; 32];
    let b = s.as_bytes();
    c[..b.len().min(32)].copy_from_slice(&b[..b.len().min(32)]);
    c
}

fn send(svm: &mut LiteSVM, signer: &Keypair, ix: Instruction) {
    let bh = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&signer.pubkey()), &bh);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[signer]).unwrap();
    svm.send_transaction(tx).expect("E2E step failed");
}

fn read_gs(svm: &LiteSVM, p: &Pubkey) -> GlobalState {
    let acc = svm.get_account(&global_state_pda(p)).unwrap();
    let mut d: &[u8] = &acc.data;
    GlobalState::try_deserialize(&mut d).unwrap()
}
fn read_inc(svm: &LiteSVM, id: u64, p: &Pubkey) -> IncidentAccount {
    let acc = svm.get_account(&incident_pda(id, p)).unwrap();
    let mut d: &[u8] = &acc.data;
    IncidentAccount::try_deserialize(&mut d).unwrap()
}
fn read_eq(svm: &LiteSVM, c: &[u8; 32], p: &Pubkey) -> EquipmentAccount {
    let acc = svm.get_account(&equipment_pda(c, p)).unwrap();
    let mut d: &[u8] = &acc.data;
    EquipmentAccount::try_deserialize(&mut d).unwrap()
}

// ── Test ──────────────────────────────────────────────────────────────────────

#[test]
fn test_complete_firefighting_incident_lifecycle() {
    let program_id = traz_log::id();
    let admin         = Keypair::new();
    let scene_cmd     = Keypair::new();
    let operator      = Keypair::new();
    let op_base       = Keypair::new();
    let code          = to_code("ENGINE-01");

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/traz_log.so");
    svm.add_program(program_id, bytes).unwrap();
    for kp in [&admin, &scene_cmd, &operator, &op_base] {
        svm.airdrop(&kp.pubkey(), 10_000_000_000).unwrap();
    }

    // ── Paso 1: Inicializar sistema ──────────────────────────────────────────
    send(&mut svm, &admin, Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::Initialize {}.data(),
        traz_log::accounts::Initialize {
            admin: admin.pubkey(),
            global_state: global_state_pda(&program_id),
            system_program: system_program::ID,
        }.to_account_metas(None),
    ));

    let gs = read_gs(&svm, &program_id);
    assert_eq!(gs.admin, admin.pubkey(), "admin incorrecto en GlobalState");
    assert!(!gs.is_paused, "sistema debe estar activo tras inicialización");
    assert_eq!(gs.next_incident_id, 0, "contador de incidentes debe iniciar en 0");

    // ── Paso 2: Registrar personal ──────────────────────────────────────────
    for (wallet, role, name) in [
        (scene_cmd.pubkey(), Role::SceneCommander, "Cmdr. García"),
        (operator.pubkey(),  Role::Operator,       "Op. Rodríguez"),
        (op_base.pubkey(),   Role::OperationalBase,"Base Logística"),
    ] {
        send(&mut svm, &admin, Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterPersonnel {
                name: name.to_string(), specialty: "Firefighting".to_string(), role,
            }.data(),
            traz_log::accounts::RegisterPersonnel {
                signer: admin.pubkey(),
                global_state: global_state_pda(&program_id),
                new_personnel: personnel_pda(&wallet, &program_id),
                wallet,
                system_program: system_program::ID,
            }.to_account_metas(None),
        ));
    }

    assert!(svm.get_account(&personnel_pda(&scene_cmd.pubkey(), &program_id)).is_some());
    assert!(svm.get_account(&personnel_pda(&operator.pubkey(),  &program_id)).is_some());
    assert!(svm.get_account(&personnel_pda(&op_base.pubkey(),   &program_id)).is_some());

    // ── Paso 3: Registrar equipo ─────────────────────────────────────────────
    send(&mut svm, &op_base, Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::RegisterEquipment {
            code,
            description: "Autobomba pesada".to_string(),
            nominal_consumption: 1200,
        }.data(),
        traz_log::accounts::RegisterEquipment {
            signer: op_base.pubkey(),
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&op_base.pubkey(), &program_id),
            equipment: equipment_pda(&code, &program_id),
            system_program: system_program::ID,
        }.to_account_metas(None),
    ));

    let eq = read_eq(&svm, &code, &program_id);
    assert_eq!(eq.status, EquipmentStatus::Available, "equipo recién registrado debe ser Available");
    assert_eq!(eq.nominal_consumption, 1200);

    // ── Paso 4: Abrir incidente 0 ────────────────────────────────────────────
    send(&mut svm, &scene_cmd, Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::OpenFireIncident {
            incident_id: 0,
            description: "Test Incident".to_string(),
            coordinates: "9.9281,-84.0907".to_string(),
            risk_level: 4,
        }.data(),
        traz_log::accounts::OpenFireIncident {
            signer: scene_cmd.pubkey(),
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&scene_cmd.pubkey(), &program_id),
            incident: incident_pda(0, &program_id),
            system_program: system_program::ID,
        }.to_account_metas(None),
    ));

    let inc = read_inc(&svm, 0, &program_id);
    assert!(inc.is_active, "incidente debe estar activo tras apertura");
    assert_eq!(inc.risk_level, 4);
    assert_eq!(inc.commander, scene_cmd.pubkey());
    assert_eq!(read_gs(&svm, &program_id).next_incident_id, 1, "contador debe ser 1 tras abrir incidente 0");

    // ── Paso 5: Asignar equipo al operador ───────────────────────────────────
    send(&mut svm, &scene_cmd, Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::AssignEquipment {
            equipment_code: code, incident_id: 0,
        }.data(),
        traz_log::accounts::AssignEquipment {
            signer: scene_cmd.pubkey(),
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&scene_cmd.pubkey(), &program_id),
            equipment: equipment_pda(&code, &program_id),
            incident: incident_pda(0, &program_id),
            operator_personnel: personnel_pda(&operator.pubkey(), &program_id),
            operator_wallet: operator.pubkey(),
        }.to_account_metas(None),
    ));

    let eq = read_eq(&svm, &code, &program_id);
    assert_eq!(eq.status, EquipmentStatus::InUse, "equipo asignado debe estar InUse");
    assert_eq!(eq.custodian, operator.pubkey(), "custodio debe ser el operador asignado");
    assert_eq!(eq.incident_id, 0);

    // ── Paso 6: Operador reporta condición del equipo ────────────────────────
    // log_count es 0 — primer reporte para este equipo
    send(&mut svm, &operator, Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::LogMilestone {
            equipment_code: code,
            notes: "Bomba operando con presión reducida".to_string(),
            condition: ReportedCondition::MinorDamage,
        }.data(),
        traz_log::accounts::LogMilestone {
            signer: operator.pubkey(),
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&operator.pubkey(), &program_id),
            equipment: equipment_pda(&code, &program_id),
            log_entry: log_entry_pda(&code, 0, &program_id),
            system_program: system_program::ID,
        }.to_account_metas(None),
    ));

    let eq_after_log = read_eq(&svm, &code, &program_id);
    assert_eq!(eq_after_log.reported_condition, ReportedCondition::MinorDamage, "condición reportada debe ser MinorDamage");
    assert_eq!(eq_after_log.log_count, 1, "log_count debe ser 1 tras primer reporte");

    // Verificar que la LogEntry fue creada con los datos correctos
    let log_acc = svm.get_account(&log_entry_pda(&code, 0, &program_id)).unwrap();
    let mut log_data: &[u8] = &log_acc.data;
    let entry = LogEntry::try_deserialize(&mut log_data).unwrap();
    assert_eq!(entry.entry_index, 0);
    assert_eq!(entry.condition, ReportedCondition::MinorDamage);
    assert_eq!(entry.operator, operator.pubkey());

    // ── Paso 7: Comandante cierra el incidente ───────────────────────────────
    // Diseño lazy-close: el incidente se cierra aunque el equipo siga en campo
    send(&mut svm, &scene_cmd, Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::CloseIncident { incident_id: 0 }.data(),
        traz_log::accounts::CloseIncident {
            signer: scene_cmd.pubkey(),
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&scene_cmd.pubkey(), &program_id),
            incident: incident_pda(0, &program_id),
        }.to_account_metas(None),
    ));

    assert!(!read_inc(&svm, 0, &program_id).is_active, "incidente debe estar inactivo tras cierre");
    // El equipo sigue InUse aunque el incidente esté cerrado (lazy-close)
    assert_eq!(
        read_eq(&svm, &code, &program_id).status,
        EquipmentStatus::InUse,
        "equipo permanece InUse hasta que el operador inicie retorno"
    );

    // ── Paso 8: Operador inicia retorno del equipo ───────────────────────────
    send(&mut svm, &operator, Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::InitiateReturn { equipment_code: code }.data(),
        traz_log::accounts::InitiateReturn {
            signer: operator.pubkey(),
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&operator.pubkey(), &program_id),
            equipment: equipment_pda(&code, &program_id),
        }.to_account_metas(None),
    ));

    assert_eq!(
        read_eq(&svm, &code, &program_id).status,
        EquipmentStatus::Returning,
        "equipo debe estar Returning tras initiate_return"
    );

    // ── Estado final ─────────────────────────────────────────────────────────
    let gs_final = read_gs(&svm, &program_id);
    assert_eq!(gs_final.next_incident_id, 1, "GlobalState debe reflejar 1 incidente registrado");
    assert!(!gs_final.is_paused, "sistema debe seguir activo al finalizar el ciclo");
}
