use anchor_lang::prelude::*;
use crate::state::Role;

#[event]
pub struct PersonnelRegistered {
    pub wallet: Pubkey,
    pub role: Role,
}

#[event]
pub struct EquipmentRegistered {
    pub code: [u8; 32],
}

#[event]
pub struct IncidentOpened {
    pub incident_id: u64,
    pub commander: Pubkey,
    pub risk_level: u8,
}

#[event]
pub struct EquipmentAssigned {
    pub incident_id: u64,
    pub equipment_code: [u8; 32],
    pub operator: Pubkey,
}

#[event]
pub struct IncidentClosed {
    pub incident_id: u64,
}
