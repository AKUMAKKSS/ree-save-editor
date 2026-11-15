use crate::save::types::{Class, Field};

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
        // 弹出文件夹选择框，选择批量导出的目标目录
        if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
            
            // 1. 导出系统/主文件 (system_save_export.json)
            let default_path = folder_path.join("system_save_export.json");
            match self {
                CopyBuffer::Null => {
                    std::fs::write(&default_path, "{}")?;
                }
                CopyBuffer::Array(class) => {
                    // 使用标准的 JSON 序列化（如果支持）或者项目内置转换
                    // 这里我们采用标准的结构格式化写入
                    let content = serde_json::to_string_pretty(class).unwrap_or_else(|_| format!("{:#?}", class));
                    std::fs::write(&default_path, content)?;

                    // 2. 自动遍历并批量导出所有角色槽位 (slot1_haru.json 等)
                    for field in &class.fields {
                        if field.name.contains("_Data") {
                            if let crate::save::types::FieldValue::Array(items) = &field.value {
                                for (index, item) in items.iter().enumerate() {
                                    let slot_num = index + 1;
                                    let mut char_name = format!("character_{}", index);

                                    if let crate::save::types::FieldValue::Class(c) = item {
                                        for sub_field in &c.fields {
                                            if sub_field.name.contains("Name") || sub_field.name.contains("ShortId") {
                                                if let crate::save::types::FieldValue::String(s) = &sub_field.value {
                                                    let s_str = format!("{:?}", s);
                                                    if !s_str.is_empty() {
                                                        char_name = sub_field.name.clone();
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    let char_content = serde_json::to_string_pretty(item).unwrap_or_else(|_| format!("{:#?}", item));
                                    let file_name = format!("slot{}_{}.json", slot_num, char_name);
                                    let _ = std::fs::write(folder_path.join(file_name), char_content);
                                }
                            }
                        }
                    }
                }
                CopyBuffer::Field(field) => {
                    let content = format!("{:#?}", field);
                    std::fs::write(&default_path, content)?;
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
