use crate::errors::Result;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HangingSpellEffect {
    InvalidSpellEffect = 0,
    AbsorbDamage = 35,
    AbsorbHeal = 36,
    AddCombatTriggerList = 85,
    Afterlife = 80,
    BacklashDamage = 87,
    BounceAll = 34,
    BounceBack = 33,
    BounceNext = 31,
    BouncePrevious = 32,
    CloakedCharm = 40,
    CloakedWard = 41,
    CloakedWardNoRemove = 84,
    Confusion = 39,
    ConfusionBlock = 107,
    CritBlock = 45,
    CritBoost = 44,
    CritBoostSchoolSpecific = 95,
    Damage = 1,
    DamageNoCrit = 2,
    DamageOverTime = 73,
    DamagePerTotalPipPower = 82,
    Dampen = 66,
    DeferredDamage = 81,
    DelayCast = 47,
    DetonateOverTime = 7,
    Dispel = 38,
    DivideDamage = 103,
    Heal = 3,
    HealOverTime = 74,
    InstantKill = 79,
    Intercept = 89,
    MaxHealthDamage = 110,
    MaximumIncomingDamage = 23,
    MindControl = 68,
    ModifyAccuracy = 37,
    ModifyBacklash = 88,
    ModifyCardAccuracy = 51,
    ModifyCardArmorPiercing = 54,
    ModifyCardCloak = 48,
    ModifyCardDamage = 49,
    ModifyCardMutation = 52,
    ModifyCardRank = 53,
    ModifyHate = 72,
    ModifyIncomingArmorPiercing = 26,
    ModifyIncomingDamage = 22,
    ModifyIncomingDamageFlat = 117,
    ModifyIncomingDamageType = 25,
    ModifyIncomingHeal = 24,
    ModifyIncomingHealFlat = 116,
    ModifyIncomingHealOverTime = 136,
    ModifyOutgoingArmorPiercing = 30,
    ModifyOutgoingDamage = 27,
    ModifyOutgoingDamageFlat = 119,
    ModifyOutgoingDamageType = 29,
    ModifyOutgoingHeal = 28,
    ModifyOutgoingHealFlat = 118,
    ModifyPipRoundRate = 108,
    ModifyPips = 69,
    ModifyPowerPipChance = 75,
    ModifyPowerPips = 70,
    ModifyRank = 76,
    ModifyShadowCreatureLevel = 92,
    ModifyShadowPips = 71,
    PipConversion = 43,
    Polymorph = 46,
    PowerPipConversion = 98,
    ProtectBeneficial = 101,
    ProtectCardBeneficial = 99,
    ProtectCardHarmful = 100,
    ProtectHarmful = 102,
    PushCharm = 8,
    PushOverTime = 12,
    PushWard = 10,
    ReduceOverTime = 6,
    RemoveAura = 17,
    RemoveCharm = 14,
    RemoveCombatTriggerList = 86,
    RemoveOverTime = 16,
    RemoveWard = 15,
    Reshuffle = 67,
    RevealCloak = 78,
    SelectShadowCreatureAttackTarget = 93,
    ShadowCreature = 91,
    ShadowDecrementTurn = 94,
    ShadowSelf = 90,
    SpawnCreature = 96,
    StealCharm = 9,
    StealHealth = 5,
    StealOverTime = 13,
    StealWard = 11,
    Stun = 65,
    StunBlock = 77,
    StunResist = 42,
    SummonCreature = 63,
    SwapCharm = 19,
    SwapOverTime = 21,
    SwapWard = 20,
    TeleportPlayer = 64,
    UnPolymorph = 97,
    MaxHealthHeal = 127,
    HealByWard = 128,
    Taunt = 129,
    Pacify = 130,
}

pub fn school_id_to_names() -> std::collections::HashMap<&'static str, i32> {
    let mut map = std::collections::HashMap::new();
    map.insert("Fire", 2343174);
    map.insert("Ice", 72777);
    map.insert("Storm", 83375795);
    map.insert("Myth", 2448141);
    map.insert("Life", 2330892);
    map.insert("Death", 78318724);
    map.insert("Balance", 1027491821);
    map.insert("Star", 2625203);
    map.insert("Sun", 78483);
    map.insert("Moon", 2504141);
    map.insert("Gardening", 663550619);
    map.insert("Shadow", 1429009101);
    map.insert("Fishing", 1488274711);
    map.insert("Cantrips", 1760873841);
    map.insert("CastleMagic", 806477568);
    map.insert("WhirlyBurly", 931528087);
    map
}

