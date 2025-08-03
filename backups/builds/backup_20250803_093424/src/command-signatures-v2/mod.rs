//! Command Signatures V2 Module
//! 
//! Enhanced command signature detection and analysis system
//! with improved parsing, validation, and completion features.

pub mod parser;
pub mod validator;
pub mod completion;
pub mod analyzer;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSignature {
    pub name: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
    pub examples: Vec<String>,
    pub flags: Vec<Flag>,
    pub subcommands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub description: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Path,
    Url,
    Email,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flag {
    pub short: Option<char>,
    pub long: String,
    pub description: String,
    pub takes_value: bool,
    pub multiple: bool,
}

pub struct CommandSignatureManager {
    signatures: HashMap<String, CommandSignature>,
    custom_parsers: HashMap<String, Box<dyn Fn(&str) -> Option<CommandSignature>>>,
}

impl CommandSignatureManager {
    pub fn new() -> Self {
        Self {
            signatures: HashMap::new(),
            custom_parsers: HashMap::new(),
        }
    }

    pub fn register_signature(&mut self, signature: CommandSignature) {
        self.signatures.insert(signature.name.clone(), signature);
    }

    pub fn get_signature(&self, command: &str) -> Option<&CommandSignature> {
        self.signatures.get(command)
    }

    pub fn parse_command(&self, command_line: &str) -> Option<ParsedCommand> {
        // Basic parsing implementation
        let parts: Vec<&str> = command_line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let command_name = parts[0];
        if let Some(signature) = self.get_signature(command_name) {
            Some(ParsedCommand {
                name: command_name.to_string(),
                args: parts[1..].iter().map(|s| s.to_string()).collect(),
                signature: signature.clone(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub name: String,
    pub args: Vec<String>,
    pub signature: CommandSignature,
}

impl Default for CommandSignatureManager {
    fn default() -> Self {
        Self::new()
    }
}
