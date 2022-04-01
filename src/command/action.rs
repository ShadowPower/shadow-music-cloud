use std::collections::HashMap;

use crate::model::dto::{SimpleFileInfo, FileInfo};

use super::command::Command;

#[derive(Debug)]
pub enum ContextData {
    String(String),
    FileList(Vec<SimpleFileInfo>),
    FileInfo(Vec<FileInfo>),
}

/// 动作
/// 包含一组命令
/// 可以并行执行不同动作
pub struct Action {
    commands: Vec<Box<dyn Command + Send + Sync>>,
}

impl Action {
    pub fn new() -> Action {
        Action {
            commands: Vec::new(),
        }
    }

    pub fn add_command(&mut self, command: Box<dyn Command + Send + Sync>) {
        self.commands.push(command);
    }

    pub fn add_commands(&mut self, commands: Vec<Box<dyn Command + Send + Sync>>) {
        self.commands.extend(commands);
    }

    pub fn execute(&self) {
        let mut action_context: HashMap<&str, ContextData> = HashMap::new();
        // 存放已经执行过的命令
        let mut executed_command_stack: Vec<&Box<dyn Command + Send + Sync>> = Vec::new();
        for command in self.commands.iter() {
            executed_command_stack.push(command);
            match command.execute(&mut action_context) {
                Err(err) => {
                    println!("Error from command: {}", err);
                    // 倒过来执行回滚操作
                    while let Some(command) = executed_command_stack.pop() {
                        command.rollback(&mut action_context);
                    }
                    break;
                },
                _ => {}
            }
        }
    }
}

/// 将一组命令包装成动作
/// 
/// 例子:
/// ```
/// action![DownloadCommand, OpenCommand, PlayCommand];
/// ```
#[macro_export]
macro_rules! action {
    ($($command:expr),*) => {
        {
            let mut action = Action::new();
            $(
                action.add_command(Box::new($command));
            )*
            Box::new(action)
        }
    };
}