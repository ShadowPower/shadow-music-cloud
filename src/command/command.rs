use anyhow::Result;
use radix_fmt::radix;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator, IntoParallelRefMutIterator, IntoParallelIterator};
use std::{collections::{HashMap, HashSet}, path::PathBuf};

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

/// 生成详细的文件信息
struct GenAndStorageFileInfo;
impl Command for GenAndStorageFileInfo {
    fn execute(&self, context: &mut HashMap<&str, ContextData>) -> Result<()> {
        if let ContextData::FileList(simple_file_list) = context.get("simple_file_list").unwrap() {
            // 获取文件 Hash 集合
            let file_info_hash_set: HashSet<String> = simple_file_list.iter().map(|file_info| {
                file_info.file_info_hash.as_ref().unwrap().clone()
            }).collect();

            // 清理旧的文件信息
            let old_file_info_hash_set = file_info::list_key();
            old_file_info_hash_set.into_iter().for_each(|old_file_info_hash| {
                if !file_info_hash_set.contains(&old_file_info_hash) {
                    file_info::remove(old_file_info_hash);
                }
            });

            // 生成详细的文件信息
            simple_file_list.par_iter().map(|simple_file_info| {
                todo!()
            });
        }
        
        Ok(())
    }
}