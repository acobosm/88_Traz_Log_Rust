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
    traz_log::{PersonnelAccount, Role},
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn global_state_pda(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"global"], program_id).0
}

fn personnel_pda(wallet: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"personnel", wallet.as_ref()], program_id).0
}

fn setup() -> (LiteSVM, Keypair, Pubkey) {
    let program_id = traz_log::id();
    let admin = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/traz_log.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();

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

    (svm, admin, program_id)
}

fn register_personnel_ix(
    admin_pubkey: Pubkey,
    wallet: Pubkey,
    program_id: Pubkey,
    name: &str,
    specialty: &str,
    role: Role,
) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::RegisterPersonnel {
            name: name.to_string(),
            specialty: specialty.to_string(),
            role,
        }
        .data(),
        traz_log::accounts::RegisterPersonnel {
            signer: admin_pubkey,
            global_state: global_state_pda(&program_id),
            new_personnel: personnel_pda(&wallet, &program_id),
            wallet,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
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

fn read_personnel(svm: &LiteSVM, wallet: &Pubkey, program_id: &Pubkey) -> PersonnelAccount {
    let pda = personnel_pda(wallet, program_id);
    let account = svm.get_account(&pda).expect("PersonnelAccount PDA not found");
    let mut data: &[u8] = &account.data;
    PersonnelAccount::try_deserialize(&mut data).expect("deserialization failed")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_admin_registers_personnel_successfully() {
    let (mut svm, admin, program_id) = setup();
    let member = Keypair::new();
    svm.airdrop(&member.pubkey(), 1_000_000_000).unwrap();

    let ix = register_personnel_ix(
        admin.pubkey(),
        member.pubkey(),
        program_id,
        "Ana Garcia",
        "Logistics",
        Role::OperationalBase,
    );
    send_ix(&mut svm, &admin, ix);

    let pda = personnel_pda(&member.pubkey(), &program_id);
    assert!(
        svm.get_account(&pda).is_some(),
        "PersonnelAccount PDA must exist after registration"
    );
}

#[test]
fn test_personnel_account_fields_are_correct() {
    let (mut svm, admin, program_id) = setup();
    let member = Keypair::new();
    svm.airdrop(&member.pubkey(), 1_000_000_000).unwrap();

    let ix = register_personnel_ix(
        admin.pubkey(),
        member.pubkey(),
        program_id,
        "Carlos Ruiz",
        "Scene Command",
        Role::SceneCommander,
    );
    send_ix(&mut svm, &admin, ix);

    let p = read_personnel(&svm, &member.pubkey(), &program_id);
    assert_eq!(p.wallet, member.pubkey(), "wallet mismatch");
    assert_eq!(p.name, "Carlos Ruiz", "name mismatch");
    assert_eq!(p.specialty, "Scene Command", "specialty mismatch");
    assert!(p.is_active, "personnel must be active after registration");
    assert_eq!(p.role, Role::SceneCommander, "role mismatch");
}

#[test]
fn test_personnel_account_size_is_189_bytes() {
    let (mut svm, admin, program_id) = setup();
    let member = Keypair::new();
    svm.airdrop(&member.pubkey(), 1_000_000_000).unwrap();

    send_ix(
        &mut svm,
        &admin,
        register_personnel_ix(
            admin.pubkey(),
            member.pubkey(),
            program_id,
            "Test",
            "Test",
            Role::Operator,
        ),
    );

    let pda = personnel_pda(&member.pubkey(), &program_id);
    let len = svm.get_account(&pda).unwrap().data.len();
    // 8 discriminador + 32 wallet + (4+64) name + (4+64) specialty + 1 is_active + 1 role
    // + (1+8) current_incident + 1 active_assignments + 1 bump
    let expected = 8 + 32 + 68 + 68 + 1 + 1 + 9 + 1 + 1;
    assert_eq!(
        len, expected,
        "PersonnelAccount size mismatch: got {len}, expected {expected}"
    );
}

#[test]
fn test_non_admin_cannot_register_personnel() {
    let (mut svm, _admin, program_id) = setup();
    let attacker = Keypair::new();
    let victim = Keypair::new();
    svm.airdrop(&attacker.pubkey(), 10_000_000_000).unwrap();

    let ix = register_personnel_ix(
        attacker.pubkey(),
        victim.pubkey(),
        program_id,
        "Fake",
        "Intruder",
        Role::Admin,
    );
    assert!(
        !try_send_ix(&mut svm, &attacker, ix),
        "non-admin must not be able to register personnel"
    );
}

#[test]
fn test_paused_system_blocks_personnel_registration() {
    let (mut svm, admin, program_id) = setup();

    // Pausar el sistema
    send_ix(
        &mut svm,
        &admin,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::TogglePause {}.data(),
            traz_log::accounts::TogglePause {
                signer: admin.pubkey(),
                global_state: global_state_pda(&program_id),
            }
            .to_account_metas(None),
        ),
    );

    let member = Keypair::new();
    svm.airdrop(&member.pubkey(), 1_000_000_000).unwrap();
    let ix = register_personnel_ix(
        admin.pubkey(),
        member.pubkey(),
        program_id,
        "Blocked",
        "None",
        Role::Operator,
    );
    assert!(
        !try_send_ix(&mut svm, &admin, ix),
        "paused system must reject personnel registration"
    );
}
