use std::{fs, path::PathBuf, time::Duration};

use anyhow::{Context, Result, anyhow};
use clap::{Args, Parser, Subcommand};
use reqwest::Client;
use serde_json::{Map, Value, json};

const DEFAULT_API_BASE: &str = "https://api.exa.ai";
const MCP_BASE: &str = "https://mcp.exa.ai/mcp";
const MCP_TOOLS: [&str; 9] = [
    "web_search_exa",
    "web_search_advanced_exa",
    "get_code_context_exa",
    "deep_search_exa",
    "crawling_exa",
    "company_research_exa",
    "people_search_exa",
    "deep_researcher_start",
    "deep_researcher_check",
];

#[derive(Parser)]
#[command(name = "exa", about = "Exa CLI")]
struct Cli {
    #[arg(long, global = true)]
    api_key: Option<String>,

    #[arg(long, global = true)]
    api_base: Option<String>,

    #[arg(long, global = true)]
    pretty: bool,

    #[arg(long, value_name = "SECONDS", global = true)]
    timeout: Option<u64>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Search(SearchArgs),
    Contents(ContentsArgs),
    FindSimilar(FindSimilarArgs),
    Answer(AnswerArgs),
    Context(ContextArgs),
    Research(ResearchArgs),
    Mcp(McpArgs),
}

#[derive(Args)]
struct BodyArgs {
    #[arg(long)]
    body: Option<String>,

    #[arg(long, value_name = "PATH")]
    body_file: Option<PathBuf>,
}

#[derive(Args)]
struct SearchArgs {
    #[arg(long)]
    query: Option<String>,

    #[command(flatten)]
    body: BodyArgs,
}

#[derive(Args)]
struct ContentsArgs {
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    urls: Vec<String>,

    #[arg(long, value_delimiter = ',', num_args = 1..)]
    ids: Vec<String>,

    #[command(flatten)]
    body: BodyArgs,
}

#[derive(Args)]
struct FindSimilarArgs {
    #[arg(long)]
    url: Option<String>,

    #[command(flatten)]
    body: BodyArgs,
}

#[derive(Args)]
struct AnswerArgs {
    #[arg(long)]
    query: Option<String>,

    #[command(flatten)]
    body: BodyArgs,
}

#[derive(Args)]
struct ContextArgs {
    #[arg(long)]
    query: Option<String>,

    #[command(flatten)]
    body: BodyArgs,
}

#[derive(Subcommand)]
enum ResearchCommand {
    Start(ResearchStartArgs),
    Check(ResearchCheckArgs),
}

#[derive(Args)]
struct ResearchArgs {
    #[command(subcommand)]
    command: ResearchCommand,
}

#[derive(Args)]
struct ResearchStartArgs {
    #[arg(long)]
    instructions: Option<String>,

    #[command(flatten)]
    body: BodyArgs,
}

#[derive(Args)]
struct ResearchCheckArgs {
    #[arg(long)]
    task_id: Option<String>,
}

#[derive(Subcommand)]
enum McpCommand {
    Url(McpUrlArgs),
    Tools,
}

#[derive(Args)]
struct McpArgs {
    #[command(subcommand)]
    command: McpCommand,
}

#[derive(Args)]
struct McpUrlArgs {
    #[arg(long, value_name = "LIST|all")]
    tools: Option<String>,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Command::Mcp(cmd) = &cli.command {
        return handle_mcp(&cmd.command);
    }

    let api_key = cli
        .api_key
        .or_else(|| std::env::var("EXA_API_KEY").ok())
        .context("EXA_API_KEY missing")?;
    let api_base = cli
        .api_base
        .or_else(|| std::env::var("EXA_API_BASE").ok())
        .unwrap_or_else(|| DEFAULT_API_BASE.to_string());
    let api_base = api_base.trim_end_matches('/');
    let timeout = Duration::from_secs(cli.timeout.unwrap_or(30));
    let client = Client::builder().timeout(timeout).build()?;

