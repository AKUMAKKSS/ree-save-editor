use std::str::FromStr;
use serde_json::{Map, Value as JsonValue};
use uuid::Uuid;

use crate::save::{
    SaveFile,
    types::{Class, EnumValue, Field, FieldValue},
};
use crate::sdk::{
    types::{Mandrake, StringU16},
    type_map::{FieldInfo, TypeMap, murmur3},
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
    let type_info = type_map.get_by_hash(class.hash);

    for field in &class.fields {
        let field_info = type_info.and_then(|ti| ti.get_by_hash(field.hash));
        let name = field_info
            .map(|fi| fi.name.clone())
            .or_else(|| type_map.get_hash_str(field.hash).cloned())
            .unwrap_or_else(|| format!("0x{:08x}", field.hash));
        map.insert(name, field_value_to_json(&field.value, field_info, type_map));
    }
    JsonValue::Object(map)
}

fn field_value_to_json(
    val: &FieldValue,
    field_info: Option<&FieldInfo>,
    type_map: &TypeMap,
) -> JsonValue {
    match val {
        FieldValue::Boolean(b) => JsonValue::from(if *b { 1 } else { 0 }),
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
            let og_type = field_info.map(|f| f.original_type.as_str()).unwrap_or("");
            if og_type == "via.rds.Mandrake" {
                if let Ok(m) = Mandrake::try_from(s.as_ref()) {
                    if let Some(real_val) = m.get() {
                        return JsonValue::from(real_val);
                    }
                }
            }
            if og_type == "System.Guid" || s.data.len() == 16 {
                if let Ok(bytes) = <[u8; 16]>::try_from(s.data.as_slice()) {
                    return JsonValue::String(Uuid::from_bytes_le(bytes).to_string());
                }
            }
            JsonValue::Array(s.data.iter().map(|b| JsonValue::from(*b)).collect())
        }
        FieldValue::Array(arr) => JsonValue::Array(
            arr.values
                .iter()
                .map(|v| field_value_to_json(v, field_info, type_map))
                .collect(),
        ),
        FieldValue::Unknown => JsonValue::Null,
    }
}

/// 将修改器导出的 JSON 递归无损写回底层 Class
fn update_class_from_json(class: &mut Class, json: &JsonValue, type_map: &TypeMap) {
    if let JsonValue::Object(map) = json {
        let type_info = type_map.get_by_hash(class.hash);
        for (key, jval) in map {
            let hash = if let Some(hex) = key.strip_prefix("0x") {
                u32::from_str_radix(hex, 16).unwrap_or_else(|_| murmur3(key, 0xffffffff))
            } else {
                murmur3(key, 0xffffffff)
            };
            let field_info = type_info.and_then(|ti| ti.get_by_hash(hash));
            if let Some(field) = class.fields.iter_mut().find(|f| f.hash == hash) {
                update_field_value_from_json(&mut field.value, jval, field_info, type_map);
            }
        }
    }
}

