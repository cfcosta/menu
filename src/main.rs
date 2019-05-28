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

pub fn walk_node(node: TreeItem, prefix: String) -> Vec<TreeItem> {
    let name = if prefix == "" {
        node.name
    } else {
        format!("{} -> {}", prefix, node.name)
    };

    match node.cmd {
        Some(cmd) => vec![TreeItem { name: name, cmd: Some(cmd), items: vec![] }],
        None => node.items.iter().flat_map(|item| walk_node(item.clone(), name.clone()) ).collect::<Vec<TreeItem>>()
    }
}

pub fn tree_to_map(tree: Vec<TreeItem>) -> HashMap<String, TreeItem> {
    let mut res = HashMap::new();

    for item in tree.iter().flat_map(|item| walk_node(item.clone(), "".into())) {
        let new_item = item.clone();
        res.insert(item.name, new_item);
    }

    res
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
        .flat_map(|nodes|
             nodes
             .iter()
             .flat_map(|tree| walk_node(tree.clone(), String::new()))
             .collect::<Vec<TreeItem>>()
         )
        .collect();

    let tree = tree_to_map(items);

    let mut command = Command::new("bash")
      .args(&["-c", &config.defaults.menu_cmd])
      .stdin(Stdio::piped())
      .stderr(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn()?;

    if let Some(ref mut stdin) = command.stdin {
        for (k,_) in tree.iter() {
            writeln!(stdin, "{}", k)?;
        }
    }

    let stdout = command.wait_with_output()?.stdout;
    let output = String::from(str::from_utf8(&stdout)?.trim());

    let selected = tree.get(&output).unwrap();

    println!("{}", selected.cmd.clone().unwrap());

    Ok(())
}
