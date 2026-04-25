pub static VERSION: &str = "0.1.5";
pub static GETTEXT_PACKAGE: &str = "spark-serial";
pub static LOCALEDIR: &str = "/app/share/locale";
pub static PKGDATADIR: &str = "/app/share/spark";
pub static DEV_MODE: bool = true;

#[derive(PartialEq, Eq)]
pub enum ColorTheme {
    Accent,
    Sparking,
}
pub static COLOR_THEME: ColorTheme = ColorTheme::Accent;
