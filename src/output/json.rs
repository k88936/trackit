use serde::Serialize;

pub fn format_json<T: Serialize + ?Sized>(data: &T) -> crate::error::Result<String> {
    Ok(serde_json::to_string_pretty(data)?)
}
