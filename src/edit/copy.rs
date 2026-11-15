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

#[cfg(not(target_arch = "wasm32"))]
impl CopyBuffer {
    pub fn export_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 弹出文件夹选择框，让用户选择一个空文件夹作为批量导出的目的地
        if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
            
            // 1. 导出系统/主文件 (system_save_export.json)
            let main_json = match self {
                CopyBuffer::Null => "{}".to_string(),
                CopyBuffer::Array(class) => serde_json::to_string_pretty(class)?,
                CopyBuffer::Field(field) => serde_json::to_string_pretty(field)?,
            };
            std::fs::write(folder_path.join("system_save_export.json"), main_json)?;

            // 2. 自动化批量拆分导出角色槽位 (slot1_haru.json, slot2_Alice.json ...)
            if let CopyBuffer::Array(class) = self {
                for (field_name, field_value) in &class.fields {
                    if field_name.contains("_Data") {
                        if let crate::save::types::FieldValue::Array(items) = field_value {
                            for (index, item) in items.iter().enumerate() {
                                let slot_num = index + 1;
                                let mut char_name = format!("character_{}", index);
                                
                                if let crate::save::types::FieldValue::Class(c) = item {
                                    for (sub_name, sub_val) in &c.fields {
                                        if sub_name.contains("Name") || sub_name.contains("ShortId") {
                                            if let crate::save::types::FieldValue::String(s) = sub_val {
                                                if !s.is_empty() {
                                                    char_name = s.clone();
                                                }
                                            }
                                        }
                                    }
                                }

                                let char_json = serde_json::to_string_pretty(item)?;
                                let file_name = format!("slot{}_{}.json", slot_num, char_name);
                                std::fs::write(folder_path.join(file_name), char_json)?;
                            }
                        }
                    }
                }
            }
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