fn update_field_value_from_json(
    val: &mut FieldValue,
    json: &JsonValue,
    field_info: Option<&FieldInfo>,
    type_map: &TypeMap,
) {
    match val {
        FieldValue::Boolean(b) => {
            if let Some(v) = json.as_bool() {
                *b = v;
            } else if let Some(v) = json.as_i64() {
                *b = v != 0;
            }
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
        FieldValue::S8(x) => {
            if let Some(v) = json.as_i64() {
                *x = v as i8;
            }
        }
        FieldValue::U8(x) | FieldValue::C8(x) => {
            if let Some(v) = json.as_u64() {
                *x = v as u8;
            }
        }
        FieldValue::S16(x) => {
            if let Some(v) = json.as_i64() {
                *x = v as i16;
            }
        }
        FieldValue::U16(x) | FieldValue::C16(x) => {
            if let Some(v) = json.as_u64() {
                *x = v as u16;
            }
        }
        FieldValue::S32(x) => {
            if let Some(v) = json.as_i64() {
                *x = v as i32;
            }
        }
        FieldValue::U32(x) => {
            if let Some(v) = json.as_u64() {
                *x = v as u32;
            }
        }
        FieldValue::S64(x) => {
            if let Some(v) = json.as_i64() {
                *x = v;
            }
        }
        FieldValue::U64(x) => {
            if let Some(v) = json.as_u64() {
                *x = v;
            }
        }
        FieldValue::F32(x) => {
            if let Some(v) = json.as_f64() {
                *x = v as f32;
            }
        }
        FieldValue::F64(x) => {
            if let Some(v) = json.as_f64() {
                *x = v;
            }
        }
        FieldValue::String(s) => {
            if let Some(str_val) = json.as_str() {
                let mut u16_vec: Vec<u16> = str_val.encode_utf16().collect();
                u16_vec.push(0);
                **s = StringU16::new(u16_vec);
            }
        }
        FieldValue::Class(c) => update_class_from_json(c, json, type_map),
        FieldValue::Array(arr) => {
            if let Some(jarr) = json.as_array() {
                for (item, jitem) in arr.values.iter_mut().zip(jarr.iter()) {
                    update_field_value_from_json(item, jitem, field_info, type_map);
                }
            }
        }
        FieldValue::Struct(s) => {
            let og_type = field_info.map(|f| f.original_type.as_str()).unwrap_or("");
            if og_type == "via.rds.Mandrake" {
                if let Some(v) = json.as_i64() {
                    if let Ok(mut m) = Mandrake::try_from(&**s) {
                        m.set(v);
                        s.data = m.to_buf().to_vec();
                        return;
                    }
                }
            }
            if let Some(str_val) = json.as_str() {
                if s.data.len() == 16 {
                    if let Ok(uuid) = Uuid::from_str(str_val) {
                        s.data = uuid.to_bytes_le().to_vec();
                        return;
                    }
                }
            }
            if let Some(jarr) = json.as_array() {
                if jarr.len() == s.data.len() {
                    for (b, jb) in s.data.iter_mut().zip(jarr.iter()) {
                        if let Some(v) = jb.as_u64() {
                            *b = v as u8;
                        }
                    }
                }
            }
        }
        FieldValue::Unknown => {}
    }
}

fn get_class_hunter_id(class: &Class) -> Option<String> {
    let hid_hash = murmur3("HunterId", 0xffffffff);
    let field = class.fields.iter().find(|f| f.hash == hid_hash)?;
    match &field.value {
        FieldValue::String(s) => Some(s.to_string()),
        FieldValue::Struct(s) if s.data.len() == 16 => {
            let bytes: [u8; 16] = s.data.as_slice().try_into().ok()?;
            Some(Uuid::from_bytes_le(bytes).to_string())
        }
        _ => None,
    }
}

/// 先用只读引用查找目标角色槽位下标，再按索引返回可变引用（完全避免 E0499 二次可变借用报错）
fn find_character_slot_mut<'a>(
    save: &'a mut SaveFile,
    target_hunter_id: Option<&str>,
) -> Option<&'a mut Class> {
    let data_hash = murmur3("_Data", 0xffffffff);
    let mut found_indices: Option<(usize, usize, usize)> = None;

    for (r_idx, (_, root_class)) in save.fields.iter().enumerate() {
        if let Some((f_idx, data_field)) = root_class
            .fields
            .iter()
            .enumerate()
            .find(|(_, f)| f.hash == data_hash)
        {
            if let FieldValue::Array(arr) = &data_field.value {
                if let Some(hid) = target_hunter_id {
                    for (a_idx, val) in arr.values.iter().enumerate() {
                        if let FieldValue::Class(c) = val {
                            if get_class_hunter_id(c).as_deref() == Some(hid) {
                                found_indices = Some((r_idx, f_idx, a_idx));
                                break;
                            }
                        }
                    }
                }
                if found_indices.is_none() && !arr.values.is_empty() {
                    found_indices = Some((r_idx, f_idx, 0));
                }
                if found_indices.is_some() {
                    break;
                }
            }
        }
    }

    if let Some((r_idx, f_idx, a_idx)) = found_indices {
        if let FieldValue::Array(arr) = &mut save.fields[r_idx].1.fields[f_idx].value {
            if let Some(FieldValue::Class(c)) = arr.values.get_mut(a_idx) {
                return Some(c);
            }
        }
    }

    save.fields.first_mut().map(|(_, c)| c)
}

#[cfg(not(target_arch = "wasm32"))]
impl CopyBuffer {
    pub fn export_to_file(
        &self,
        save: &mut SaveFile,
        type_map: &TypeMap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(file_path) = rfd::FileDialog::new()
            .set_file_name("save_export.json")
            .save_file()
        {
            let json_val = match self {
                CopyBuffer::Array(class) => class_to_json(class, type_map),
                CopyBuffer::Field(field) => field_value_to_json(&field.value, None, type_map),
                CopyBuffer::Null => {
                    if let Some(slot_class) = find_character_slot_mut(save, None) {
                        class_to_json(slot_class, type_map)
                    } else {
                        JsonValue::Object(Map::new())
                    }
                }
            };
            let content = serde_json::to_string(&json_val)?;
            std::fs::write(file_path, content)?;
        }
        Ok(())
    }

    pub fn import_from_file(
        &mut self,
        save: &mut SaveFile,
        type_map: &TypeMap,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            let json_content = std::fs::read_to_string(&path)?;
            let parsed: JsonValue = serde_json::from_str(&json_content)?;

            let hunter_id = parsed.get("HunterId").and_then(|v| v.as_str());
            if let Some(target_class) = find_character_slot_mut(save, hunter_id) {
                update_class_from_json(target_class, &parsed, type_map);
                *self = CopyBuffer::Array(target_class.clone());
                return Ok(true);
            }
        }
        Ok(false)
    }
}
