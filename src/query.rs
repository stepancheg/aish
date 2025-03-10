// https://docs.x.ai/docs/api-reference#chat-completions

use std::process::Command;
use std::process::Stdio;

use anyhow::Context;

use crate::api_key::xai_api_key;

#[derive(serde::Serialize)]
struct QueryJsonMessage {
    role: &'static str,
    content: String,
}

#[derive(serde::Serialize)]
struct QueryJson {
    model: &'static str,
    messages: Vec<QueryJsonMessage>,
}

#[derive(serde::Deserialize)]
struct ResponseChoiceMessage {
    content: String,
}

#[derive(serde::Deserialize)]
struct ResponseChoice {
    message: ResponseChoiceMessage,
}

#[derive(serde::Deserialize)]
struct ResponseJson {
    choices: Vec<ResponseChoice>,
}

pub fn query(prompt: &str, query: &str) -> anyhow::Result<String> {
    let query_json = QueryJson {
        model: "grok-2-latest",
        messages: vec![
            QueryJsonMessage {
                role: "system",
                content: prompt.to_owned(),
            },
            QueryJsonMessage {
                role: "user",
                content: query.to_owned(),
            },
        ],
    };
    let query_json = serde_json::to_string(&query_json)?;
    let mut command = Command::new("curl");
    let xai_api_key = xai_api_key()?;
    command.args([
        "-s",
        "-H",
        &format!("Authorization: Bearer {xai_api_key}"),
        "-H",
        "Content-Type: application/json",
        "-d",
        query_json.as_str(),
        "https://api.x.ai/v1/chat/completions",
    ]);
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::inherit());
    let output = command.output().context("Failed to spawn curl")?;
    let status = output.status;
    if !status.success() {
        return Err(anyhow::anyhow!("curl failed: {status}"));
    }

    let response_json: ResponseJson =
        serde_json::from_slice(&output.stdout).context("Failed to parse response JSON")?;

    let answer = response_json
        .choices
        .iter()
        .map(|c| c.message.content.as_str())
        .collect::<Vec<_>>()
        .concat();
    if answer.is_empty() {
        return Err(anyhow::anyhow!("Empty answer"));
    }
    Ok(answer)
}
