use std::{hash::Hasher, path::PathBuf, io::Read};

use ffmpeg_next::format;
use xxhash_rust::xxh3::Xxh3;
use anyhow::*;

use crate::{
    model::dto::SimpleFileInfo,
    config::app_config::HASH_SEED
};

use super::audio_utils::get_best_audio_stream_index;

/// 计算 Hash 值
fn hash(f: &dyn Fn(&mut Xxh3)) -> u128 {
    let mut hasher = Xxh3::with_seed(HASH_SEED);
    f(&mut hasher);
    hasher.digest128()
}

/// 计算文件信息的 Hash 值
/// @param file_info 文件信息
/// @return Hash 值
pub fn hash_media_file_info(file_info: &SimpleFileInfo) -> u128 {
    // 简单处理不同平台的文件路径差异
    let origin_file_path = file_info.path.as_os_str().to_str().unwrap();
    let unify_file_path = origin_file_path.replace("\\", "/");
    hash(&|hasher| {
        hasher.write(unify_file_path.as_bytes());
        hasher.write_u64(file_info.size);
        hasher.write_u128(file_info.last_modified);
    })
}

/// 计算媒体文件音频数据的 Hash 值
/// @param file_path 媒体文件路径
/// @return 音频数据 Hash 值
pub fn hash_audio_data(file_path: &PathBuf) -> Result<u128> {
    let mut hasher = Xxh3::with_seed(HASH_SEED);
    let mut input_ctx = format::input(file_path)?;
    if let Some(audio_stream_index) = get_best_audio_stream_index(&input_ctx) {
        for (stream, packet) in input_ctx.packets() {
            if stream.index() == audio_stream_index {
                match packet.data() {
                    Some(data) => {
                        hasher.write(data);
                    },
                    _ => {}
                }
            }
        }
    } else {
        return Err(anyhow!("No audio stream found"));
    }
    Ok(hasher.digest128())
}

/// 计算文件的 Hash 值
/// @param file_path 文件路径
/// @return Hash 值
pub fn hash_file(file_path: &PathBuf) -> Result<u128> {
    let mut hasher = Xxh3::with_seed(HASH_SEED);
    let mut file = std::fs::File::open(file_path)?;
    let mut buffer = [0u8; 256];
    loop {
        let read_size = file.read(&mut buffer)?;
        if read_size == 0 {
            break;
        }
        hasher.write(&buffer[0..read_size]);
    }
    Ok(hasher.digest128())
}