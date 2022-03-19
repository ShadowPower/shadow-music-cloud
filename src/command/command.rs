use std::collections::HashMap;
use anyhow::Result;

use crate::infra::file_utils;

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
        context.insert("file_list", ContextData::FileList(audio_file_list));
        Ok(())
    }
}