pub fn school_to_str() -> std::collections::HashMap<i32, &'static str> {
    let mut map = std::collections::HashMap::new();
    for (k, v) in school_id_to_names() {
        map.insert(v, k);
    }
    map
}

pub struct Requirement {
    pub inner: DynamicMemoryObject,
}

impl Requirement {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub fn apply_not(&self) -> Result<bool> {
        self.inner.read_value_from_offset(72)
    }
}

pub struct ReqCombatHealth {
    pub requirement: Requirement,
}

impl ReqCombatHealth {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: Requirement::new(inner) }
    }

    pub fn min_percent(&self) -> Result<f32> {
        self.requirement.inner.read_value_from_offset(88)
    }

    pub fn max_percent(&self) -> Result<f32> {
        self.requirement.inner.read_value_from_offset(92)
    }
}

pub struct ReqHangingOverTime {
    pub requirement: Requirement,
}

impl ReqHangingOverTime {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: Requirement::new(inner) }
    }

    pub fn min_count(&self) -> Result<i32> {
        self.requirement.inner.read_value_from_offset(92)
    }

    pub fn max_count(&self) -> Result<i32> {
        self.requirement.inner.read_value_from_offset(96)
    }
}

pub struct ReqIsSchool {
    pub requirement: Requirement,
}

impl ReqIsSchool {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: Requirement::new(inner) }
    }

    // pub fn magic_school_name(&self) -> Result<String>
}

pub struct ReqHangingWard {
    pub requirement: Requirement,
}

impl ReqHangingWard {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: Requirement::new(inner) }
    }

    pub fn min_count(&self) -> Result<i32> {
        self.requirement.inner.read_value_from_offset(92)
    }

    pub fn max_count(&self) -> Result<i32> {
        self.requirement.inner.read_value_from_offset(96)
    }
}

pub struct ReqHangingEffectType {
    pub requirement: Requirement,
}

impl ReqHangingEffectType {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: Requirement::new(inner) }
    }

    pub fn param_low(&self) -> Result<i32> {
        self.requirement.inner.read_value_from_offset(92)
    }

    pub fn param_high(&self) -> Result<i32> {
        self.requirement.inner.read_value_from_offset(96)
    }

    pub fn min_count(&self) -> Result<i32> {
        self.requirement.inner.read_value_from_offset(100)
    }

    pub fn max_count(&self) -> Result<i32> {
        self.requirement.inner.read_value_from_offset(104)
    }
}

pub struct ReqPvPCombat {
    pub requirement: Requirement,
}

impl ReqPvPCombat {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: Requirement::new(inner) }
    }
}

pub struct ReqShadowPipCount {
    pub requirement: Requirement,
}

impl ReqShadowPipCount {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: Requirement::new(inner) }
    }

    pub fn min_pips(&self) -> Result<i32> {
        self.requirement.inner.read_value_from_offset(88)
    }

    pub fn max_pips(&self) -> Result<i32> {
        self.requirement.inner.read_value_from_offset(92)
    }
}

pub struct ReqPipCount {
    pub requirement: Requirement,
}

impl ReqPipCount {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: Requirement::new(inner) }
    }

    pub fn min_pips(&self) -> Result<i32> {
        self.requirement.inner.read_value_from_offset(88)
    }

    pub fn max_pips(&self) -> Result<i32> {
        self.requirement.inner.read_value_from_offset(92)
    }
}

pub struct ReqMinion {
    pub requirement: Requirement,
}

impl ReqMinion {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: Requirement::new(inner) }
    }
}

pub struct ReqCombatStatus {
    pub requirement: Requirement,
}

impl ReqCombatStatus {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: Requirement::new(inner) }
    }
}

pub struct RequirementList {
    pub requirement: Requirement,
}

impl RequirementList {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: Requirement::new(inner) }
    }
}

pub struct ConditionalSpellEffectRequirement {
    pub requirement: Requirement,
}

impl ConditionalSpellEffectRequirement {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: Requirement::new(inner) }
    }
}

pub struct ReqHangingCharm {
    pub requirement: ConditionalSpellEffectRequirement,
}

impl ReqHangingCharm {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { requirement: ConditionalSpellEffectRequirement::new(inner) }
    }

    pub fn min_count(&self) -> Result<i32> {
        self.requirement.requirement.inner.read_value_from_offset(92)
    }

    pub fn max_count(&self) -> Result<i32> {
        self.requirement.requirement.inner.read_value_from_offset(96)
    }
}
