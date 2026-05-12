/// Config command implementation
use crate::config;
use crate::errors::ThoughtError;
use std::path::Path;

/// Execute the config command
pub fn execute(data_dir: &Path, key: String, value: Option<String>) -> Result<(), ThoughtError> {
    if let Some(value) = value {
        let mut cfg = config::load_config(data_dir)?;
        cfg.set_value(&key, &value)?;
        config::save_config(data_dir, &cfg)?;
    } else {
        let cfg = config::load_config(data_dir)?;
        let current = cfg.get_value(&key)?;
        println!("{current}");
    }
    Ok(())
}
