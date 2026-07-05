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
    traz_log::{EquipmentAccount, EquipmentStatus, ReportedCondition, Role},
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

/// Convierte un string en un código de equipo [u8; 32] con padding de ceros.
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

/// Inicializa el sistema y registra op_base como OperationalBase.
fn setup() -> (LiteSVM, Keypair, Keypair, Pubkey) {
    let program_id = traz_log::id();
    let admin = Keypair::new();
    let op_base = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/traz_log.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&op_base.pubkey(), 10_000_000_000).unwrap();

    // 1. Inicializar GlobalState
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

    // 2. Registrar op_base con rol OperationalBase
    send_ix(
        &mut svm,
        &admin,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterPersonnel {
                name: "Ops Base Member".to_string(),
                specialty: "Inventory".to_string(),
                role: Role::OperationalBase,
            }
            .data(),
            traz_log::accounts::RegisterPersonnel {
                signer: admin.pubkey(),
                global_state: global_state_pda(&program_id),
                new_personnel: personnel_pda(&op_base.pubkey(), &program_id),
                wallet: op_base.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    (svm, admin, op_base, program_id)
}

fn register_equipment_ix(
    signer_pubkey: Pubkey,
    code: [u8; 32],
    program_id: Pubkey,
    description: &str,
    nominal_consumption: u64,
) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::RegisterEquipment {
            code,
            description: description.to_string(),
            nominal_consumption,
        }
        .data(),
        traz_log::accounts::RegisterEquipment {
            signer: signer_pubkey,
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&signer_pubkey, &program_id),
            equipment: equipment_pda(&code, &program_id),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn read_equipment(svm: &LiteSVM, code: &[u8; 32], program_id: &Pubkey) -> EquipmentAccount {
    let pda = equipment_pda(code, program_id);
    let account = svm.get_account(&pda).expect("EquipmentAccount PDA not found");
    let mut data: &[u8] = &account.data;
    EquipmentAccount::try_deserialize(&mut data).expect("deserialization failed")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_operational_base_registers_equipment_successfully() {
    let (mut svm, _admin, op_base, program_id) = setup();
    let code = to_code("MOTO-001");

    let ix = register_equipment_ix(op_base.pubkey(), code, program_id, "Motosierra 001", 500);
    send_ix(&mut svm, &op_base, ix);

    let pda = equipment_pda(&code, &program_id);
    assert!(
        svm.get_account(&pda).is_some(),
        "EquipmentAccount PDA must exist after registration"
    );
}

#[test]
fn test_equipment_account_fields_are_correct() {
    let (mut svm, _admin, op_base, program_id) = setup();
    let code = to_code("PUMP-007");

    send_ix(
        &mut svm,
        &op_base,
        register_equipment_ix(op_base.pubkey(), code, program_id, "Bomba de agua 007", 1200),
    );

    let e = read_equipment(&svm, &code, &program_id);
    assert_eq!(e.code, code, "code mismatch");
    assert_eq!(e.description, "Bomba de agua 007", "description mismatch");
    assert_eq!(e.nominal_consumption, 1200, "nominal_consumption mismatch");
    assert_eq!(e.status, EquipmentStatus::Available, "status must be Available after registration");
    assert_eq!(
        e.reported_condition,
        ReportedCondition::Operational,
        "reported_condition must be Operational after registration"
    );
    assert_eq!(e.custodian, Pubkey::default(), "custodian must be default when Available");
    assert_eq!(e.incident_id, 0, "incident_id must be 0 when Available");
    assert_eq!(e.use_start_time, 0, "use_start_time must be 0 when Available");
}

#[test]
fn test_equipment_account_size_is_231_bytes() {
    let (mut svm, _admin, op_base, program_id) = setup();
    let code = to_code("SIZE-TEST");

    send_ix(
        &mut svm,
        &op_base,
        register_equipment_ix(op_base.pubkey(), code, program_id, "Size check", 100),
    );

    let pda = equipment_pda(&code, &program_id);
    let len = svm.get_account(&pda).unwrap().data.len();
    // 8 discriminador + 32 code + (4+128) description + 8 nominal_consumption
    // + 1 status + 1 reported_condition + 32 custodian + 8 incident_id + 8 use_start_time + 1 bump
    let expected = 8 + 32 + 132 + 8 + 1 + 1 + 32 + 8 + 8 + 1;
    assert_eq!(
        len, expected,
        "EquipmentAccount size mismatch: got {len}, expected {expected}"
    );
}

#[test]
fn test_wrong_role_cannot_register_equipment() {
    let (mut svm, admin, _op_base, program_id) = setup();

    // Registrar una persona con rol Operator
    let operator = Keypair::new();
    svm.airdrop(&operator.pubkey(), 10_000_000_000).unwrap();
    send_ix(
        &mut svm,
        &admin,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterPersonnel {
                name: "Field Op".to_string(),
                specialty: "Ground".to_string(),
                role: Role::Operator,
            }
            .data(),
            traz_log::accounts::RegisterPersonnel {
                signer: admin.pubkey(),
                global_state: global_state_pda(&program_id),
                new_personnel: personnel_pda(&operator.pubkey(), &program_id),
                wallet: operator.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    // Intentar registrar equipo con rol Operator → debe fallar
    let code = to_code("FORBIDDEN-EQ");
    let ix = register_equipment_ix(operator.pubkey(), code, program_id, "Not allowed", 0);
    assert!(
        !try_send_ix(&mut svm, &operator, ix),
        "Operator role must not be able to register equipment"
    );
}

#[test]
fn test_duplicate_equipment_code_fails() {
    let (mut svm, _admin, op_base, program_id) = setup();
    let code = to_code("DUPLICATE-EQ");

    // Primer registro: debe funcionar
    send_ix(
        &mut svm,
        &op_base,
        register_equipment_ix(op_base.pubkey(), code, program_id, "Original", 300),
    );

    // Segundo registro con el mismo code: debe fallar
    svm.expire_blockhash();
    let ix2 = register_equipment_ix(op_base.pubkey(), code, program_id, "Duplicate", 300);
    assert!(
        !try_send_ix(&mut svm, &op_base, ix2),
        "registering the same equipment code twice must fail"
    );
}
