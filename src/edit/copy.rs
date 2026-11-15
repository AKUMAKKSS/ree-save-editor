use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CopyBuffer {
    // 内部直接用字符串或者完全解耦的结构存储，避免触发复杂的外部类型 trait 检查
    pub data: Option<String>,
}

impl Default for CopyBuffer {
    fn default() -> Self {
        Self { data: None }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl CopyBuffer {
    pub fn export_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new().set_file_name("save_export.json").save_file() {
            if let Some(ref json_str) = self.data {
                std::fs::write(path, json_str)?;
            } else {
                // 如果没有暂存数据，则导出一个空对象
                std::fs::write(path, "{}")?;
            }
        }
        Ok(())
    }

    pub fn import_from_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            let json_str = std::fs::read_to_string(path)?;
            self.data = Some(json_str);
        }
        Ok(())
    }
}
