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
    traz_log::{EquipmentAccount, EquipmentStatus, Role},
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

/// Setup completo: GlobalState + SceneCommander + Operator + OperationalBase
/// + Equipment registrado + Incident 0 abierto. Listo para assign_equipment.
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
                    signer_personnel: personnel_pda(&admin.pubkey(), &program_id),
                    new_personnel: personnel_pda(&wallet, &program_id),
                    wallet,
                    system_program: system_program::ID,
                }
                .to_account_metas(None),
            ),
        );
    }

    // 3. Registrar equipo
    let code = to_code("PUMP-ALPHA");
    send_ix(
        &mut svm,
        &op_base,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterEquipment {
                code,
                description: "Bomba alfa".to_string(),
                nominal_consumption: 800,
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
                description: "Test Incident".to_string(),
                coordinates: "10.0,-84.0".to_string(),
                risk_level: 3,
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

    (svm, admin, scene_commander, operator, code, program_id)
}

fn assign_equipment_ix(
    sc_pubkey: Pubkey,
    operator_pubkey: Pubkey,
    code: [u8; 32],
    incident_id: u64,
    program_id: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::AssignEquipment {
            equipment_code: code,
            incident_id,
        }
        .data(),
        traz_log::accounts::AssignEquipment {
            signer: sc_pubkey,
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&sc_pubkey, &program_id),
            equipment: equipment_pda(&code, &program_id),
            incident: incident_pda(incident_id, &program_id),
            operator_personnel: personnel_pda(&operator_pubkey, &program_id),
            operator_wallet: operator_pubkey,
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
fn test_scene_commander_assigns_equipment_successfully() {
    let (mut svm, _admin, sc, operator, code, program_id) = setup();

    let ix = assign_equipment_ix(sc.pubkey(), operator.pubkey(), code, 0, program_id);
    send_ix(&mut svm, &sc, ix);

    let e = read_equipment(&svm, &code, &program_id);
    assert_eq!(e.status, EquipmentStatus::InUse);
}

#[test]
fn test_equipment_status_becomes_in_use() {
    let (mut svm, _admin, sc, operator, code, program_id) = setup();

    // Antes: Available
    assert_eq!(read_equipment(&svm, &code, &program_id).status, EquipmentStatus::Available);

    send_ix(&mut svm, &sc, assign_equipment_ix(sc.pubkey(), operator.pubkey(), code, 0, program_id));

    // Después: InUse
    assert_eq!(read_equipment(&svm, &code, &program_id).status, EquipmentStatus::InUse);
}

#[test]
fn test_custodian_is_set_to_operator() {
    let (mut svm, _admin, sc, operator, code, program_id) = setup();

    send_ix(&mut svm, &sc, assign_equipment_ix(sc.pubkey(), operator.pubkey(), code, 0, program_id));

    let e = read_equipment(&svm, &code, &program_id);
    assert_eq!(e.custodian, operator.pubkey(), "custodian must be the assigned operator");
    assert_eq!(e.incident_id, 0, "incident_id must be set after assignment");
}

#[test]
fn test_cannot_reassign_in_use_equipment() {
    let (mut svm, _admin, sc, operator, code, program_id) = setup();

    // Primera asignación exitosa
    send_ix(&mut svm, &sc, assign_equipment_ix(sc.pubkey(), operator.pubkey(), code, 0, program_id));

    // Segunda asignación debe fallar (equipo ya está InUse)
    svm.expire_blockhash();
    let ix2 = assign_equipment_ix(sc.pubkey(), operator.pubkey(), code, 0, program_id);
    assert!(!try_send_ix(&mut svm, &sc, ix2), "re-assigning InUse equipment must fail");
}

#[test]
fn test_non_operator_role_cannot_be_assigned() {
    let (mut svm, admin, sc, _operator, code, program_id) = setup();

    // Registrar una persona con rol SceneCommander (no es Operator)
    let bad_assignee = Keypair::new();
    svm.airdrop(&bad_assignee.pubkey(), 10_000_000_000).unwrap();
    send_ix(
        &mut svm,
        &admin,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterPersonnel {
                name: "Wrong Role".to_string(),
                specialty: "Test".to_string(),
                role: Role::SceneCommander,
            }
            .data(),
            traz_log::accounts::RegisterPersonnel {
                signer: admin.pubkey(),
                global_state: global_state_pda(&program_id),
                signer_personnel: personnel_pda(&admin.pubkey(), &program_id),
                new_personnel: personnel_pda(&bad_assignee.pubkey(), &program_id),
                wallet: bad_assignee.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    let ix = assign_equipment_ix(sc.pubkey(), bad_assignee.pubkey(), code, 0, program_id);
    assert!(
        !try_send_ix(&mut svm, &sc, ix),
        "assigning equipment to non-Operator role must fail"
    );
}

#[test]
fn test_scene_commander_cannot_assign_to_incident_they_do_not_command() {
    let (mut svm, admin, sc, operator, code, program_id) = setup();

    // Segundo Jefe de Escena, dueño del incidente 1
    let other_sc = Keypair::new();
    svm.airdrop(&other_sc.pubkey(), 10_000_000_000).unwrap();
    send_ix(
        &mut svm,
        &admin,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterPersonnel {
                name: "Other Commander".to_string(),
                specialty: "Test".to_string(),
                role: Role::SceneCommander,
            }
            .data(),
            traz_log::accounts::RegisterPersonnel {
                signer: admin.pubkey(),
                global_state: global_state_pda(&program_id),
                signer_personnel: personnel_pda(&admin.pubkey(), &program_id),
                new_personnel: personnel_pda(&other_sc.pubkey(), &program_id),
                wallet: other_sc.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );
    send_ix(
        &mut svm,
        &other_sc,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::OpenFireIncident {
                incident_id: 1,
                description: "Other Incident".to_string(),
                coordinates: "11.0,-85.0".to_string(),
                risk_level: 2,
            }
            .data(),
            traz_log::accounts::OpenFireIncident {
                signer: other_sc.pubkey(),
                global_state: global_state_pda(&program_id),
                signer_personnel: personnel_pda(&other_sc.pubkey(), &program_id),
                incident: incident_pda(1, &program_id),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    // `sc` comanda el incidente 0, no el 1: no puede asignar equipo al incidente 1
    let ix = assign_equipment_ix(sc.pubkey(), operator.pubkey(), code, 1, program_id);
    assert!(
        !try_send_ix(&mut svm, &sc, ix),
        "assigning equipment to an incident commanded by someone else must fail"
    );
}

#[test]
fn test_operator_cannot_be_assigned_to_second_active_incident() {
    let (mut svm, admin, sc, operator, code, program_id) = setup();

    // Operador queda comprometido con el incidente 0
    send_ix(&mut svm, &sc, assign_equipment_ix(sc.pubkey(), operator.pubkey(), code, 0, program_id));

    // Segundo equipo, registrado por un OperationalBase nuevo
    let op_base2 = Keypair::new();
    svm.airdrop(&op_base2.pubkey(), 10_000_000_000).unwrap();
    send_ix(
        &mut svm,
        &admin,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterPersonnel {
                name: "OpsBase 2".to_string(),
                specialty: "Test".to_string(),
                role: Role::OperationalBase,
            }
            .data(),
            traz_log::accounts::RegisterPersonnel {
                signer: admin.pubkey(),
                global_state: global_state_pda(&program_id),
                signer_personnel: personnel_pda(&admin.pubkey(), &program_id),
                new_personnel: personnel_pda(&op_base2.pubkey(), &program_id),
                wallet: op_base2.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );
    let code2 = to_code("PUMP-BETA");
    send_ix(
        &mut svm,
        &op_base2,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterEquipment {
                code: code2,
                description: "Bomba beta".to_string(),
                nominal_consumption: 800,
            }
            .data(),
            traz_log::accounts::RegisterEquipment {
                signer: op_base2.pubkey(),
                global_state: global_state_pda(&program_id),
                signer_personnel: personnel_pda(&op_base2.pubkey(), &program_id),
                equipment: equipment_pda(&code2, &program_id),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    // Mismo Jefe de Escena abre un segundo incidente
    send_ix(
        &mut svm,
        &sc,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::OpenFireIncident {
                incident_id: 1,
                description: "Second Incident".to_string(),
                coordinates: "11.0,-85.0".to_string(),
                risk_level: 2,
            }
            .data(),
            traz_log::accounts::OpenFireIncident {
                signer: sc.pubkey(),
                global_state: global_state_pda(&program_id),
                signer_personnel: personnel_pda(&sc.pubkey(), &program_id),
                incident: incident_pda(1, &program_id),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    // El operador ya está comprometido con el incidente 0: no puede tomar equipo del incidente 1
    let ix = assign_equipment_ix(sc.pubkey(), operator.pubkey(), code2, 1, program_id);
    assert!(
        !try_send_ix(&mut svm, &sc, ix),
        "operator already committed to another active incident must not be re-assignable"
    );
}
