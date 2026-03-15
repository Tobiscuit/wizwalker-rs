use crate::errors::Result;
use crate::memory::memory_object::MemoryReader;
use crate::memory::objects::core_object::CoreObject;
use crate::memory::objects::actor_body::ActorBody;
use crate::memory::objects::client_zone::ClientZone;
use std::sync::Arc;

pub struct ClientObject<R: MemoryReader + 'static> {
    pub base: CoreObject<R>,
}

impl<R: MemoryReader + 'static> ClientObject<R> {
    pub fn new(reader: Arc<R>, base_address: u64) -> Self {
        Self {
            base: CoreObject::new(reader, base_address),
        }
    }

    pub fn reader(&self) -> Arc<R> {
        Arc::clone(&self.base.reader)
    }

    pub fn read_base_address(&self) -> Result<u64> {
        self.base.read_base_address()
    }

    pub async fn try_get_inventory_behavior(&self) -> Result<Option<u64>> {
        let behavior = self.base.search_behavior_by_name("WizardInventoryBehavior").await?;
        Ok(behavior)
    }

    pub async fn try_get_equipment_behavior(&self) -> Result<Option<u64>> {
        let behavior = self.base.search_behavior_by_name("WizardEquipmentBehavior").await?;
        Ok(behavior)
    }

    pub async fn actor_body(&self) -> Result<Option<ActorBody<R>>> {
        let behavior = self.base.search_behavior_by_name("AnimationBehavior").await?;
        if let Some(addr) = behavior {
            // Need to read from the behavior address 0x70
            let animation_behavior_addr = self.reader().read_typed::<u64>((addr + 0x70) as usize)?;
            if animation_behavior_addr == 0 {
                return Ok(None);
            }
            return Ok(Some(ActorBody::new(self.reader(), animation_behavior_addr)));
        }
        Ok(None)
    }

    pub async fn object_name(&self) -> Result<Option<String>> {
        Ok(None)
    }

    pub async fn display_name(&self) -> Result<Option<String>> {
        Ok(None)
    }

    pub async fn parent(&self) -> Result<Option<ClientObject<R>>> {
        let core_parent = self.base.parent().await?;
        if let Some(p) = core_parent {
            Ok(Some(ClientObject::new(self.reader(), p.read_base_address()?)))
        } else {
            Ok(None)
        }
    }

    pub async fn children(&self) -> Result<Vec<ClientObject<R>>> {
        Ok(vec![])
    }

    pub async fn client_zone(&self) -> Result<Option<ClientZone<R>>> {
        let addr = self.base.read_value_from_offset::<i64>(304)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(ClientZone::new(self.reader(), addr as u64)))
    }

    pub async fn object_template(&self) -> Result<Option<u64>> {
        let core_template = self.base.object_template().await?;
        if let Some(t) = core_template {
            Ok(Some(t.read_base_address()?))
        } else {
            Ok(None)
        }
    }

    pub async fn character_id(&self) -> Result<u64> {
        self.base.read_value_from_offset::<u64>(448)
    }

    pub async fn write_character_id(&self, character_id: u64) -> Result<()> {
        self.base.write_value_to_offset::<u64>(448, &character_id)
    }

    pub async fn game_stats(&self) -> Result<Option<u64>> {
        let addr = self.base.read_value_from_offset::<i64>(560)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(addr as u64))
    }

    pub async fn fetch_npc_behavior_template(&self) -> Result<Option<u64>> {
        Ok(None)
    }
}
