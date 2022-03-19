use std::{collections::HashSet, path::PathBuf};
use once_cell::sync::Lazy;
use walkdir::{WalkDir, DirEntry};

use crate::{config::app_config, model::dto::SimpleFileInfo};

use super::time_utils;

/// Supported audio file extensions.
static AUDIO_EXTENSIONS : Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["wav", "mp3", "flac", "ogg", "m4a", "aac", "wma", "opus"]
        .iter()
        .cloned()
        .collect()
});

/// 获取媒体目录下的所有音频文件信息
/// @returns 音频文件信息列表
pub fn list_audio_file() -> Vec<SimpleFileInfo> {
    let mut audio_file_list: Vec<SimpleFileInfo> = Vec::new();
    let dir_map = WalkDir::new(app_config::AUDIO_PATH)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok());
    for entry in dir_map {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                let ext: &str = &ext.to_str().unwrap().to_lowercase();
                if AUDIO_EXTENSIONS.contains(ext) {
                    audio_file_list.push(dir_entry_to_simple_file_info(&entry));
                }
            }
        }
    }
    audio_file_list
}

/// 将 DirEntry 转换成文件信息结构体
/// @param entry DirEntry
/// @returns 文件信息结构体
fn dir_entry_to_simple_file_info(dir_entry: &DirEntry) -> SimpleFileInfo {
    let path = dir_entry.path().strip_prefix(app_config::AUDIO_PATH).unwrap();
    let metadata = dir_entry.metadata().unwrap();
    let size = metadata.len();
    let last_modified = time_utils::time_to_millis(&metadata.modified().unwrap());
    SimpleFileInfo::new(path, size, last_modified)
}
    
/// 路径转字符串列表
/// @param path 路径
/// @returns 字符串列表
pub fn path_to_list(path: &PathBuf) -> Vec<String> {
    path.iter().map(|e| e.to_str().unwrap().to_string()).collect()
}

/// 字符串列表转路径
/// @param list 字符串列表
/// @returns 路径
pub fn list_to_path(list: &Vec<String>) -> PathBuf {
    list.iter().fold(PathBuf::new(), |mut path, e| {
        path.push(e);
        path
    })
}