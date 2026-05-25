use std::process::Command;

pub struct NotiyConfig {
    pub app_name: String,
    pub summary: String,
    pub body: String,
}

impl Default for NotiyConfig {
    fn default() -> Self {
        Self { 
            app_name: "Spark".to_string(),
            summary: "%Default Summary%".to_string(),
            body: "%Default Body Text%".to_string() 
        }
    }
}

pub fn notify (nc: NotiyConfig)
{    
    let _command = Command::new("notify-send")
    .args([
        "--app-name=".to_owned() + &nc.app_name,
        nc.summary,
        nc.body
        ])
    .spawn();
}

/* pub fn notify (app_name: &str, summary: &str, body: &str)
{
    let app_name_arg = "--app-name=".to_owned() + app_name;

    let _command = Command::new("notify-send")
    .args(&[&app_name_arg, "--wait", "--urgency=critical", summary, body])
    .spawn();
} */