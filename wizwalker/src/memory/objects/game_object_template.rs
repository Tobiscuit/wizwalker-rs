use crate::errors::{Result, WizWalkerError};
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject, MemoryObjectExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ObjectType {
    Undefined = 0,
    Player = 1,
    Npc = 2,
    Prop = 3,
    Object = 4,
    House = 5,
    Key = 6,
    OldKey = 7,
    Deed = 8,
    Mail = 9,
    Recipe = 17,
    EquipHead = 10,
    EquipChest = 11,
    EquipLegs = 12,
    EquipHands = 13,
    EquipFinger = 14,
    EquipFeet = 15,
    EquipEar = 16,
    BuildingBlock = 18,
    BuildingBlockSolid = 19,
    Golf = 20,
    Door = 21,
    Pet = 22,
    Fabric = 23,
    Window = 24,
    Roof = 25,
    Horse = 26,
    Structure = 27,
    HousingTexture = 28,
    Plant = 29,
}

impl TryFrom<i32> for ObjectType {
    type Error = WizWalkerError;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(ObjectType::Undefined),
            1 => Ok(ObjectType::Player),
            2 => Ok(ObjectType::Npc),
            3 => Ok(ObjectType::Prop),
            4 => Ok(ObjectType::Object),
            5 => Ok(ObjectType::House),
            6 => Ok(ObjectType::Key),
            7 => Ok(ObjectType::OldKey),
            8 => Ok(ObjectType::Deed),
            9 => Ok(ObjectType::Mail),
            10 => Ok(ObjectType::EquipHead),
            11 => Ok(ObjectType::EquipChest),
            12 => Ok(ObjectType::EquipLegs),
            13 => Ok(ObjectType::EquipHands),
            14 => Ok(ObjectType::EquipFinger),
            15 => Ok(ObjectType::EquipFeet),
            16 => Ok(ObjectType::EquipEar),
            17 => Ok(ObjectType::Recipe),
            18 => Ok(ObjectType::BuildingBlock),
            19 => Ok(ObjectType::BuildingBlockSolid),
            20 => Ok(ObjectType::Golf),
            21 => Ok(ObjectType::Door),
            22 => Ok(ObjectType::Pet),
            23 => Ok(ObjectType::Fabric),
            24 => Ok(ObjectType::Window),
            25 => Ok(ObjectType::Roof),
            26 => Ok(ObjectType::Horse),
            27 => Ok(ObjectType::Structure),
            28 => Ok(ObjectType::HousingTexture),
            29 => Ok(ObjectType::Plant),
            _ => Err(WizWalkerError::Other(format!("Invalid ObjectType value: {}", value))),
        }
    }
}

pub trait WizGameObjectTemplate: MemoryObject {
    fn object_name(&self) -> Result<String> {
        self.read_string_from_offset(96)
    }

    fn write_object_name(&self, object_name: &str) -> Result<()> {
        self.write_string_to_offset(96, object_name)
    }

    fn template_id(&self) -> Result<u32> {
        self.read_value_from_offset(128)
    }

    fn write_template_id(&self, template_id: u32) -> Result<()> {
        self.write_value_to_offset(128, &template_id)
    }

    fn visual_id(&self) -> Result<u32> {
        self.read_value_from_offset(132)
    }

    fn write_visual_id(&self, visual_id: u32) -> Result<()> {
        self.write_value_to_offset(132, &visual_id)
    }

    fn adjective_list(&self) -> Result<String> {
        self.read_string_from_offset(248)
    }

    fn write_adjective_list(&self, adjective_list: &str) -> Result<()> {
        self.write_string_to_offset(248, adjective_list)
    }

    fn exempt_from_aoi(&self) -> Result<bool> {
        self.read_value_from_offset(240)
    }

    fn write_exempt_from_aoi(&self, exempt_from_aoi: bool) -> Result<()> {
        self.write_value_to_offset(240, &exempt_from_aoi)
    }

    fn display_name(&self) -> Result<String> {
        self.read_string_from_offset(168)
    }

    fn write_display_name(&self, display_name: &str) -> Result<()> {
        self.write_string_to_offset(168, display_name)
    }

    fn description(&self) -> Result<String> {
        self.read_string_from_offset(136)
    }

    fn write_description(&self, description: &str) -> Result<()> {
        self.write_string_to_offset(136, description)
    }

    fn object_type(&self) -> Result<ObjectType> {
        self.read_enum(200)
    }

    fn icon(&self) -> Result<String> {
        self.read_string_from_offset(208)
    }

    fn write_icon(&self, icon: &str) -> Result<()> {
        self.write_string_to_offset(208, icon)
    }

    fn object_property_hashset(&self) -> Result<std::collections::HashSet<u32>> {
        self.read_hashset_basic(264)
    }

    fn loot_table(&self) -> Result<String> {
        self.read_string_from_offset(280)
    }

    fn write_loot_table(&self, loot_table: &str) -> Result<()> {
        self.write_string_to_offset(280, loot_table)
    }

    fn death_particles(&self) -> Result<String> {
        self.read_string_from_offset(296)
    }

    fn write_death_particles(&self, death_particles: &str) -> Result<()> {
        self.write_string_to_offset(296, death_particles)
    }

    fn death_sound(&self) -> Result<String> {
        self.read_string_from_offset(328)
    }

    fn write_death_sound(&self, death_sound: &str) -> Result<()> {
        self.write_string_to_offset(328, death_sound)
    }

    fn hit_sound(&self) -> Result<String> {
        self.read_string_from_offset(360)
    }

    fn write_hit_sound(&self, hit_sound: &str) -> Result<()> {
        self.write_string_to_offset(360, hit_sound)
    }

    fn cast_sound(&self) -> Result<String> {
        self.read_string_from_offset(392)
    }

    fn write_cast_sound(&self, cast_sound: &str) -> Result<()> {
        self.write_string_to_offset(392, cast_sound)
    }

    fn aggro_sound(&self) -> Result<String> {
        self.read_string_from_offset(424)
    }

    fn write_aggro_sound(&self, aggro_sound: &str) -> Result<()> {
        self.write_string_to_offset(424, aggro_sound)
    }

    fn primary_school_name(&self) -> Result<String> {
        self.read_string_from_offset(456)
    }

    fn write_primary_school_name(&self, primary_school_name: &str) -> Result<()> {
        self.write_string_to_offset(456, primary_school_name)
    }

    fn location_preference(&self) -> Result<String> {
        self.read_string_from_offset(488)
    }

    fn write_location_preference(&self, location_preference: &str) -> Result<()> {
        self.write_string_to_offset(488, location_preference)
    }
}

pub struct DynamicWizGameObjectTemplate {
    inner: DynamicMemoryObject,
}

impl DynamicWizGameObjectTemplate {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::memory_object::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: DynamicMemoryObject::new(reader, base_address)?,
        })
    }
}

impl MemoryObject for DynamicWizGameObjectTemplate {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::memory_object::MemoryReader> {
        self.inner.reader()
    }

    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}

impl WizGameObjectTemplate for DynamicWizGameObjectTemplate {}
