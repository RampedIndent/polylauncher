use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const APP_NAME: &str = "Polybar-Launcher";
pub const QUIT_COMMAND: &str = "QUIT POLYBAR LAUNCHER";

#[derive(Debug, Clone, Serialize, Deserialize)] // we'll be cloning it later on
pub struct DispConfig {
    pub display: String,
    pub bar_name: String,
}
impl DispConfig {
    pub fn new(display_name: &str) -> Self {
        Self {
            display: display_name.to_string(),
            bar_name: "".to_string(),
        }
    }
}

#[derive(Clone, Debug)] // we'll be cloning it later on
#[derive(Serialize, Deserialize)] // we'll be cloning it later on
pub struct ComputerConfig {
    pub displays: Vec<DispConfig>,
}

// #[serde_as]
#[derive(Clone, Debug)] // we'll be cloning it later on
#[derive(Serialize, Deserialize)] // we'll be cloning it later on
pub struct MyConfig {
    pub config_version: u8,
    pub polybar_config: String,
    pub computers: HashMap<String, ComputerConfig>, // the hash map owns the struct
}

impl MyConfig {
    pub fn new() -> MyConfig {
        MyConfig {
            computers: HashMap::new(),
            ..Default::default()
        }
    }

    /// Join is used to add a new ComputerConfig into the hashmap
    pub fn join(
        &mut self, // must be mutable
        key: &str,
        node: ComputerConfig,
    ) {
        // do not pass a reference
        self.computers.insert(key.to_string(), node); // inserting moves `node`
    }

    pub fn change_bar(&mut self, key: &str, display_index: usize, bar_value: &str) -> Result<()> {
        let mut comp_config = self.computers[key].clone();
        comp_config.displays[display_index].bar_name = bar_value.to_string();
        self.computers.insert(key.to_string(), comp_config);
        Ok(())
    }

    pub fn add_display(&mut self, key: &str, new_display_name: &str) -> Result<()> {
        let mut comp_config = self.computers[key].clone();
        comp_config
            .displays
            .append(&mut vec![DispConfig::new(new_display_name)]);

        self.computers.insert(key.to_string(), comp_config);
        Ok(())
    }
    pub fn remove_display(&mut self, key: &str, display_name: &str) -> Result<()> {
        let mut comp_config = self.computers[key].clone();
        comp_config
            .displays
            .retain(|x| x.display == *display_name.to_string());

        self.computers.insert(key.to_string(), comp_config);
        Ok(())
    }
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self {
            config_version: 1,
            polybar_config: "~/.config/polybar/config.ini".to_string(),
            computers: HashMap::new(),
        }
    }
}

#[derive(Parser)]
#[clap(author, about, version)]
pub struct ArgsCli {
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}
