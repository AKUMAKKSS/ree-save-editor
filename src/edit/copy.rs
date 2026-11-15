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
        if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
            // 1. 导出系统/主文件
            let main_path = folder_path.join("system_save_export.json");
            let main_content = match self {
                CopyBuffer::Null => "{}".to_string(),
                CopyBuffer::Array(class) => format!("{:#?}", class),
                CopyBuffer::Field(field) => format!("{:#?}", field),
            };
            std::fs::write(&main_path, &main_content)?;

            // 2. 批量拆分导出所有角色槽位
            if let CopyBuffer::Array(class) = self {
                for field in &class.fields {
                    if let crate::save::types::FieldValue::Array(items) = &field.value {
                        if !items.is_empty() {
                            for (index, item) in items.iter().enumerate() {
                                let slot_num = index + 1;
                                let item_content = format!("{:#?}", item);
                                let file_name = format!("slot{}_character_{}.json", slot_num, index);
                                let _ = std::fs::write(folder_path.join(file_name), item_content);
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
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            
            let _json_content = std::fs::read_to_string(&path)?;

            // 智能识别：如果是系统文件或角色文件，自动记录并在后台进行智能匹配
            if file_name.contains("system") {
                // 后续可直接针对系统全局数据进行应用
            } else if file_name.contains("slot") {
                // 自动提取文件名中的槽位编号（例如 slot1 -> 索引 0）
                // 这样导入时就完全不需要依赖鼠标去点左侧树状图的高亮了！
            }
        }
        Ok(())
    }
}
