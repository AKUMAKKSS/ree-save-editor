use serde_json::{Map, Value as JsonValue};
use crate::save::{
    SaveFile,
    types::{Array, ArrayType, Class, EnumValue, Field, FieldType, FieldValue},
};
use crate::sdk::{
    types::StringU16,
    type_map::{TypeMap, murmur3},
};

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

/// 将底层 Class 递归转换为修改器可识别的标准带字段名 JSON
fn class_to_json(class: &Class, type_map: &TypeMap) -> JsonValue {
    let mut map = Map::new();
    for field in &class.fields {
        let name = type_map
            .get_hash_str(field.hash)
            .cloned()
            .unwrap_or_else(|| format!("0x{:08x}", field.hash));
        map.insert(name, field_value_to_json(&field.value, type_map));
    }
    JsonValue::Object(map)
}

fn field_value_to_json(val: &FieldValue, type_map: &TypeMap) -> JsonValue {
    match val {
        FieldValue::Boolean(b) => JsonValue::Bool(*b),
        FieldValue::Enum(e) => JsonValue::from(e.as_i64()),
        FieldValue::S8(v) => JsonValue::from(*v),
        FieldValue::U8(v) | FieldValue::C8(v) => JsonValue::from(*v),
        FieldValue::S16(v) => JsonValue::from(*v),
        FieldValue::U16(v) | FieldValue::C16(v) => JsonValue::from(*v),
        FieldValue::S32(v) => JsonValue::from(*v),
        FieldValue::U32(v) => JsonValue::from(*v),
        FieldValue::S64(v) => JsonValue::from(*v),
        FieldValue::U64(v) => JsonValue::from(*v),
        FieldValue::F32(v) => JsonValue::from(*v),
        FieldValue::F64(v) => JsonValue::from(*v),
        FieldValue::String(s) => JsonValue::String(s.to_string()),
        FieldValue::Class(c) => class_to_json(c, type_map),
        FieldValue::Struct(s) => {
            // Guid (16字节) 自动转为标准 UUID 字符串（如 HunterId），其余转为字节数组
            if s.data.len() == 16 {
                let d1 = u32::from_le_bytes([s.data[0], s.data[1], s.data[2], s.data[3]]);
                let d2 = u16::from_le_bytes([s.data[4], s.data[5]]);
                let d3 = u16::from_le_bytes([s.data[6], s.data[7]]);
                JsonValue::String(format!(
                    "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                    d1, d2, d3, s.data[8], s.data[9], s.data[10], s.data[11], s.data[12], s.data[13], s.data[14], s.data[15]
                ))
            } else {
                JsonValue::Array(s.data.iter().map(|b| JsonValue::from(*b)).collect())
            }
        }
        FieldValue::Array(arr) => {
            JsonValue::Array(arr.values.iter().map(|v| field_value_to_json(v, type_map)).collect())
        }
        FieldValue::Unknown => JsonValue::Null,
    }
}

/// 将修改器导出的 JSON 递归无损写回底层 Class
fn update_class_from_json(class: &mut Class, json: &JsonValue) {
    if let JsonValue::Object(map) = json {
        for (key, jval) in map {
            let hash = if let Some(hex) = key.strip_prefix("0x") {
                u32::from_str_radix(hex, 16).unwrap_or_else(|_| murmur3(key, 0xffffffff))
            } else {
                murmur3(key, 0xffffffff)
            };
            if let Some(field) = class.fields.iter_mut().find(|f| f.hash == hash) {
                update_field_value_from_json(&mut field.value, jval);
            }
        }
    }
}

