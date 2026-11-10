//! Laravel's `Str::slug()`.

pub fn slug(value: &str) -> String {
    let transliterated = deunicode::deunicode(value);
    let mut result = String::with_capacity(transliterated.len());
    let mut previous_dash = false;
    for character in transliterated.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash && !result.is_empty() {
            result.push('-');
            previous_dash = true;
        }
    }
    while result.ends_with('-') {
        result.pop();
    }
    result
}

/// Alias used by the controllers (Laravel's `Str::slug()`).
pub fn slugify(value: &str) -> String {
    slug(value)
}
