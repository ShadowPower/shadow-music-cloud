use std::collections::{HashMap};

use once_cell::sync::Lazy;
use sled::Db;

use crate::{config::app_config::FILE_INFO_STORAGE_PATH, model::dto::FileInfo};

// TODO
// 检查没有访问权限
// 检查磁盘已满

static FILE_INFO_DB: Lazy<Db> = Lazy::new(|| {
    sled::open(FILE_INFO_STORAGE_PATH).unwrap()
});

pub fn get(file_info_hash: String) -> Option<FileInfo> {
    match FILE_INFO_DB.get(file_info_hash) {
        Ok(Some(value)) => {
            let file_info: FileInfo = serde_json::from_slice(&value).unwrap();
            Some(file_info)
        }, 
        _ => None,
    }
}

pub fn list() -> HashMap<String, FileInfo> {
    let mut file_infos: HashMap<String, FileInfo> = HashMap::new();
    for item in FILE_INFO_DB.iter() {
        let (key, value) = item.unwrap();
        let file_info: FileInfo = serde_json::from_slice(&value).unwrap();
        file_infos.insert(String::from_utf8(key.to_vec()).unwrap(), file_info);
    }
    file_infos
}

pub fn set(file_info_hash: String, file_info: &FileInfo) {
    let value = serde_json::to_vec(file_info).unwrap();
    FILE_INFO_DB.insert(file_info_hash, value).unwrap();
}

pub fn remove(file_info_hash: String) {
    FILE_INFO_DB.remove(file_info_hash).unwrap();
}

pub fn clear() {
    FILE_INFO_DB.clear().unwrap();
}

pub fn sync(data: &HashMap<String, FileInfo>) {
    // 删除
    for item in FILE_INFO_DB.iter() {
        let (file_info_hash, _) = item.unwrap();
        let file_info_hash = std::str::from_utf8(&file_info_hash).unwrap().to_string();
        if !data.contains_key(&file_info_hash) {
            remove(file_info_hash);
        }
    }
    // 添加
    for item in data.iter() {
        let (file_info_hash, file_info) = item;
        if !FILE_INFO_DB.contains_key(file_info_hash).unwrap() {
            set(file_info_hash.to_string(), file_info);
        }
    }
}