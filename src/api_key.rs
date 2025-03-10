use std::env;

pub(crate) fn xai_api_key() -> anyhow::Result<String> {
    match env::var_os("XAI_API_KEY") {
        None => Err(anyhow::anyhow!("XAI_API_KEY is not set")),
        Some(key) => {
            let key = key
                .into_string()
                .map_err(|s| anyhow::anyhow!("XAI_API_KEY not UTF-8: {s:?}"))?;
            if key.is_empty() {
                return Err(anyhow::anyhow!("XAI_API_KEY env var is set but empty"));
            }
            Ok(key)
        }
    }
}
