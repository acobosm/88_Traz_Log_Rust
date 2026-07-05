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
    traz_log::{EquipmentAccount, ReportedCondition, Role},
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn global_state_pda(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"global"], program_id).0
}

fn personnel_pda(wallet: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"personnel", wallet.as_ref()], program_id).0
}

fn equipment_pda(code: &[u8; 32], program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"equipment", code.as_ref()], program_id).0
}

fn incident_pda(incident_id: u64, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"incident", &incident_id.to_le_bytes()], program_id).0
}

fn to_code(s: &str) -> [u8; 32] {
    let mut code = [0u8; 32];
    let bytes = s.as_bytes();
    let len = bytes.len().min(32);
    code[..len].copy_from_slice(&bytes[..len]);
    code
}

fn send_ix(svm: &mut LiteSVM, signer: &Keypair, ix: Instruction) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&signer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[signer]).unwrap();
    svm.send_transaction(tx).expect("send_ix: transaction failed");
}

fn try_send_ix(svm: &mut LiteSVM, signer: &Keypair, ix: Instruction) -> bool {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&signer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[signer]).unwrap();
    svm.send_transaction(tx).is_ok()
}

/// Setup completo: GlobalState + personal + equipo + incidente + equipo asignado al operator.
/// Retorna: (svm, admin, scene_commander, operator, equipment_code, program_id)
fn setup() -> (LiteSVM, Keypair, Keypair, Keypair, [u8; 32], Pubkey) {
    let program_id = traz_log::id();
    let admin = Keypair::new();
    let scene_commander = Keypair::new();
    let operator = Keypair::new();
    let op_base = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/traz_log.so");
    svm.add_program(program_id, bytes).unwrap();
    for kp in [&admin, &scene_commander, &operator, &op_base] {
        svm.airdrop(&kp.pubkey(), 10_000_000_000).unwrap();
    }

    // 1. Inicializar
    send_ix(
        &mut svm,
        &admin,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::Initialize {}.data(),
            traz_log::accounts::Initialize {
                admin: admin.pubkey(),
                global_state: global_state_pda(&program_id),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    // 2. Registrar personal
    for (wallet, role, name) in [
        (scene_commander.pubkey(), Role::SceneCommander, "Commander"),
        (operator.pubkey(), Role::Operator, "Operator"),
        (op_base.pubkey(), Role::OperationalBase, "OpsBase"),
    ] {
        send_ix(
            &mut svm,
            &admin,
            Instruction::new_with_bytes(
                program_id,
                &traz_log::instruction::RegisterPersonnel {
                    name: name.to_string(),
                    specialty: "Test".to_string(),
                    role,
                }
                .data(),
                traz_log::accounts::RegisterPersonnel {
                    signer: admin.pubkey(),
                    global_state: global_state_pda(&program_id),
                    new_personnel: personnel_pda(&wallet, &program_id),
                    wallet,
                    system_program: system_program::ID,
                }
                .to_account_metas(None),
            ),
        );
    }

    // 3. Registrar equipo
    let code = to_code("MOTOR-BRAVO");
    send_ix(
        &mut svm,
        &op_base,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterEquipment {
                code,
                description: "Motor bravo".to_string(),
                nominal_consumption: 600,
            }
            .data(),
            traz_log::accounts::RegisterEquipment {
                signer: op_base.pubkey(),
                global_state: global_state_pda(&program_id),
                signer_personnel: personnel_pda(&op_base.pubkey(), &program_id),
                equipment: equipment_pda(&code, &program_id),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    // 4. Abrir incidente 0
    send_ix(
        &mut svm,
        &scene_commander,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::OpenFireIncident {
                incident_id: 0,
                coordinates: "10.0,-84.0".to_string(),
                risk_level: 2,
            }
            .data(),
            traz_log::accounts::OpenFireIncident {
                signer: scene_commander.pubkey(),
                global_state: global_state_pda(&program_id),
                signer_personnel: personnel_pda(&scene_commander.pubkey(), &program_id),
                incident: incident_pda(0, &program_id),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    // 5. Asignar equipo al operator
    send_ix(
        &mut svm,
        &scene_commander,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::AssignEquipment {
                equipment_code: code,
                incident_id: 0,
            }
            .data(),
            traz_log::accounts::AssignEquipment {
                signer: scene_commander.pubkey(),
                global_state: global_state_pda(&program_id),
                signer_personnel: personnel_pda(&scene_commander.pubkey(), &program_id),
                equipment: equipment_pda(&code, &program_id),
                incident: incident_pda(0, &program_id),
                operator_personnel: personnel_pda(&operator.pubkey(), &program_id),
                operator_wallet: operator.pubkey(),
            }
            .to_account_metas(None),
        ),
    );

    (svm, admin, scene_commander, operator, code, program_id)
}

fn log_milestone_ix(
    operator_pubkey: Pubkey,
    code: [u8; 32],
    program_id: Pubkey,
    condition: ReportedCondition,
) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::LogMilestone {
            equipment_code: code,
            notes: "Field report".to_string(),
            condition,
        }
        .data(),
        traz_log::accounts::LogMilestone {
            signer: operator_pubkey,
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&operator_pubkey, &program_id),
            equipment: equipment_pda(&code, &program_id),
        }
        .to_account_metas(None),
    )
}

fn read_equipment(svm: &LiteSVM, code: &[u8; 32], program_id: &Pubkey) -> EquipmentAccount {
    let account = svm.get_account(&equipment_pda(code, program_id)).unwrap();
    let mut data: &[u8] = &account.data;
    EquipmentAccount::try_deserialize(&mut data).unwrap()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_custodian_can_log_milestone() {
    let (mut svm, _admin, _sc, operator, code, program_id) = setup();

    let ix = log_milestone_ix(operator.pubkey(), code, program_id, ReportedCondition::MinorDamage);
    send_ix(&mut svm, &operator, ix);

    let e = read_equipment(&svm, &code, &program_id);
    assert_eq!(e.reported_condition, ReportedCondition::MinorDamage);
}

#[test]
fn test_milestone_updates_reported_condition() {
    let (mut svm, _admin, _sc, operator, code, program_id) = setup();

    // Estado inicial: Operational
    assert_eq!(
        read_equipment(&svm, &code, &program_id).reported_condition,
        ReportedCondition::Operational
    );

    // Reportar CriticalDamage
    send_ix(
        &mut svm,
        &operator,
        log_milestone_ix(operator.pubkey(), code, program_id, ReportedCondition::CriticalDamage),
    );

    assert_eq!(
        read_equipment(&svm, &code, &program_id).reported_condition,
        ReportedCondition::CriticalDamage,
        "reported_condition must reflect the last logged milestone"
    );
}

#[test]
fn test_non_custodian_cannot_log_milestone() {
    let (mut svm, admin, _sc, _operator, code, program_id) = setup();

    // Registrar un segundo operador (no es custodio del equipo)
    let intruder = Keypair::new();
    svm.airdrop(&intruder.pubkey(), 10_000_000_000).unwrap();
    send_ix(
        &mut svm,
        &admin,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterPersonnel {
                name: "Intruder Op".to_string(),
                specialty: "Test".to_string(),
                role: Role::Operator,
            }
            .data(),
            traz_log::accounts::RegisterPersonnel {
                signer: admin.pubkey(),
                global_state: global_state_pda(&program_id),
                new_personnel: personnel_pda(&intruder.pubkey(), &program_id),
                wallet: intruder.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    let ix = log_milestone_ix(intruder.pubkey(), code, program_id, ReportedCondition::MinorDamage);
    assert!(
        !try_send_ix(&mut svm, &intruder, ix),
        "non-custodian operator must not be able to log a milestone"
    );
}

#[test]
fn test_wrong_role_cannot_log_milestone() {
    let (mut svm, _admin, scene_commander, _operator, code, program_id) = setup();

    // scene_commander tiene rol SceneCommander, no Operator
    let ix = log_milestone_ix(scene_commander.pubkey(), code, program_id, ReportedCondition::Lost);
    assert!(
        !try_send_ix(&mut svm, &scene_commander, ix),
        "SceneCommander role must not be able to log a milestone"
    );
}
