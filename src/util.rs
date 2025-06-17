mod spin;
pub use spin::*;

pub fn dialoguer_theme() -> dialoguer::theme::ColorfulTheme {
    dialoguer::theme::ColorfulTheme::default()
}
