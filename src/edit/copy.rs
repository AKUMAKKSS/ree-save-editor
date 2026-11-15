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
                    let content = format!("{:#?}", class);
                    std::fs::write(&default_path, &content)?;

                    // 2. 自动遍历字段，安全查找角色数组
                    for field in &class.fields {
                        // 利用 hash 或 value 匹配数组
                        if let crate::save::types::FieldValue::Array(items) = &field.value {
                            for (index, item) in items.iter().enumerate() {
                                let slot_num = index + 1;
                                let char_name = format!("character_{}", index);

                                // 格式化并独立导出该角色的内容
                                let char_content = format!("{:#?}", item);
                                let file_name = format!("slot{}_{}.json", slot_num, char_name);
                                let _ = std::fs::write(folder_path.join(file_name), char_content);
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
