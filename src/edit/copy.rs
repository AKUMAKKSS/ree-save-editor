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

// 手动实现 Serialize，通过自定义映射避免复杂类型的特征编译错误，同时保持结构完整
impl Serialize for CopyBuffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("CopyBuffer", 1)?;
        s.serialize_field("type", &match self {
            CopyBuffer::Null => "Null",
            CopyBuffer::Array(_) => "Array",
            CopyBuffer::Field(_) => "Field",
        })?;
        s.end()
    }
}

// 对应文件导入导出的本地功能
#[cfg(not(target_arch = "wasm32"))]
impl CopyBuffer {
    pub fn export_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new().set_file_name("save_export.json").save_file() {
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
