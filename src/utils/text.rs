use std::fs;
use std::path::Path;
use api::apis::Error as ApiError;
use crate::error::{Result, TrackItError};

pub fn decode_cli_escapes(value: &str) -> String {
    let mut decoded = String::with_capacity(value.len());
    let mut chars = value.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            decoded.push(ch);
            continue;
        }

        match chars.next() {
            Some('n') => decoded.push('\n'),
            Some('r') => decoded.push('\r'),
            Some('t') => decoded.push('\t'),
            Some('\\') => decoded.push('\\'),
            Some('"') => decoded.push('"'),
            Some(other) => {
                decoded.push('\\');
                decoded.push(other);
            }
            None => decoded.push('\\'),
        }
    }

    decoded
}

pub fn read_text_file(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

#[cfg(test)]
mod tests {
    use super::decode_cli_escapes;

    #[test]
    fn decodes_common_escapes() {
        let input = "a\\nb\\tc\\r\\\\\\\"";
        let expected = "a\nb\tc\r\\\"";
        assert_eq!(decode_cli_escapes(input), expected);
    }

    #[test]
    fn leaves_unknown_escapes_untouched() {
        let input = "value\\x";
        assert_eq!(decode_cli_escapes(input), "value\\x");
    }

    #[test]
    fn keeps_trailing_backslash() {
        let input = "value\\";
        assert_eq!(decode_cli_escapes(input), "value\\");
    }
}

pub(crate) fn normalize_base_path(url: &str) -> Result<String> {
    let trimmed = url.trim().trim_end_matches('/');

    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err(TrackItError::InvalidUrl(
            "URL must start with http:// or https://".to_string(),
        ));
    }

    if trimmed.ends_with("/api") {
        return Ok(trimmed.to_string());
    }

    Ok(format!("{trimmed}/api"))
}

pub(crate) fn map_api_error<T: std::fmt::Debug>(error: ApiError<T>) -> TrackItError {
    match error {
        ApiError::ResponseError(content) => TrackItError::ApiResponse {
            status: content.status.as_u16(),
            body: content.content,
        },
        other => TrackItError::ApiMessage(other.to_string()),
    }
}

pub(crate) fn quote_command_part(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.chars().any(|c| c.is_whitespace() || c == '"') {
        format!("\"{}\"", trimmed.replace('"', "\\\""))
    } else {
        trimmed.to_string()
    }
}