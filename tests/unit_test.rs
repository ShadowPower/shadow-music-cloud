use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use radix_fmt::radix;
use rayon::prelude::*;

use shadow_music_cloud::{model::dto::FileInfo, command::actor::act, action};
use shadow_music_cloud::repository::file_info;
use shadow_music_cloud::{
    command::{
        action::{Action, ContextData},
        command::Command,
    },
    config::app_config,
    infra::{file_utils, hash_utils},
};

struct WriteValueCommand;
impl Command for WriteValueCommand {
    fn execute(&self, context: &mut HashMap<&str, ContextData>) -> Result<()> {
        println!("execute TestCommand");
        std::thread::sleep(std::time::Duration::from_millis(1000));
        context.insert(
            "data",
            ContextData::String("string from another command".to_string()),
        );
        Ok(())
    }
}

struct ReadValueCommand;
impl Command for ReadValueCommand {
    fn execute(&self, context: &mut HashMap<&str, ContextData>) -> Result<()> {
        match context.get("data") {
            Some(ContextData::String(s)) => println!("{}", s),
            _ => println!("no value"),
        }
        Ok(())
    }
}

#[test]
fn test_action() {
    let test_action = action![WriteValueCommand, ReadValueCommand];
    act(test_action);
    std::thread::sleep(std::time::Duration::from_millis(1000));
}

#[test]
fn test_file_hash() {
    let audio_file_info_list = file_utils::list_audio_file();
    for audio_file_info in audio_file_info_list {
        let hash = hash_utils::hash_media_file_info(&audio_file_info);
        println!("{}", base62::encode(hash));
        println!("{}", radix(hash, 36));
    }
}

#[test]
fn test_audio_hash() -> Result<()> {
    let audio_file_info_list = file_utils::list_audio_file();
    audio_file_info_list.par_iter().for_each(|audio_file_info| {
        let mut path = PathBuf::new();
        path.push(Path::new(app_config::AUDIO_PATH));
        path.push(audio_file_info.path.clone());
        println!("{}", audio_file_info.path.display());
        match hash_utils::hash_audio_data(&path) {
            Ok(hash) => println!("{}", base62::encode(hash)),
            Err(e) => println!("{}", e),
        }
    });
    Ok(())
}

#[test]
fn test_storage() {
    let test_data = FileInfo {
        path: ["test", "test2"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        file_type: "audio".to_string(),
        size: 1000,
        last_modified: 2000,
        file_info_hash: "TestData".to_string(),
        cue_media_path: None,
        cue_media_file_info_hash: None,
        cover_hash: "TestData".to_string(),
        medias: vec![],
    };

    file_info::set("TestData".to_string(), &test_data);
    let data_from_storage = file_info::get("TestData".to_string()).unwrap();
    println!("{:?}", data_from_storage);
}
