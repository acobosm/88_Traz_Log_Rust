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
    traz_log::{EquipmentAccount, LogEntry, ReportedCondition, Role},
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
fn log_entry_pda(c: &[u8; 32], idx: u64, p: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"log", c.as_ref(), &idx.to_le_bytes()], p).0
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
    svm.send_transaction(tx).expect("transaction failed");
}

fn read_equipment(svm: &LiteSVM, code: &[u8; 32], program_id: &Pubkey) -> EquipmentAccount {
    let acc = svm.get_account(&equipment_pda(code, program_id)).unwrap();
    let mut d: &[u8] = &acc.data;
    EquipmentAccount::try_deserialize(&mut d).unwrap()
}

fn read_log_entry(svm: &LiteSVM, code: &[u8; 32], idx: u64, program_id: &Pubkey) -> LogEntry {
    let acc = svm.get_account(&log_entry_pda(code, idx, program_id)).unwrap();
    let mut d: &[u8] = &acc.data;
    LogEntry::try_deserialize(&mut d).unwrap()
}

fn log_milestone_ix(
    operator: &Keypair,
    code: [u8; 32],
    notes: &str,
    condition: ReportedCondition,
    log_count: u64,
    program_id: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::LogMilestone {
            equipment_code: code,
            notes: notes.to_string(),
            condition,
        }
        .data(),
        traz_log::accounts::LogMilestone {
            signer: operator.pubkey(),
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&operator.pubkey(), &program_id),
            equipment: equipment_pda(&code, &program_id),
            log_entry: log_entry_pda(&code, log_count, &program_id),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

/// Setup: GlobalState + personal + equipo + incidente + equipo asignado al operator.
fn setup() -> (LiteSVM, Keypair, [u8; 32], Pubkey) {
    let program_id = traz_log::id();
    let admin = Keypair::new();
    let scene_cmd = Keypair::new();
    let operator = Keypair::new();
    let op_base = Keypair::new();
    let code = to_code("BRAVO-LOG");
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/traz_log.so");
    svm.add_program(program_id, bytes).unwrap();
    for kp in [&admin, &scene_cmd, &operator, &op_base] {
        svm.airdrop(&kp.pubkey(), 10_000_000_000).unwrap();
    }

    send(&mut svm, &admin, Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::Initialize {}.data(),
        traz_log::accounts::Initialize {
            admin: admin.pubkey(),
            global_state: global_state_pda(&program_id),
            system_program: system_program::ID,
        }.to_account_metas(None),
    ));

    for (wallet, role, name) in [
        (scene_cmd.pubkey(), Role::SceneCommander, "Commander"),
        (operator.pubkey(),  Role::Operator,       "Operator"),
        (op_base.pubkey(),   Role::OperationalBase, "OpsBase"),
    ] {
        send(&mut svm, &admin, Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterPersonnel {
                name: name.to_string(), specialty: "Test".to_string(), role,
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

    send(&mut svm, &op_base, Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::RegisterEquipment {
            code, description: "Equipo bravo".to_string(), nominal_consumption: 500,
        }.data(),
        traz_log::accounts::RegisterEquipment {
            signer: op_base.pubkey(),
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&op_base.pubkey(), &program_id),
            equipment: equipment_pda(&code, &program_id),
            system_program: system_program::ID,
        }.to_account_metas(None),
    ));

    send(&mut svm, &scene_cmd, Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::OpenFireIncident {
            incident_id: 0, description: "Test Incident".to_string(), coordinates: "0.0,0.0".to_string(), risk_level: 2,
        }.data(),
        traz_log::accounts::OpenFireIncident {
            signer: scene_cmd.pubkey(),
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&scene_cmd.pubkey(), &program_id),
            incident: incident_pda(0, &program_id),
            system_program: system_program::ID,
        }.to_account_metas(None),
    ));

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

    (svm, operator, code, program_id)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_log_milestone_creates_log_entry() {
    let (mut svm, operator, code, program_id) = setup();

    send(&mut svm, &operator, log_milestone_ix(
        &operator, code, "Primer reporte", ReportedCondition::Operational, 0, program_id,
    ));

    assert!(
        svm.get_account(&log_entry_pda(&code, 0, &program_id)).is_some(),
        "LogEntry PDA debe existir tras log_milestone"
    );
}

#[test]
fn test_log_entry_fields_are_correct() {
    let (mut svm, operator, code, program_id) = setup();

    send(&mut svm, &operator, log_milestone_ix(
        &operator, code, "Manguera con desgaste", ReportedCondition::MinorDamage, 0, program_id,
    ));

    let entry = read_log_entry(&svm, &code, 0, &program_id);
    assert_eq!(entry.entry_index, 0);
    assert_eq!(entry.equipment_code, code);
    assert_eq!(entry.notes, "Manguera con desgaste");
    assert_eq!(entry.condition, ReportedCondition::MinorDamage);
    assert_eq!(entry.operator, operator.pubkey());
    // LiteSVM inicia el clock en unix_timestamp=0; el campo se escribe correctamente
    assert!(entry.timestamp >= 0, "timestamp debe ser un valor unix válido");
}

#[test]
fn test_log_count_increments_on_each_milestone() {
    let (mut svm, operator, code, program_id) = setup();

    assert_eq!(read_equipment(&svm, &code, &program_id).log_count, 0);

    send(&mut svm, &operator, log_milestone_ix(
        &operator, code, "Reporte 1", ReportedCondition::Operational, 0, program_id,
    ));
    assert_eq!(read_equipment(&svm, &code, &program_id).log_count, 1);

    svm.expire_blockhash();
    send(&mut svm, &operator, log_milestone_ix(
        &operator, code, "Reporte 2", ReportedCondition::MinorDamage, 1, program_id,
    ));
    assert_eq!(read_equipment(&svm, &code, &program_id).log_count, 2);
}

#[test]
fn test_multiple_log_entries_have_sequential_indices() {
    let (mut svm, operator, code, program_id) = setup();

    send(&mut svm, &operator, log_milestone_ix(
        &operator, code, "Estado inicial", ReportedCondition::Operational, 0, program_id,
    ));
    svm.expire_blockhash();
    send(&mut svm, &operator, log_milestone_ix(
        &operator, code, "Daño detectado", ReportedCondition::CriticalDamage, 1, program_id,
    ));

    let entry0 = read_log_entry(&svm, &code, 0, &program_id);
    let entry1 = read_log_entry(&svm, &code, 1, &program_id);

    assert_eq!(entry0.entry_index, 0);
    assert_eq!(entry0.condition, ReportedCondition::Operational);
    assert_eq!(entry1.entry_index, 1);
    assert_eq!(entry1.condition, ReportedCondition::CriticalDamage);
    assert_eq!(entry1.notes, "Daño detectado");
}
