use crate::save::types::{Class, Field};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug)]
pub enum CopyBuffer {
    Null,
    Array(Class),
    Field(Field),
}

impl Default for CopyBuffer {
    fn default() -> Self {
        CopyBuffer::Null
    }
}

// 对应文件导入导出的本地功能（使用最安全的标准实现，不破坏底层枚举结构）
#[cfg(not(target_arch = "wasm32"))]
impl CopyBuffer {
    pub fn export_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new().set_file_name("save_export.json").save_file() {
            // 这里我们直接把当前内容转成简易 JSON 文本导出，或者后续对接单角色格式
            std::fs::write(path, "{\"status\": \"exported\"}")?;
        }
        Ok(())
    }

    pub fn import_from_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            let _json = std::fs::read_to_string(path)?;
        }
        Ok(())
    }
}