    let payload = match cli.command {
        Command::Search(args) => {
            let mut body = load_body(&args.body)?;
            if let Some(query) = args.query {
                body.insert("query".to_string(), Value::String(query));
            }
            ensure_string_field(&body, "query")?;
            exa_post(&client, api_base, &api_key, "/search", Value::Object(body)).await?
        }
        Command::Contents(args) => {
            let mut body = load_body(&args.body)?;
            if let Some(urls) = normalize_list(&args.urls) {
                body.insert("urls".to_string(), Value::Array(urls));
            }
            if let Some(ids) = normalize_list(&args.ids) {
                body.insert("ids".to_string(), Value::Array(ids));
            }
            ensure_any_field(&body, &["urls", "ids"])?;
            exa_post(&client, api_base, &api_key, "/contents", Value::Object(body)).await?
        }
        Command::FindSimilar(args) => {
            let mut body = load_body(&args.body)?;
            if let Some(url) = args.url {
                body.insert("url".to_string(), Value::String(url));
            }
            ensure_string_field(&body, "url")?;
            exa_post(
                &client,
                api_base,
                &api_key,
                "/findSimilar",
                Value::Object(body),
            )
            .await?
        }
        Command::Answer(args) => {
            let mut body = load_body(&args.body)?;
            if let Some(query) = args.query {
                body.insert("query".to_string(), Value::String(query));
            }
            ensure_string_field(&body, "query")?;
            body.insert("stream".to_string(), Value::Bool(false));
            exa_post(&client, api_base, &api_key, "/answer", Value::Object(body)).await?
        }
        Command::Context(args) => {
            let mut body = load_body(&args.body)?;
            if let Some(query) = args.query {
                body.insert("query".to_string(), Value::String(query));
            }
            ensure_string_field(&body, "query")?;
            exa_post(&client, api_base, &api_key, "/context", Value::Object(body)).await?
        }
        Command::Research(cmd) => match cmd.command {
            ResearchCommand::Start(args) => {
                let mut body = load_body(&args.body)?;
                if let Some(instructions) = args.instructions {
                    body.insert("instructions".to_string(), Value::String(instructions));
                }
                ensure_string_field(&body, "instructions")?;
                exa_post(
                    &client,
                    api_base,
                    &api_key,
                    "/research/v0/tasks",
                    Value::Object(body),
                )
                .await?
            }
            ResearchCommand::Check(args) => {
                let task_id = args.task_id.context("task_id missing")?;
                let path = format!("/research/v0/tasks/{task_id}");
                exa_get(&client, api_base, &api_key, &path).await?
            }
        },
        Command::Mcp(_) => unreachable!("mcp handled earlier"),
    };

    let output = if cli.pretty {
        serde_json::to_string_pretty(&payload)?
    } else {
        serde_json::to_string(&payload)?
    };
    println!("{output}");
    Ok(())
}

fn handle_mcp(cmd: &McpCommand) -> Result<()> {
    match cmd {
        McpCommand::Tools => {
            for tool in MCP_TOOLS {
                println!("{tool}");
            }
        }
        McpCommand::Url(args) => {
            let url = match args.tools.as_deref() {
                None => MCP_BASE.to_string(),
                Some("all") => format!("{MCP_BASE}?tools={}", MCP_TOOLS.join(",")),
                Some(list) => {
                    let trimmed = list.trim().trim_matches(',');
                    if trimmed.is_empty() {
                        return Err(anyhow!("tools list is empty"));
                    }
                    format!("{MCP_BASE}?tools={trimmed}")
                }
            };
            println!("{url}");
        }
    }
    Ok(())
}

fn load_body(args: &BodyArgs) -> Result<Map<String, Value>> {
    if args.body.is_some() && args.body_file.is_some() {
        return Err(anyhow!("use only one of --body or --body-file"));
    }
    let raw = match (&args.body, &args.body_file) {
        (Some(body), None) => Some(body.clone()),
        (None, Some(path)) => Some(
            fs::read_to_string(path)
                .with_context(|| format!("read body file {}", path.display()))?,
        ),
        (None, None) => None,
        _ => None,
    };
    let Some(raw) = raw else {
        return Ok(Map::new());
    };
    let value: Value = serde_json::from_str(&raw).context("parse body json")?;
    match value {
        Value::Object(map) => Ok(map),
        Value::Null => Ok(Map::new()),
        _ => Err(anyhow!("--body must be a JSON object")),
    }
}

fn normalize_list(items: &[String]) -> Option<Vec<Value>> {
    let values: Vec<Value> = items
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| Value::String(s.to_string()))
        .collect();
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

fn ensure_string_field(body: &Map<String, Value>, key: &str) -> Result<()> {
    let value = body
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .context(format!("missing {key}"))?;
    if value.is_empty() {
        return Err(anyhow!("missing {key}"));
    }
    Ok(())
}

fn ensure_any_field(body: &Map<String, Value>, keys: &[&str]) -> Result<()> {
    if keys.iter().any(|k| body.contains_key(*k)) {
        return Ok(());
    }
    Err(anyhow!("missing one of: {}", keys.join(", ")))
}

async fn exa_post(client: &Client, base: &str, key: &str, path: &str, body: Value) -> Result<Value> {
    let url = format!("{base}{path}");
    let resp = client
        .post(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header("x-api-key", key)
        .json(&body)
        .send()
        .await
        .context("exa request")?;
    parse_response(resp).await
}

async fn exa_get(client: &Client, base: &str, key: &str, path: &str) -> Result<Value> {
    let url = format!("{base}{path}");
    let resp = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .header("x-api-key", key)
        .send()
        .await
        .context("exa request")?;
    parse_response(resp).await
}

async fn parse_response(resp: reqwest::Response) -> Result<Value> {
    let status = resp.status();
    let text = resp.text().await.context("exa body")?;
    let payload = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
    if !status.is_success() {
        return Err(anyhow!("exa api failed status={} body={}", status, payload));
    }
    Ok(payload)
}
