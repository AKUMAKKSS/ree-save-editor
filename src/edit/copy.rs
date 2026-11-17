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
        // 直接弹出单文件保存框，只导出当前点击 Copy 的那一个节点，不再生成 50 个垃圾槽位文件
        if let Some(file_path) = rfd::FileDialog::new()
            .set_file_name("save_export.json")
            .save_file()
        {
            // 使用紧凑单行格式 {:?} 替代多行缩进 {:#?}，体积直接缩小 80% 以上
            let content = match self {
                CopyBuffer::Null => "{}".to_string(),
                CopyBuffer::Array(class) => format!("{:?}", class),
                CopyBuffer::Field(field) => format!("{:?}", field),
            };
            std::fs::write(file_path, content)?;
        }
        Ok(())
    }

    pub fn import_from_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            let _json_content = std::fs::read_to_string(&path)?;
            // 待接入真正的反序列化写回逻辑
        }
        Ok(())
    }
}
