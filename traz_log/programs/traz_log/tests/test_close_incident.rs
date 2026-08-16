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
    traz_log::{IncidentAccount, Role},
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn global_state_pda(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"global"], program_id).0
}

fn personnel_pda(wallet: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"personnel", wallet.as_ref()], program_id).0
}

fn incident_pda(incident_id: u64, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"incident", &incident_id.to_le_bytes()], program_id).0
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

/// Setup: GlobalState + SceneCommander + Operator registrado + incidente 0 abierto.
fn setup() -> (LiteSVM, Keypair, Keypair, Keypair, Pubkey) {
    let program_id = traz_log::id();
    let admin = Keypair::new();
    let scene_commander = Keypair::new();
    let operator = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/traz_log.so");
    svm.add_program(program_id, bytes).unwrap();
    for kp in [&admin, &scene_commander, &operator] {
        svm.airdrop(&kp.pubkey(), 10_000_000_000).unwrap();
    }

    send_ix(&mut svm, &admin, Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::Initialize {}.data(),
        traz_log::accounts::Initialize {
            admin: admin.pubkey(),
            global_state: global_state_pda(&program_id),
            system_program: system_program::ID,
        }.to_account_metas(None),
    ));

    for (wallet, role, name) in [
        (scene_commander.pubkey(), Role::SceneCommander, "Commander"),
        (operator.pubkey(), Role::Operator, "Operator"),
    ] {
        send_ix(&mut svm, &admin, Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterPersonnel {
                name: name.to_string(), specialty: "Test".to_string(), role,
            }.data(),
            traz_log::accounts::RegisterPersonnel {
                signer: admin.pubkey(),
                global_state: global_state_pda(&program_id),
                signer_personnel: personnel_pda(&admin.pubkey(), &program_id),
                new_personnel: personnel_pda(&wallet, &program_id),
                wallet,
                system_program: system_program::ID,
            }.to_account_metas(None),
        ));
    }

    send_ix(&mut svm, &scene_commander, Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::OpenFireIncident {
            incident_id: 0, description: "Test Incident".to_string(), coordinates: "9.934,-84.082".to_string(), risk_level: 5,
        }.data(),
        traz_log::accounts::OpenFireIncident {
            signer: scene_commander.pubkey(),
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&scene_commander.pubkey(), &program_id),
            incident: incident_pda(0, &program_id),
            system_program: system_program::ID,
        }.to_account_metas(None),
    ));

    (svm, admin, scene_commander, operator, program_id)
}

fn close_incident_ix(sc_pubkey: Pubkey, incident_id: u64, program_id: Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::CloseIncident { incident_id }.data(),
        traz_log::accounts::CloseIncident {
            signer: sc_pubkey,
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&sc_pubkey, &program_id),
            incident: incident_pda(incident_id, &program_id),
        }.to_account_metas(None),
    )
}

fn read_incident(svm: &LiteSVM, incident_id: u64, program_id: &Pubkey) -> IncidentAccount {
    let account = svm.get_account(&incident_pda(incident_id, program_id)).unwrap();
    let mut data: &[u8] = &account.data;
    IncidentAccount::try_deserialize(&mut data).unwrap()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_scene_commander_closes_incident_successfully() {
    let (mut svm, _admin, sc, _operator, program_id) = setup();

    send_ix(&mut svm, &sc, close_incident_ix(sc.pubkey(), 0, program_id));

    assert!(!read_incident(&svm, 0, &program_id).is_active);
}

#[test]
fn test_incident_is_inactive_after_close() {
    let (mut svm, _admin, sc, _operator, program_id) = setup();

    assert!(read_incident(&svm, 0, &program_id).is_active, "incident must start active");

    send_ix(&mut svm, &sc, close_incident_ix(sc.pubkey(), 0, program_id));

    assert!(
        !read_incident(&svm, 0, &program_id).is_active,
        "incident must be inactive after close"
    );
}

#[test]
fn test_cannot_close_already_closed_incident() {
    let (mut svm, _admin, sc, _operator, program_id) = setup();

    send_ix(&mut svm, &sc, close_incident_ix(sc.pubkey(), 0, program_id));

    svm.expire_blockhash();
    assert!(
        !try_send_ix(&mut svm, &sc, close_incident_ix(sc.pubkey(), 0, program_id)),
        "closing an already closed incident must fail"
    );
}

#[test]
fn test_wrong_role_cannot_close_incident() {
    let (mut svm, _admin, _sc, operator, program_id) = setup();

    assert!(
        !try_send_ix(&mut svm, &operator, close_incident_ix(operator.pubkey(), 0, program_id)),
        "Operator role must not be able to close an incident"
    );
}
