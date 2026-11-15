#[cfg(not(target_arch = "wasm32"))]
impl CopyBuffer {
    pub fn export_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new().set_file_name("save_export.json").save_file() {
            // 根据当前 CopyBuffer 里存的内容，将其序列化为漂亮的 JSON 字符串
            let json_str = match self {
                CopyBuffer::Null => "{}".to_string(),
                CopyBuffer::Array(class) => serde_json::to_string_pretty(class)?,
                CopyBuffer::Field(field) => serde_json::to_string_pretty(field)?,
            };
            std::fs::write(path, json_str)?;
        }
        Ok(())
    }

    pub fn import_from_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            let _json = std::fs::read_to_string(path)?;
            // 后续我们在这里对接单角色的 JSON 覆盖导入
        }
        Ok(())
    }
}
