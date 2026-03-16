use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject};
use super::quest_data::QuestData;
use std::collections::HashMap;

pub struct QuestClientManager {
    pub inner: DynamicMemoryObject,
}

impl QuestClientManager {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn quest_data(&self) -> Result<HashMap<i32, QuestData>> {
        Ok(HashMap::new())
    }
}
