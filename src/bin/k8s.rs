use std::process;
use std::process::Command;
use std::process::Stdio;

use aish::cache::read_cache;
use aish::cache::write_cache;
use clap::Parser;
use indoc::indoc;

/// `kubectl` wrapper: sends query to xAI API, evaluates the result.
#[derive(clap::Parser)]
struct K8sArgs {
    /// Print response, but do not execute.
    #[clap(long, short = 'n')]
    no_run: bool,

    /// Do not use cache.
    ///
    /// Query results are cached by default,
    /// this option can be used if cached query result is not good enough,
    /// give a model another try.
    #[clap(long, short = 'f')]
    overwrite: bool,

    /// Arbitrary used after prompt, e.g. `pods in namespace foobar`.
    #[clap(required = true, trailing_var_arg = true)]
    query: Vec<String>,
}

fn do_query(query: &str) -> anyhow::Result<String> {
    let prompt = indoc! {
        r#"
            You are assisting with working with kubernetes on Unix.
            The response is shell command (e.g. kubectl, grep etc),
            no comments, no explanations, command only.
            Ssh directly won't work.
            For ssh shell prefer bash.
            Never change current context, never change current namespace unless a user
            explicitly asked to (avoid `kubectl config use-context` if possible).
            Pipes, && and such are allowed: result will be fed into `sh -c '...'`.
        "#
    };

    aish::query::query(prompt, &query)
}

fn main() -> anyhow::Result<()> {
    let args = K8sArgs::parse();

    let K8sArgs {
        no_run,
        overwrite,
        query,
    } = args;

    let cache_file_name = ".k8s-cache.json";

    let query = query.join(" ");

    let answer = if overwrite {
        let answer = do_query(&query)?;
        write_cache(cache_file_name, &query, &answer)?;
        answer
    } else {
        match read_cache(cache_file_name, &query)? {
            None => {
                let answer = do_query(&query)?;
                write_cache(cache_file_name, &query, &answer)?;
                answer
            }
            Some(answer) => answer,
        }
    };

    if no_run {
        eprintln!("{answer}");
        return Ok(());
    }

    eprintln!("Invoking: {answer}");
    let status = Command::new("sh")
        .arg("-c")
        .arg(&answer)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
