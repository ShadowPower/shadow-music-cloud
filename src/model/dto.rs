use std::{
    path::{PathBuf, Path}, ffi::OsStr
};

use radix_fmt::radix;
use serde::{Deserialize, Serialize};

use crate::infra::hash_utils;

/// 媒体信息
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    /// 序号
    pub track: u64,
    /// 标题
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// 歌手
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    /// 专辑
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    /// 音频数据 Hash
    pub audio_hash: String,
    /// 起始位置（毫秒）
    pub index_time: u64,
    /// 时长（毫秒）
    pub duration: u64,
    /// 比特率（比特每秒）
    pub bitrate: u64,
}

/// 文件信息
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    /// 文件路径
    pub path: Vec<String>,
    /// 文件类型 audio/cuesheet
    pub file_type: String,
    /// 文件大小
    pub size: u64,
    /// 修改时间
    pub last_modified: u128,
    /// 文件路径+大小+修改时间 Hash
    pub file_info_hash: String,
    /// cuesheet 关联的媒体文件路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cue_media_path: Option<String>,
    /// cuesheet 关联的媒体文件 Hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cue_media_file_info_hash: Option<String>,
    /// 专辑封面 Hash
    pub cover_hash: Option<String>,
    /// 媒体文件信息
    pub medias: Vec<MediaInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimpleFileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub last_modified: u128,
    /// 文件路径+大小+修改时间 Hash
    pub file_info_hash: Option<String>,
}

impl SimpleFileInfo {
    pub fn new(path: &Path, size: u64, last_modified: u128) -> SimpleFileInfo {
        SimpleFileInfo {
            path: path.to_path_buf(),
            size: size,
            last_modified: last_modified,
            file_info_hash: None,
        }
    }
}

impl FileInfo {
    pub fn from_simple(simple: &SimpleFileInfo) -> FileInfo {
        FileInfo {
            path: simple.path.components()
                .map(|x| x.as_os_str().to_string_lossy().into_owned())
                .collect(),
            file_type: if simple.path.extension() == Some(OsStr::new("cue")) {
                "cuesheet".to_string()
            } else {
                "audio".to_string()
            },
            size: simple.size,
            last_modified: simple.last_modified,
            file_info_hash: simple.file_info_hash.as_ref().unwrap_or({
                &radix(hash_utils::hash_media_file_info(simple), 36).to_string()
            }).to_string(),
            cue_media_path: None,
            cue_media_file_info_hash: None,
            cover_hash: None,
            medias: vec![],
        }
    }
}