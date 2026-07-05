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
    traz_log::{GlobalState, IncidentAccount, Role},
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

/// Inicializa el sistema y registra scene_commander con rol SceneCommander.
fn setup() -> (LiteSVM, Keypair, Keypair, Pubkey) {
    let program_id = traz_log::id();
    let admin = Keypair::new();
    let scene_commander = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/traz_log.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&scene_commander.pubkey(), 10_000_000_000).unwrap();

    // Inicializar GlobalState
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

    // Registrar scene_commander
    send_ix(
        &mut svm,
        &admin,
        Instruction::new_with_bytes(
            program_id,
            &traz_log::instruction::RegisterPersonnel {
                name: "Commander Torres".to_string(),
                specialty: "Scene Command".to_string(),
                role: Role::SceneCommander,
            }
            .data(),
            traz_log::accounts::RegisterPersonnel {
                signer: admin.pubkey(),
                global_state: global_state_pda(&program_id),
                new_personnel: personnel_pda(&scene_commander.pubkey(), &program_id),
                wallet: scene_commander.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    (svm, admin, scene_commander, program_id)
}

fn open_incident_ix(
    signer_pubkey: Pubkey,
    incident_id: u64,
    program_id: Pubkey,
    coordinates: &str,
    risk_level: u8,
) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &traz_log::instruction::OpenFireIncident {
            incident_id,
            coordinates: coordinates.to_string(),
            risk_level,
        }
        .data(),
        traz_log::accounts::OpenFireIncident {
            signer: signer_pubkey,
            global_state: global_state_pda(&program_id),
            signer_personnel: personnel_pda(&signer_pubkey, &program_id),
            incident: incident_pda(incident_id, &program_id),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn read_global_state(svm: &LiteSVM, program_id: &Pubkey) -> GlobalState {
    let account = svm.get_account(&global_state_pda(program_id)).unwrap();
    let mut data: &[u8] = &account.data;
    GlobalState::try_deserialize(&mut data).unwrap()
}

fn read_incident(svm: &LiteSVM, incident_id: u64, program_id: &Pubkey) -> IncidentAccount {
    let account = svm.get_account(&incident_pda(incident_id, program_id)).unwrap();
    let mut data: &[u8] = &account.data;
    IncidentAccount::try_deserialize(&mut data).unwrap()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_scene_commander_opens_incident_successfully() {
    let (mut svm, _admin, sc, program_id) = setup();

    send_ix(&mut svm, &sc, open_incident_ix(sc.pubkey(), 0, program_id, "10.123,-84.567", 3));

    let pda = incident_pda(0, &program_id);
    assert!(svm.get_account(&pda).is_some(), "IncidentAccount PDA must exist after open");
}

#[test]
fn test_incident_account_fields_are_correct() {
    let (mut svm, _admin, sc, program_id) = setup();

    send_ix(&mut svm, &sc, open_incident_ix(sc.pubkey(), 0, program_id, "9.934,-84.082", 4));

    let inc = read_incident(&svm, 0, &program_id);
    assert_eq!(inc.incident_id, 0);
    assert_eq!(inc.coordinates, "9.934,-84.082");
    assert_eq!(inc.risk_level, 4);
    assert!(inc.is_active, "incident must be active after open");
    assert_eq!(inc.commander, sc.pubkey());
}

#[test]
fn test_incident_account_size_is_191_bytes() {
    let (mut svm, _admin, sc, program_id) = setup();

    send_ix(&mut svm, &sc, open_incident_ix(sc.pubkey(), 0, program_id, "0,0", 1));

    let pda = incident_pda(0, &program_id);
    let len = svm.get_account(&pda).unwrap().data.len();
    // 8 discriminador + 8 incident_id + (4+128) coordinates + 1 risk_level
    // + 1 is_active + 8 opened_at + 32 commander + 1 bump
    let expected = 8 + 8 + 132 + 1 + 1 + 8 + 32 + 1;
    assert_eq!(len, expected, "IncidentAccount size mismatch: got {len}, expected {expected}");
}

#[test]
fn test_next_incident_id_increments_after_open() {
    let (mut svm, _admin, sc, program_id) = setup();

    let before = read_global_state(&svm, &program_id).next_incident_id;
    assert_eq!(before, 0);

    send_ix(&mut svm, &sc, open_incident_ix(sc.pubkey(), 0, program_id, "0,0", 2));

    let after = read_global_state(&svm, &program_id).next_incident_id;
    assert_eq!(after, 1, "next_incident_id must increment after opening an incident");
}

#[test]
fn test_invalid_risk_level_fails() {
    let (mut svm, _admin, sc, program_id) = setup();

    // risk_level = 6 está fuera del rango válido (1–5)
    let ix = open_incident_ix(sc.pubkey(), 0, program_id, "0,0", 6);
    assert!(!try_send_ix(&mut svm, &sc, ix), "risk_level=6 must be rejected");
}

#[test]
fn test_wrong_incident_id_fails() {
    let (mut svm, _admin, sc, program_id) = setup();

    // El contador es 0 pero pasamos incident_id = 1
    let ix = open_incident_ix(sc.pubkey(), 1, program_id, "0,0", 2);
    assert!(!try_send_ix(&mut svm, &sc, ix), "mismatched incident_id must be rejected");
}