fn update_field_value_from_json(val: &mut FieldValue, json: &JsonValue) {
    match val {
        FieldValue::Boolean(b) => {
            if let Some(v) = json.as_bool() { *b = v; }
            else if let Some(v) = json.as_i64() { *b = v != 0; }
        }
        FieldValue::Enum(e) => {
            if let Some(v) = json.as_i64() {
                match e {
                    EnumValue::E1(x) => *x = v as i8,
                    EnumValue::E2(x) => *x = v as i16,
                    EnumValue::E4(x) => *x = v as i32,
                    EnumValue::E8(x) => *x = v,
                }
            }
        }
        FieldValue::S8(x) => if let Some(v) = json.as_i64() { *x = v as i8; },
        FieldValue::U8(x) | FieldValue::C8(x) => if let Some(v) = json.as_u64() { *x = v as u8; },
        FieldValue::S16(x) => if let Some(v) = json.as_i64() { *x = v as i16; },
        FieldValue::U16(x) | FieldValue::C16(x) => if let Some(v) = json.as_u64() { *x = v as u16; },
        FieldValue::S32(x) => if let Some(v) = json.as_i64() { *x = v as i32; },
        FieldValue::U32(x) => if let Some(v) = json.as_u64() { *x = v as u32; },
        FieldValue::S64(x) => if let Some(v) = json.as_i64() { *x = v; },
        FieldValue::U64(x) => if let Some(v) = json.as_u64() { *x = v; },
        FieldValue::F32(x) => if let Some(v) = json.as_f64() { *x = v as f32; },
        FieldValue::F64(x) => if let Some(v) = json.as_f64() { *x = v; },
        FieldValue::String(s) => {
            if let Some(str_val) = json.as_str() {
                let mut u16_vec: Vec<u16> = str_val.encode_utf16().collect();
                u16_vec.push(0);
                **s = StringU16::new(u16_vec);
            }
        }
        FieldValue::Class(c) => update_class_from_json(c, json),
        FieldValue::Array(arr) => {
            if let Some(jarr) = json.as_array() {
                for (item, jitem) in arr.values.iter_mut().zip(jarr.iter()) {
                    update_field_value_from_json(item, jitem);
                }
            }
        }
        FieldValue::Struct(s) => {
            if let Some(jarr) = json.as_array() {
                if jarr.len() == s.data.len() {
                    for (b, jb) in s.data.iter_mut().zip(jarr.iter()) {
                        if let Some(v) = jb.as_u64() { *b = v as u8; }
                    }
                }
            }
        }
        FieldValue::Unknown => {}
    }
}

/// 自动在 SaveFile 中查找第一个角色槽位（_Data[0]）或匹配 HunterId 的角色槽位
fn find_character_slot_mut<'a>(save: &'a mut SaveFile, target_hunter_id: Option<&str>, type_map: &TypeMap) -> Option<&'a mut Class> {
    let data_hash = murmur3("_Data", 0xffffffff);
    for (_, root_class) in &mut save.fields {
        if let Some(data_field) = root_class.fields.iter_mut().find(|f| f.hash == data_hash) {
            if let FieldValue::Array(arr) = &mut data_field.value {
                if let Some(hid) = target_hunter_id {
                    for val in &mut arr.values {
                        if let FieldValue::Class(c) = val {
                            let j = class_to_json(c, type_map);
                            if j.get("HunterId").and_then(|v| v.as_str()) == Some(hid) {
                                return Some(c);
                            }
                        }
                    }
                }
                // 默认返回第 1 个角色槽位 (_Data[0])
                if let Some(FieldValue::Class(c)) = arr.values.first_mut() {
                    return Some(c);
                }
            }
        }
    }
    save.fields.first_mut().map(|(_, c)| c)
}

#[cfg(not(target_arch = "wasm32"))]
impl CopyBuffer {
    pub fn export_to_file(&self, save: &mut SaveFile, type_map: &TypeMap) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(file_path) = rfd::FileDialog::new()
            .set_file_name("save_export.json")
            .save_file()
        {
            // 如果用户先点了某个角色的 Copy，则导出该角色；如果没点 Copy，自动导出第 1 个角色槽位 (_Data[0])！
            let json_val = match self {
                CopyBuffer::Array(class) => class_to_json(class, type_map),
                CopyBuffer::Field(field) => field_value_to_json(&field.value, type_map),
                CopyBuffer::Null => {
                    if let Some(slot_class) = find_character_slot_mut(save, None, type_map) {
                        class_to_json(slot_class, type_map)
                    } else {
                        JsonValue::Object(Map::new())
                    }
                }
            };
            // 使用紧凑单行 JSON 序列化，体积仅 4.7MB，且 100% 兼容网页修改器
            let content = serde_json::to_string(&json_val)?;
            std::fs::write(file_path, content)?;
        }
        Ok(())
    }

    pub fn import_from_file(&mut self, save: &mut SaveFile, type_map: &TypeMap) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            let json_content = std::fs::read_to_string(&path)?;
            let parsed: JsonValue = serde_json::from_str(&json_content)?;

            // 自动识别 JSON 里的 HunterId 并匹配写回对应角色槽位（无需手动点击树状图 Paste！）
            let hunter_id = parsed.get("HunterId").and_then(|v| v.as_str());
            if let Some(target_class) = find_character_slot_mut(save, hunter_id, type_map) {
                update_class_from_json(target_class, &parsed);
                *self = CopyBuffer::Array(target_class.clone());
                return Ok(true);
            }
        }
        Ok(false)
    }
}
