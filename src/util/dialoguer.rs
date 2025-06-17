use dialoguer::theme::ColorfulTheme;

pub fn dialoguer_theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

pub fn ask_yn(prompt: impl Into<String>) -> color_eyre::Result<bool> {
    let theme = dialoguer_theme();

    Ok(dialoguer::Confirm::with_theme(&theme)
        .default(true)
        .show_default(false)
        .with_prompt(prompt)
        .interact()?)
}
