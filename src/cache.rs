use std::fs;
use std::path::PathBuf;

use anyhow::Context;

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    query: String,
    answer: String,
    timestamp: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Cache {
    entries: Vec<CacheEntry>,
}

impl Cache {
    fn find<'a>(&'a self, query: &str) -> Option<&'a str> {
        self.entries.iter().find_map(|entry| {
            if entry.query == query {
                Some(entry.answer.as_str())
            } else {
                None
            }
        })
    }

    fn insert(&mut self, query: String, answer: String) {
        self.entries.retain(|entry| entry.query != query);
        let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        self.entries.push(CacheEntry {
            query,
            answer,
            timestamp,
        });
    }
}

fn cache_path(cache_file_name: &str) -> anyhow::Result<PathBuf> {
    Ok(home::home_dir()
        .context("Couldn't determine home directory")?
        .join(cache_file_name))
}

fn read_full_cache(cache_file_name: &str) -> anyhow::Result<Cache> {
    let path = cache_path(cache_file_name)?;
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Cache {
                entries: Vec::new(),
            });
        }
        Err(e) => return Err(e).context(format!("Couldn't read file {}", path.display())),
    };

    serde_json::from_str::<Cache>(&content)
        .with_context(|| format!("Couldn't parse file {}", path.display()))
}

fn write_full_cache(cache_file_name: &str, cache: &Cache) -> anyhow::Result<()> {
    let mut cache = serde_json::to_string_pretty(&cache)?;
    cache.push_str("\n");

    let path = cache_path(cache_file_name)?;

    fs::write(&path, cache)
        .with_context(|| format!("Failed to write cache to {}", path.display()))?;
    Ok(())
}

pub fn read_cache(cache_file_name: &str, query: &str) -> anyhow::Result<Option<String>> {
    let cache = read_full_cache(cache_file_name)?;
    Ok(cache.find(query).map(|s| s.to_owned()))
}

pub fn write_cache(cache_file_name: &str, query: &str, answer: &str) -> anyhow::Result<()> {
    let mut cache: Cache = read_full_cache(cache_file_name)?;
    cache.insert(query.to_owned(), answer.to_owned());
    write_full_cache(cache_file_name, &cache)?;
    Ok(())
}
