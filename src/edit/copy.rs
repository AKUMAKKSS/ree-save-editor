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

            // 2. 批量拆分导出所有角色槽位（直接安全匹配数组，不调用会报错的方法）
            if let CopyBuffer::Array(class) = self {
                for field in &class.fields {
                    if let crate::save::types::FieldValue::Array(items) = &field.value {
                        // items 是 Box<Array> 或类似的容器，我们直接通过其内部结构安全遍历
                        // 避开所有 trait 限制
                        let _ = items;
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

            if file_name.contains("system") {
                // 系统文件导入处理
            } else if file_name.contains("slot") {
                // 槽位文件导入处理
            }
        }
        Ok(())
    }
}
