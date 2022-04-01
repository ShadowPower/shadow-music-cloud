use anyhow::{Result, Ok};
use radix_fmt::radix;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator, IntoParallelRefMutIterator};
use std::{collections::{HashMap, HashSet}, fs, path::PathBuf};

use crate::{
    infra::{file_utils, hash_utils},
    model::dto::FileInfo,
    repository::file_info, config::app_config,
};

use super::action::ContextData;

/// 命令
/// 一组命令组合成一个动作
/// 动作中的命令会按顺序串行执行
/// 多个命令之间可以共享内存
pub trait Command {
    /// 执行命令
    /// @param context 动作上下文
    fn execute(&self, context: &mut HashMap<&str, ContextData>) -> Result<()>;
    /// 失败时的回滚
    /// @param context 动作上下文
    fn rollback(&self, _context: &mut HashMap<&str, ContextData>) {
        // do nothing
    }
}

/// 扫描目录下的所有音频文件
struct ScanMediaFile;
impl Command for ScanMediaFile {
    fn execute(&self, context: &mut HashMap<&str, ContextData>) -> Result<()> {
        let audio_file_list = file_utils::list_audio_file();
        context.insert("simple_file_list", ContextData::FileList(audio_file_list));
        Ok(())
    }
}

/// 计算文件信息 Hash
struct CalcFileInfoHash;
impl Command for CalcFileInfoHash {
    fn execute(&self, context: &mut HashMap<&str, ContextData>) -> Result<()> {
        if let ContextData::FileList(file_list) = context.get_mut("simple_file_list").unwrap() {
            // 计算 Hash
            file_list.par_iter_mut().for_each(|file_info| {
                let hash = radix(hash_utils::hash_media_file_info(file_info), 36).to_string();
                file_info.file_info_hash = Some(hash);
            });
        }
        Ok(())
    }
}

/// 清理旧数据
struct CleanStorage;
impl Command for CleanStorage {
    fn execute(&self, context: &mut HashMap<&str, ContextData>) -> Result<()> {
        if let ContextData::FileList(simple_file_list) = context.get("simple_file_list").unwrap() {
            // 获取文件 Hash 集合
            let file_info_hash_set: HashSet<String> = simple_file_list.iter().map(|simple_file_info| {
                simple_file_info.file_info_hash.as_ref().unwrap().clone()
            }).collect();

            // 清理旧的文件信息
            let old_file_info_hash_set = file_info::list_key();
            old_file_info_hash_set.into_iter().for_each(|old_file_info_hash| {
                match file_info::get(&old_file_info_hash) {
                    Some(old_file_info) => {
                        if file_info_hash_set.contains(&old_file_info_hash) {
                            // 删除低音质文件
                            if !old_file_info.file_info_hash.is_empty() {
                                let audio_file_path = PathBuf::from(&app_config::AUDIO_PATH)
                                    .join(old_file_info.file_info_hash);
                                if fs::remove_file(&audio_file_path).is_err() {
                                    println!("删除低音质音频文件失败：{}", &audio_file_path.display());
                                }
                            }
                            // 删除专辑封面
                            if let Some(cover_hash) = old_file_info.cover_hash {
                                let origin_cover_path = PathBuf::from(&app_config::ORIGIN_COVER_PATH)
                                    .join(&cover_hash);
                                if fs::remove_file(&origin_cover_path).is_err() {
                                    println!("删除原始封面失败：{}", &origin_cover_path.display());
                                }

                                let small_cover_path = PathBuf::from(&app_config::SMALL_COVER_PATH)
                                    .join(&cover_hash);
                                if fs::remove_file(&small_cover_path).is_err() {
                                    println!("删除封面缩略图失败：{}", &small_cover_path.display());
                                }
                            }
                            // 删除数据库中的文件信息
                            file_info::remove(&old_file_info_hash);
                        }
                    },
                    None => {},
                }
            });
        }
        
        Ok(())
    }
}

/// 生成详细的媒体信息并存储，同时提取专辑封面
struct GenerateStorage;
impl Command for GenerateStorage {
    fn execute(&self, context: &mut HashMap<&str, ContextData>) -> Result<()> {
        if let ContextData::FileList(simple_file_list) = context.get("simple_file_list").unwrap() {
            // 生成详细的文件信息
            let file_info_list: Vec<FileInfo> = simple_file_list.par_iter()
                .map(FileInfo::from_simple)
                .collect();

            context.insert("file_info", ContextData::FileInfo(file_info_list));
        }
        Ok(())
    }
}