use anchor_lang::prelude::*;

// ── Account structs ───────────────────────────────────────────────────────────

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub next_incident_id: u64,
    pub is_paused: bool,
    pub admin: Pubkey,  // única fuente de autoridad admin, no depende del role en PersonnelAccount
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct PersonnelAccount {
    pub wallet: Pubkey,
    #[max_len(64)]
    pub name: String,
    #[max_len(64)]
    pub specialty: String,
    pub is_active: bool,
    pub role: Role,
    pub current_incident: Option<u64>,  // Some(id) mientras tenga equipo asignado en ese incidente
    pub active_assignments: u8,         // equipos InUse a su cargo; current_incident se limpia al llegar a 0
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct EquipmentAccount {
    pub code: [u8; 32],
    #[max_len(128)]
    pub description: String,
    pub nominal_consumption: u64,  // ml/hour
    pub status: EquipmentStatus,               // lo mueve el programa (asignación/retorno)
    pub reported_condition: ReportedCondition, // lo reporta el operador en campo, no toca status
    pub custodian: Pubkey,
    pub incident_id: u64,
    pub use_start_time: i64,       // unix timestamp
    pub log_count: u64,            // número de LogEntry creados para este equipo
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct IncidentAccount {
    pub incident_id: u64,
    #[max_len(64)]
    pub description: String,
    #[max_len(128)]
    pub coordinates: String,
    pub risk_level: u8,            // 1–5
    pub is_active: bool,
    pub opened_at: i64,            // unix timestamp
    pub commander: Pubkey,  // único wallet que puede asignar equipo a este incidente
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct LogEntry {
    pub equipment_code: [u8; 32],
    #[max_len(256)]
    pub notes: String,
    pub condition: ReportedCondition,
    pub operator: Pubkey,
    pub timestamp: i64,
    pub entry_index: u64,
    pub bump: u8,
}

// ── Enums ─────────────────────────────────────────────────────────────────────

// quién puede hacer qué — se verifica con constraints en cada instrucción, sin AccessControl
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum Role {
    Admin,
    OperationalBase,
    SceneCommander,
    Operator,
}

// ciclo de vida del equipo, lo mueve el programa (assign/initiate_return)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum EquipmentStatus {
    Available,
    InUse,
    InRepair,
    Lost,
    Returning,
}

// condición que reporta el operador en campo vía log_milestone, es diferente de los estados de EquipmentStatus
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum ReportedCondition {
    Operational,
    MinorDamage,
    CriticalDamage,
    Lost,
}
