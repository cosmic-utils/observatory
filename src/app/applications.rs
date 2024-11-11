use freedesktop_entry_parser::parse_entry;

pub struct Application {
    name: String,
    cmd: String,
    icon: String,
}

impl Application {
    pub fn name(&self) -> &str { &self.name }
    pub fn cmd(&self) -> &str { &self.cmd }
    pub fn icon(&self) -> &str { &self.icon }

    pub fn scan_all() -> Vec<Application> {
        let mut apps: Vec<Application> = Vec::new();
        let files = std::fs::read_dir("/usr/share/applications").expect("Unable to read /usr/share/applications!");
        for file in files {
            let file = file.unwrap();
            let path = file.path();
            let desktop_entry = parse_entry(&path).unwrap();

            let name = if let Some(name) = desktop_entry.section("Desktop Entry").attr("Name") {
                name.to_string()
            } else {
                path.file_name().unwrap().to_str().unwrap().to_string()
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

            apps.push(Application {
                name,
                cmd,
                icon,
            });
        }

        apps
    }
}