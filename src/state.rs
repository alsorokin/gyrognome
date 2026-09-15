use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Character {
    pub document: Value,
    pub name: String,
    pub race: String,
    pub class: String,
    pub level: u64,
    pub online_realm: Option<String>,
}

impl Character {
    pub fn summary(&self) -> String {
        let online = self
            .online_realm
            .as_deref()
            .map(|realm| format!("online (realm: {realm}; passkey: [redacted])"))
            .unwrap_or_else(|| "offline".to_owned());
        format!(
            "Name: {}\nRace: {}\nClass: {}\nLevel: {}\nStatus: {online}",
            self.name, self.race, self.class, self.level
        )
    }
}
