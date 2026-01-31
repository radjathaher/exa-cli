# exa-cli

Thin CLI for Exa Search API + MCP URL helper.

## Install

### Install script (macOS arm64 + Linux x86_64)

```bash
curl -fsSL https://raw.githubusercontent.com/radjathaher/exa-cli/main/scripts/install.sh | bash
```

### Build from source

```bash
cargo build --release
./target/release/exa --help
```

## Auth

```bash
export EXA_API_KEY="exa_..."
```

Optional override:

```bash
export EXA_API_BASE="https://api.exa.ai"
```

## Usage

```bash
exa search --query "agentic workflows" --pretty
exa contents --urls https://example.com --pretty
exa find-similar --url https://example.com --pretty
exa answer --query "Summarize this topic" --pretty
exa context --query "RAG prompt" --pretty
exa research start --instructions "Deep research on robotics startups"
exa research check --task-id "task_123"
```

Raw body overrides (merge with flags):

```bash
exa search --query "robots" --body '{"numResults":5}' --pretty
exa search --body-file ./payload.json --pretty
```

MCP helpers:

```bash
exa mcp tools
exa mcp url
exa mcp url --tools all
```
