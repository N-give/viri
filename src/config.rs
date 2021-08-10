use std::collections::HashMap;
// use tokio::{
//     fs::read,
// };
//
// pub struct Config {
//     keys: HashMap<String, Action>,
// }
//
// impl Config {
//     pub async fn new(con_f: &str) -> Result<Config> {
//         let config = String::from_utf8(read(con_f).await?)?;
//         let mut keys = HashMap::new();
//         for line in bindings.lines() {
//             let vals = line.split("=").map(|l| l.trim()).collect();
//             keys.insert(vals[0].to_string(), vals[1].to_string());
//         }
//         Ok(Config { keys })
//     }
// }

// pub enum Action {
//     BuiltIn,
//     UserDef,
// }

// TODO actually read from config file
pub fn get_config(_con_f: &str) -> std::io::Result<HashMap<String, String>> {
    // let mut config: Config = Config::new(con_f).await?;
    let mut config: HashMap<String, String> = HashMap::new();
    config.insert("l".to_string(), "CursorRight".to_string());
    config.insert("h".to_string(), "CursorLeft".to_string());
    config.insert("w".to_string(), "CursorRightWord".to_string());
    config.insert("b".to_string(), "CursorLeftWord".to_string());
    config.insert("$".to_string(), "CursorRightAll".to_string());
    config.insert("^".to_string(), "CursorLeftAll".to_string());
    config.insert("i".to_string(), "Insert".to_string());
    config.insert("I".to_string(), "InsertStart".to_string());
    config.insert("a".to_string(), "Append".to_string());
    config.insert("A".to_string(), "AppendEnd".to_string());
    config.insert("S".to_string(), "DeleteLineInsert".to_string());
    config.insert("q".to_string(), "Quit".to_string());
    Ok(config)
}
