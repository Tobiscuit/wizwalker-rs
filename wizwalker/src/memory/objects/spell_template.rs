use crate::memory::MemoryObject;

#[async_trait::async_trait]
pub trait SpellTemplate: MemoryObject {
    async fn name(&self) -> String {
        self.read_string_from_offset(96).await.unwrap_or_default()
    }

    async fn write_name(&self, name: String) {
        let _ = self.write_string_to_offset(96, &name).await;
    }

    async fn description(&self) -> String {
        self.read_string_from_offset(168).await.unwrap_or_default()
    }

    async fn write_description(&self, description: String) {
        let _ = self.write_string_to_offset(168, &description).await;
    }

    async fn advanced_description(&self) -> String {
        self.read_string_from_offset(200).await.unwrap_or_default()
    }

    async fn write_advanced_description(&self, advanced_description: String) {
        let _ = self.write_string_to_offset(200, &advanced_description).await;
    }

    async fn display_name(&self) -> String {
        self.read_string_from_offset(136).await.unwrap_or_default()
    }

    async fn write_display_name(&self, display_name: String) {
        let _ = self.write_string_to_offset(136, &display_name).await;
    }

    async fn spell_base(&self) -> String {
        self.read_string_from_offset(248).await.unwrap_or_default()
    }

    async fn write_spell_base(&self, spell_base: String) {
        let _ = self.write_string_to_offset(248, &spell_base).await;
    }
}
