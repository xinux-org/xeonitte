use anyhow::{Context, Result};
use std::collections::HashMap;

pub fn get_languages() -> HashMap<String, HashMap<String, String>> {
    let locale_codes = include_str!("../assets/locale_codes.txt")
        .trim()
        .split('\n');

    let all = gnome_desktop::functions::all_locales();

    let languages: HashMap<String, HashMap<String, String>> = locale_codes
        .filter_map(|locale| locale.split('/').nth(0))
        .filter(|locale| all.iter().any(|all_locale| all_locale.eq(locale)))
        .filter_map(|locale| {
            let language_from_locale =
                gnome_desktop::functions::language_from_locale(locale, Some(locale))?;

            let language_from_code = gnome_desktop::functions::language_from_code(
                &get_lang(locale.to_string()).ok()?,
                Some(locale),
            )?;

            Some((
                language_from_code.to_string(),
                locale.to_string(),
                language_from_locale.to_string(),
            ))
        })
        .fold(
            HashMap::new(),
            |mut hm, (language_from_code, locale, language_from_locale)| {
                hm.entry(language_from_code)
                    .or_default()
                    .insert(locale, language_from_locale);

                hm
            },
        );

    languages
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
