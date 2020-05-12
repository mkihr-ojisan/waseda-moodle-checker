use crate::*;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct LoginInfo {
    pub login_id: String,
    pub password: String,
}
impl LoginInfo {
    pub fn save(&self) -> Result<()> {
        let login_info_file = dirs::home_dir()
            .unwrap()
            .join(".waseda-moodle-checker")
            .join("login_info.json");
        let writer = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(login_info_file)?;
        serde_json::to_writer(writer, self)?;
        Ok(())
    }
    pub fn load() -> Result<LoginInfo> {
        let login_info_file = dirs::home_dir()
            .unwrap()
            .join(".waseda-moodle-checker")
            .join("login_info.json");
        if !login_info_file.exists() {
            return Err(ErrorKind::LoginRequired.into());
        }
        let reader = std::fs::OpenOptions::new()
            .read(true)
            .open(login_info_file)?;
        let info = serde_json::from_reader(reader)?;
        Ok(info)
    }
}
