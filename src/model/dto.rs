use std::{
    path::{PathBuf, Path}, ffi::OsStr, fs::{File, self}, io::Write
};

use lofty::{TagItem, ItemKey, ItemValue, Accessor};
use radix_fmt::radix;
use serde::{Deserialize, Serialize};

use crate::{infra::{hash_utils, audio_utils}, config};

/// 媒体信息
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    /// 序号
    pub track: u32,
    /// 光碟序号
    pub disc: u32,
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
    pub index_time: u128,
    /// 时长（毫秒）
    pub duration: u128,
    /// 比特率（比特每秒）
    pub bitrate: u32,
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
        // 注：目前不支持 cuesheet 文件
        let mut media_info = MediaInfo::default();
        let mut cover_hash: Option<String> = None;

        let media_file_path = PathBuf::from(config::app_config::AUDIO_PATH).join(&simple.path);
        // 获取音频属性
        match audio_utils::get_properties_from_media_file(&media_file_path) {
            Ok(properties) => {
                media_info.bitrate = properties.audio_bitrate().unwrap_or(0);
                media_info.duration = properties.duration().as_micros();
            },
            Err(err) => println!("{}", err),
        }
        // 获取音频标签
        match audio_utils::get_tags_from_media_file(&media_file_path) {
            Ok(tag) => {
                // 如果文件有标签
                media_info.title = tag.title().map(|s| s.to_string());
                media_info.artist = tag.artist().map(|s| s.to_string());
                media_info.album = tag.album().map(|s| s.to_string());
                if let Some(track) = tag.get_item_ref(&ItemKey::TrackNumber).map(TagItem::value) {
                    match track {
                        ItemValue::Text(text) => media_info.track = text.parse().unwrap_or(1),
                        ItemValue::Locator(text) => media_info.track = text.parse().unwrap_or(1),
                        ItemValue::Binary(binary) => {
                            panic!("{} has binary track number: {:?}", simple.path.display(), binary)
                        },
                    }
                } else {
                    media_info.track = 1;
                }
                if let Some(disc) = tag.get_item_ref(&ItemKey::DiscNumber).map(TagItem::value) {
                    match disc {
                        ItemValue::Text(text) => media_info.disc = text.parse().unwrap_or(1),
                        ItemValue::Locator(text) => media_info.disc = text.parse().unwrap_or(1),
                        ItemValue::Binary(binary) => {
                            panic!("{} has binary disc number: {:?}", simple.path.display(), binary)
                        },
                    }
                } else {
                    media_info.disc = 1;
                }
                // 提取专辑封面
                if tag.picture_count() > 0 {
                    let first_picture = tag.pictures().first().unwrap();
                    let cover_picture = tag.get_picture_type(lofty::PictureType::CoverFront)
                        .unwrap_or(first_picture);
                    let picture_data_hash = radix(hash_utils::hash_data(cover_picture.data()), 36).to_string();
                    let cover_path = PathBuf::from(config::app_config::COVER_PATH).join(&picture_data_hash);
                    cover_hash = Some(picture_data_hash);
                    // 保存专辑封面到文件
                    if !cover_path.exists() {
                        fs::create_dir_all(&cover_path.parent().unwrap()).unwrap();
                        let mut cover_file = File::create(&cover_path).unwrap();
                        cover_file.write_all(cover_picture.data()).unwrap();
                    }
                }
            },
            Err(err) => println!("{}", err),
        }
        // 计算音频数据 Hash
        match hash_utils::hash_audio_data(&media_file_path) {
            Ok(hash) => media_info.audio_hash = radix(hash, 36).to_string(),
            Err(err) => println!("{}", err),
        }

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
            cover_hash: cover_hash,
            medias: vec![media_info],
        }
    }
}