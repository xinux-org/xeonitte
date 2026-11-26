use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct LocaleUtf {
    pub code: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct Locale {
    pub name: String,
    pub variants: Vec<LocaleUtf>,
}

pub fn get_languages() -> Vec<Locale> {
    let locale_codes = include_str!("../assets/locale_codes.txt")
        .trim()
        .split('\n');

    // get all utf codes of locales available in gnome
    let gnome_utf_codes = gnome_desktop::functions::all_locales();

    locale_codes
        .filter_map(|locale| locale.split('/').next())
        .filter(|locale| {
            gnome_utf_codes
                .iter()
                .any(|all_locale| all_locale.eq(locale))
        })
        .filter_map(|locale| {
            let language_from_locale =
                gnome_desktop::functions::language_from_locale(locale, Some(locale))?;

            let language_from_code = gnome_desktop::functions::language_from_code(
                &get_lang(locale.to_string()).ok()?,
                Some(locale),
            )?;

            Locale {
                name: language_from_code.into(),
                variants: vec![LocaleUtf {
                    code: locale.into(),
                    title: language_from_locale.into(),
                }],
            }
            .into()
        })
        .collect()
}

pub fn get_lang(code: String) -> Result<String> {
    Ok(code
        .split('_')
        .next()
        .context("Invalid country")?
        .split('.')
        .next()
        .context("Invalid country")?
        .split('/')
        .next()
        .context("Invalid country")?
        .split('@')
        .next()
        .context("Invalid country")?
        .to_string())
}

pub fn get_country(code: String) -> Result<String> {
    Ok(code
        .split('_')
        .nth(1)
        .context("Invalid country")?
        .split('.')
        .next()
        .context("Invalid country")?
        .split('/')
        .next()
        .context("Invalid country")?
        .split('@')
        .next()
        .context("Invalid country")?
        .to_string())
}
