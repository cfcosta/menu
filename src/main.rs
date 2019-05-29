use std::str;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::process::{Command, Stdio};
use std::io::prelude::*;

use serde::{ Serialize, Deserialize };
use serde_yaml;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
  defaults: DefaultConfig
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DefaultConfig {
  menu_cmd: String,
  terminal: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TreeItem {
  name: String,
  cmd: Option<String>,
  #[serde(default = "Vec::new")]
  items: Vec<TreeItem>
}

#[derive(Debug, PartialEq, Clone)]
pub struct TreeList;
impl TreeList {
    pub fn recursive_select(&self, config: &Config, items: Vec<TreeItem>) -> Result<String, Box<Error>> {
        let mut map: HashMap<String, TreeItem> = HashMap::new();

        for item in items.clone().iter() {
            map.insert(item.name.clone(), item.clone());
        }

        let mut command = Command::new("bash")
            .args(&["-c", &config.defaults.menu_cmd])
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        if let Some(ref mut stdin) = command.stdin {
            for (k,_) in map.iter() {
                writeln!(stdin, "{}", k)?;
            }
        }

        let stdout = command.wait_with_output()?.stdout;
        let output = String::from(str::from_utf8(&stdout)?.trim());

        let item = map.get(&output).expect("Nothing (or an invalid item) was selected.");

        match &item.cmd {
            Some(cmd) => Ok(cmd.clone()),
            None => self.recursive_select(&config, item.clone().items)
        }
    }
}

fn main() -> Result<(), Box<Error>> {
    let contents = fs::read_to_string("/home/cfcosta/.config/menu/menu.yml")?;

    let config: Config = serde_yaml::from_str(&contents)?;

    let item_files = fs::read_dir("/home/cfcosta/.config/menu/items")?;
    let items: Vec<TreeItem> = item_files
        .filter_map(|file| file.ok())
        .map(|file| file.path())
        .filter(|path| path.is_file())
        .filter_map(|path| fs::read_to_string(path).ok())
        .filter_map(|body| serde_yaml::from_str::<Vec<TreeItem>>(&body).ok())
        .flatten()
        .collect::<Vec<TreeItem>>();

    let tree = TreeList {};
    println!("{}", tree.recursive_select(&config, items)?);

    Ok(())
}
