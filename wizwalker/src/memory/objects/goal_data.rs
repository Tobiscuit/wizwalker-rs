use crate::errors::{Result, WizWalkerError};
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};
use super::madlib_block::MadlibBlock;
use super::client_tag_list::ClientTagList;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalType {
    Unknown = 0,
    Bounty = 1,
    Bountycollect = 2,
    Scavenge = 3,
    Persona = 4,
    Waypoint = 5,
    Scavengefake = 6,
    Achieverank = 7,
    Usage = 8,
    Completequest = 9,
    Sociarank = 10,
    Sociacurrency = 11,
    Sociaminigame = 12,
    Sociagiveitem = 13,
    Sociagetitem = 14,
    Collectafterbounty = 15,
    EncounterWaypointForeach = 16,
}

pub struct GoalData {
    pub inner: DynamicMemoryObject,
}

impl GoalData {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn goal_status(&self) -> Result<bool> {
        self.inner.read_value_from_offset(0x74)
    }

    pub async fn goal_type(&self) -> Result<GoalType> {
        let val: i32 = self.inner.read_value_from_offset(0xB8)?;
        match val {
            0 => Ok(GoalType::Unknown),
            1 => Ok(GoalType::Bounty),
            2 => Ok(GoalType::Bountycollect),
            3 => Ok(GoalType::Scavenge),
            4 => Ok(GoalType::Persona),
            5 => Ok(GoalType::Waypoint),
            6 => Ok(GoalType::Scavengefake),
            7 => Ok(GoalType::Achieverank),
            8 => Ok(GoalType::Usage),
            9 => Ok(GoalType::Completequest),
            10 => Ok(GoalType::Sociarank),
            11 => Ok(GoalType::Sociacurrency),
            12 => Ok(GoalType::Sociaminigame),
            13 => Ok(GoalType::Sociagiveitem),
            14 => Ok(GoalType::Sociagetitem),
            15 => Ok(GoalType::Collectafterbounty),
            16 => Ok(GoalType::EncounterWaypointForeach),
            _ => Err(WizWalkerError::ReadingEnumFailed { enum_name: "GoalType".into(), value: val.to_string() }),
        }
    }

    pub async fn madlib_block(&self) -> Result<MadlibBlock> {
        let addr = self.inner.read_value_from_offset(0xC0)?;
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(MadlibBlock::new(inner))
    }

    pub async fn client_tag_list(&self) -> Result<Option<ClientTagList>> {
        let addr: u64 = self.inner.read_value_from_offset(0x110)?;
        if addr == 0 {
            return Ok(None);
        }
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(Some(ClientTagList::new(inner)))
    }

    pub async fn no_quest_helper(&self) -> Result<bool> {
        self.inner.read_value_from_offset(0x140)
    }

    pub async fn pet_only_quest(&self) -> Result<bool> {
        self.inner.read_value_from_offset(0x141)
    }

    pub async fn has_active_results(&self) -> Result<bool> {
        self.inner.read_value_from_offset(0x142)
    }

    pub async fn hide_goal_floaty_text(&self) -> Result<bool> {
        self.inner.read_value_from_offset(0x143)
    }
}
