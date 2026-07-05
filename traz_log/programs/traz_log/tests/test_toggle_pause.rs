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
    traz_log::GlobalState,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn global_state_pda(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"global"], program_id).0
}

/// Levanta LiteSVM con el programa cargado y el GlobalState inicializado.
fn setup() -> (LiteSVM, Keypair, Pubkey) {
    let program_id = traz_log::id();
    let admin = Keypair::new();
    let mut svm = LiteSVM::new();

    let bytes = include_bytes!("../../../target/deploy/traz_log.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&admin.pubkey(), 2_000_000_000).unwrap();

    let pda = global_state_pda(&program_id);
    let ix = Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::Initialize {}.data(),
        traz_log::accounts::Initialize {
            admin: admin.pubkey(),
            global_state: pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&admin.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&admin]).unwrap();
    svm.send_transaction(tx).expect("setup: initialize failed");

    (svm, admin, program_id)
}

fn toggle_pause_ix(signer_pubkey: Pubkey, program_id: Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::TogglePause {}.data(),
        traz_log::accounts::TogglePause {
            signer: signer_pubkey,
            global_state: global_state_pda(&program_id),
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

fn read_global_state(svm: &LiteSVM, program_id: &Pubkey) -> GlobalState {
    let pda = global_state_pda(program_id);
    let account = svm.get_account(&pda).expect("GlobalState PDA not found");
    let mut data: &[u8] = &account.data;
    GlobalState::try_deserialize(&mut data).expect("deserialization failed")
}

// ── Tests: Arquitectura de cuenta ───────────────────────────────────────────

#[test]
fn test_global_state_account_size_is_50_bytes() {
    let (svm, _, program_id) = setup();
    let pda = global_state_pda(&program_id);
    let account = svm.get_account(&pda).expect("GlobalState not found");

    // 8 discriminador + 8 next_incident_id + 1 is_paused + 32 admin + 1 bump
    let expected = 8 + 8 + 1 + 32 + 1;
    assert_eq!(
        account.data.len(),
        expected,
        "GlobalState size mismatch: got {}, expected {}",
        account.data.len(),
        expected
    );
}

#[test]
fn test_global_state_fields_after_initialize() {
    let (svm, admin, program_id) = setup();
    let state = read_global_state(&svm, &program_id);

    assert_eq!(state.next_incident_id, 0, "next_incident_id debe empezar en 0");
    assert!(!state.is_paused, "sistema no debe estar pausado al inicio");
    assert_eq!(state.admin, admin.pubkey(), "admin debe ser quien firmó initialize");
}

// ── Tests: toggle_pause ──────────────────────────────────────────────────────

#[test]
fn test_admin_can_pause_system() {
    let (mut svm, admin, program_id) = setup();

    let ix = toggle_pause_ix(admin.pubkey(), program_id);
    send_ix(&mut svm, &admin, ix);

    let state = read_global_state(&svm, &program_id);
    assert!(state.is_paused, "sistema debe estar pausado después de toggle");
}

#[test]
fn test_admin_can_unpause_system() {
    let (mut svm, admin, program_id) = setup();

    // Primer toggle: pausa
    send_ix(&mut svm, &admin, toggle_pause_ix(admin.pubkey(), program_id));
    // Rota el blockhash para que la segunda tx tenga firma distinta
    svm.expire_blockhash();
    // Segundo toggle: despausa
    send_ix(&mut svm, &admin, toggle_pause_ix(admin.pubkey(), program_id));

    let state = read_global_state(&svm, &program_id);
    assert!(!state.is_paused, "sistema debe estar activo después de doble toggle");
}

#[test]
fn test_non_admin_cannot_toggle_pause() {
    let (mut svm, _admin, program_id) = setup();

    let attacker = Keypair::new();
    svm.airdrop(&attacker.pubkey(), 1_000_000_000).unwrap();

    let ix = toggle_pause_ix(attacker.pubkey(), program_id);
    let succeeded = try_send_ix(&mut svm, &attacker, ix);

    assert!(!succeeded, "wallet no-admin no debe poder pausar el sistema");
}
