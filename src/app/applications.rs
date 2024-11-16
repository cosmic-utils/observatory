use std::cmp::Ordering;
use std::collections::HashMap;
use freedesktop_entry_parser::parse_entry;

#[derive(Debug, Clone)]
pub struct Application {
    _name: String,
    cmd: String,
    icon: String,
}

impl Application {
    pub fn _name(&self) -> &str { &self._name }
    pub fn cmd(&self) -> &str { &self.cmd }
    pub fn icon(&self) -> &str { &self.icon }

    pub fn scan_all() -> Vec<Application> {
        let mut apps: HashMap<String, Application> = HashMap::new();
        let files = std::fs::read_dir("/usr/share/applications").expect("Unable to read /usr/share/applications!");
        for file in files {
            let file = file.unwrap();
            let path = file.path();
            if let Ok(desktop_entry) = parse_entry(&path) {
                let name = if let Some(name) = desktop_entry.section("Desktop Entry").attr("Name") {
                    name.to_string()
                } else {
                    path.file_name().expect("Could not read file name of application!").to_str().unwrap().to_string()
                };

                // Used for clearing multiple cmd situations
                let full_cmd = if let Some(cmd) = desktop_entry.section("Desktop Entry").attr("Exec") {
                    cmd.to_string()
                } else {
                    "".to_string()
                };

                let cmd = if let Some(cmd) = desktop_entry.section("Desktop Entry").attr("Exec") {
                    cmd.to_string().split(" ").nth(0).unwrap().to_string()
                } else {
                    "".to_string()
                };

                let icon = if let Some(icon) = desktop_entry.section("Desktop Entry").attr("Icon") {
                    icon.to_string()
                } else {
                    "application-default-symbolic".to_string()
                };

                if full_cmd.cmp(&cmd) == Ordering::Equal || !apps.contains_key(&cmd) {
                    apps.insert(cmd.clone(), Application { _name: name, cmd, icon });
                }
            }
        }

        apps.values().cloned().collect()
    }
}