use {
    anchor_lang::{
        solana_program::{instruction::Instruction, pubkey::Pubkey, system_program},
        InstructionData,
        ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

fn global_state_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"global"], program_id)
}

fn send_initialize(svm: &mut LiteSVM, admin: &Keypair, program_id: Pubkey) {
    let (pda, _) = global_state_pda(&program_id);
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
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[admin]).unwrap();
    svm.send_transaction(tx).expect("initialize failed");
}

#[test]
fn test_initialize_creates_global_state() {
    let program_id = traz_log::id();
    let admin = Keypair::new();
    let mut svm = LiteSVM::new();

    let bytes = include_bytes!("../../../target/deploy/traz_log.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&admin.pubkey(), 1_000_000_000).unwrap();

    let (global_state_pda, _) = global_state_pda(&program_id);

    let ix = Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::Initialize {}.data(),
        traz_log::accounts::Initialize {
            admin: admin.pubkey(),
            global_state: global_state_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&admin.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[admin]).unwrap();

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "initialize failed: {:?}", result.err());
}

#[test]
fn test_global_state_not_paused_after_init() {
    let program_id = traz_log::id();
    let admin = Keypair::new();
    let mut svm = LiteSVM::new();

    let bytes = include_bytes!("../../../target/deploy/traz_log.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&admin.pubkey(), 1_000_000_000).unwrap();

    send_initialize(&mut svm, &admin, program_id);

    let (pda, _) = global_state_pda(&program_id);
    let data = svm.get_account(&pda).expect("GlobalState PDA must exist").data;

    // Layout: discriminator[8] + next_incident_id[8] + is_paused[1]
    assert_eq!(data[16], 0, "system should not be paused after initialize");
}
