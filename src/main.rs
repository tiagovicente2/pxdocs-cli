use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_REMOTE: &str = "px-center/px-docs";
const FETCH_TTL_MS: u128 = 10 * 60 * 1000;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    docs_path: Option<String>,
    remote: Option<String>,
    last_fetch_at_by_path: Option<HashMap<String, u128>>,
}

#[derive(Clone)]
struct Doc {
    path: String,
    title: String,
    status: String,
    doc_type: String,
    guild: Option<String>,
    id: Option<String>,
    summary: Option<String>,
}

#[derive(Default)]
struct Options {
    positionals: Vec<String>,
    limit: usize,
    remote: bool,
    refresh: bool,
    no_fetch: bool,
    guild: Option<String>,
    status: Option<String>,
    doc_type: Option<String>,
}

struct Source {
    docs: Vec<Doc>,
    remote: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("pxdocs: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().cloned().unwrap_or_else(|| "help".to_string());
    if !args.is_empty() {
        args.remove(0);
    }

    match command.as_str() {
        "help" | "--help" | "-h" => return Ok(print_help()),
        "setup" => return setup(args.first().map(String::as_str)),
        "config" => return print_config(),
        "doctor" => return doctor(),
        _ => {}
    }

    let options = parse_options(args)?;
    let source = load_source(&options)?;

    match command.as_str() {
        "decisions" => {
            let docs = filter_docs(
                &source.docs,
                Some("decision"),
                options.guild.as_deref(),
                options.status.as_deref(),
            );
            print_docs(docs.into_iter().take(options.limit));
        }
        "search" => {
            let query = options.positionals.join(" ");
            if query.is_empty() {
                return Err("search requires a query".to_string());
            }
            let docs = search_docs(&source.docs, &query);
            print_docs(docs.into_iter().take(options.limit));
        }
        "show" => {
            let selector = options.positionals.join(" ");
            if selector.is_empty() {
                return Err("show requires a path or id".to_string());
            }
            let matches = find_docs(
                &source.docs,
                &selector,
                options.doc_type.as_deref(),
                options.guild.as_deref(),
                options.status.as_deref(),
            );
            if matches.is_empty() {
                return Err(format!("doc not found: {selector}"));
            }
            if matches.len() > 1 {
                print_docs(matches.iter().take(options.limit).cloned());
                return Err(format!(
                    "ambiguous selector: {selector}. narrow with --guild or use a path"
                ));
            }
            let content = if source.remote {
                read_remote_doc(&remote(), &matches[0].path)?
            } else {
                read_local_doc(
                    &docs_path().ok_or("px-docs path is not configured")?,
                    &matches[0].path,
                )?
            };
            print!("{content}");
        }
        _ => return Err(format!("unknown command: {command}")),
    }

    maybe_fetch_after_command(&source, &options);
    Ok(())
}

fn setup(path_arg: Option<&str>) -> Result<(), String> {
    let docs_path = ask_for_docs_path(path_arg, false)?.ok_or("px-docs path is required")?;
    save_docs_path(&docs_path)?;
    println!("configured px-docs path: {}", docs_path.display());
    Ok(())
}

fn prompt_for_docs_path() -> Result<Option<PathBuf>, String> {
    eprintln!("px-docs path is not configured.");
    let docs_path = ask_for_docs_path(None, true)?;
    if let Some(path) = &docs_path {
        save_docs_path(path)?;
        eprintln!("configured px-docs path: {}", path.display());
    } else {
        eprintln!("using GitHub fallback through gh");
    }
    Ok(docs_path)
}

fn ask_for_docs_path(path_arg: Option<&str>, allow_empty: bool) -> Result<Option<PathBuf>, String> {
    let raw = match path_arg {
        Some(value) => value.to_string(),
        None => {
            let suffix = if allow_empty {
                ", or press enter to use GitHub fallback"
            } else {
                ""
            };
            print!("px-docs local path{suffix}: ");
            io::stdout().flush().map_err(|error| error.to_string())?;
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .map_err(|error| error.to_string())?;
            input
        }
    };

    let raw = raw.trim();
    if raw.is_empty() && allow_empty {
        return Ok(None);
    }

    let path = expand_path(raw);
    if !path.exists() {
        return Err(format!("path does not exist: {}", path.display()));
    }
    if !path.join("docs").exists() {
        return Err(format!(
            "path does not look like px-docs: {}",
            path.display()
        ));
    }
    Ok(Some(path))
}

fn load_source(options: &Options) -> Result<Source, String> {
    let mut local_path = docs_path();
    if !options.remote && local_path.is_none() {
        local_path = prompt_for_docs_path()?;
    }

    if !options.remote {
        if let Some(path) = local_path {
            warn_if_known_behind(&path);
            return Ok(Source {
                docs: load_local_docs(&path)?,
                remote: false,
            });
        }
    }

    assert_gh_available()?;
    Ok(Source {
        docs: load_remote_docs(&remote())?,
        remote: true,
    })
}

fn maybe_fetch_after_command(source: &Source, options: &Options) {
    if source.remote || options.no_fetch {
        return;
    }
    let Some(path) = docs_path() else {
        return;
    };
    if !options.refresh && !should_fetch(&path) {
        return;
    }
    let _ = Command::new("git")
        .arg("-C")
        .arg(&path)
        .args(["fetch", "--quiet"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let _ = mark_docs_fetched(&path);
}

fn warn_if_known_behind(path: &Path) {
    if let Ok(status) = repo_status(path, false) {
        if status.behind > 0 {
            eprintln!(
                "warning: px-docs is {} commit(s) behind {}",
                status.behind, status.upstream
            );
        }
    }
}

struct RepoStatus {
    upstream: String,
    behind: u32,
    ahead: u32,
}

fn repo_status(path: &Path, fetch: bool) -> Result<RepoStatus, String> {
    let upstream = git_output(
        path,
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
    )?;
    if fetch {
        let _ = Command::new("git")
            .current_dir(path)
            .args(["fetch", "--quiet"])
            .status();
    }
    let counts = git_output(
        path,
        &[
            "rev-list",
            "--left-right",
            "--count",
            &format!("{upstream}...HEAD"),
        ],
    )?;
    let mut parts = counts.split_whitespace();
    let behind = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let ahead = parts.next().unwrap_or("0").parse().unwrap_or(0);
    Ok(RepoStatus {
        upstream,
        behind,
        ahead,
    })
}

fn git_output(path: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn load_local_docs(root: &Path) -> Result<Vec<Doc>, String> {
    let mut files = Vec::new();
    walk_markdown(&root.join("docs"), &mut files)?;
    files.sort();
    files
        .into_iter()
        .map(|file| {
            let content = fs::read_to_string(&file).map_err(|error| error.to_string())?;
            Ok(parse_doc(&file, Some(&content), Some(root)))
        })
        .collect()
}

fn walk_markdown(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            walk_markdown(&path, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            files.push(path);
        }
    }
    Ok(())
}

fn parse_doc(file: &Path, content: Option<&str>, root: Option<&Path>) -> Doc {
    let relative = match root {
        Some(root) => file.strip_prefix(root).unwrap_or(file).to_path_buf(),
        None => file.to_path_buf(),
    };
    let normalized = relative.to_string_lossy().replace('\\', "/");
    let base = Path::new(&normalized)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(&normalized);
    let title = content
        .and_then(|text| {
            text.lines()
                .find_map(|line| line.strip_prefix("# ").map(str::trim).map(str::to_string))
        })
        .unwrap_or_else(|| title_from_basename(base));
    let status = content
        .and_then(|text| find_field(text, "Status"))
        .unwrap_or_else(|| "unknown".to_string());
    let summary = content.and_then(|text| {
        text.lines()
            .find(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.contains(':')
            })
            .map(|line| line.trim().to_string())
    });
    let doc_type = doc_type(&normalized).to_string();
    let guild = guild(&normalized);
    let id = id_from_basename(base);

    Doc {
        path: normalized,
        title,
        status,
        doc_type,
        guild,
        id,
        summary,
    }
}

fn find_field(content: &str, field: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key.trim().eq_ignore_ascii_case(field) {
            Some(value.trim().to_string())
        } else {
            None
        }
    })
}

fn title_from_basename(base: &str) -> String {
    let mut rest = base;
    for prefix in ["adr-", "rfc-"] {
        if let Some(stripped) = rest.strip_prefix(prefix) {
            rest = stripped;
        }
    }
    let rest = rest.trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == '-');
    if rest.is_empty() {
        base.to_string()
    } else {
        rest.replace('-', " ")
    }
}

fn id_from_basename(base: &str) -> Option<String> {
    let mut rest = base;
    for prefix in ["adr-", "rfc-"] {
        if let Some(stripped) = rest.strip_prefix(prefix) {
            rest = stripped;
        }
    }
    let id: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    if id.is_empty() { None } else { Some(id) }
}

fn doc_type(path: &str) -> &str {
    if path.contains("/decisions/") {
        "decision"
    } else if path.contains("/ADRs/") {
        "adr"
    } else if path.contains("/RFCs/") || path.contains("/rfcs/") {
        "rfc"
    } else if path.contains("/guides/") {
        "guide"
    } else {
        "doc"
    }
}

fn guild(path: &str) -> Option<String> {
    for guild in ["front", "back", "qa"] {
        if path.starts_with(&format!("docs/{guild}-guild/")) {
            return Some(guild.to_string());
        }
    }
    None
}

fn read_local_doc(root: &Path, doc_path: &str) -> Result<String, String> {
    fs::read_to_string(root.join(doc_path)).map_err(|error| error.to_string())
}

fn filter_docs<'a>(
    docs: &'a [Doc],
    doc_type: Option<&str>,
    guild: Option<&str>,
    status: Option<&str>,
) -> Vec<Doc> {
    docs.iter()
        .filter(|doc| {
            doc_type.is_none_or(|value| doc.doc_type == value)
                && guild.is_none_or(|value| doc.guild.as_deref() == Some(value))
                && status
                    .is_none_or(|value| doc.status.to_lowercase().contains(&value.to_lowercase()))
        })
        .cloned()
        .collect()
}

fn search_docs(docs: &[Doc], query: &str) -> Vec<Doc> {
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|term| term.to_lowercase())
        .collect();
    let mut scored: Vec<(usize, Doc)> = docs
        .iter()
        .filter_map(|doc| {
            let haystack = format!(
                "{} {} {} {}",
                doc.title,
                doc.path,
                doc.status,
                doc.summary.as_deref().unwrap_or("")
            )
            .to_lowercase();
            let score = terms
                .iter()
                .filter(|term| haystack.contains(term.as_str()))
                .count();
            (score > 0).then(|| (score, doc.clone()))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.path.cmp(&b.1.path)));
    scored.into_iter().map(|(_, doc)| doc).collect()
}

fn find_docs(
    docs: &[Doc],
    selector: &str,
    doc_type: Option<&str>,
    guild: Option<&str>,
    status: Option<&str>,
) -> Vec<Doc> {
    let scoped = filter_docs(docs, doc_type, guild, status);
    let mut matches = Vec::new();
    for predicate in [0, 1, 2] {
        for doc in &scoped {
            let is_match = match predicate {
                0 => doc.path == selector,
                1 => doc.id.as_deref() == Some(selector),
                _ => doc.path.contains(selector),
            };
            if is_match
                && !matches
                    .iter()
                    .any(|candidate: &Doc| candidate.path == doc.path)
            {
                matches.push(doc.clone());
            }
        }
    }
    matches
}

fn print_docs<I>(docs: I)
where
    I: IntoIterator<Item = Doc>,
{
    for doc in docs {
        let mut suffix = vec![doc.doc_type];
        if let Some(guild) = doc.guild {
            suffix.push(guild);
        }
        suffix.push(doc.status);
        println!("{}\n  {} ({})", doc.path, doc.title, suffix.join(" · "));
    }
}

fn load_remote_docs(remote: &str) -> Result<Vec<Doc>, String> {
    let output = gh(&[
        "api",
        &format!("repos/{remote}/git/trees/HEAD?recursive=1"),
        "--jq",
        r#".tree[] | select(.type=="blob" and (.path|startswith("docs/")) and (.path|endswith(".md"))) | .path"#,
    ])?;
    let mut docs: Vec<Doc> = output
        .lines()
        .map(|line| parse_doc(Path::new(line), None, None))
        .collect();
    docs.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(docs)
}

fn read_remote_doc(remote: &str, path: &str) -> Result<String, String> {
    let content = gh(&[
        "api",
        &format!("repos/{remote}/contents/{path}"),
        "--jq",
        ".content",
    ])?;
    let output = Command::new("base64")
        .arg("-d")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(content.replace('\n', "").as_bytes())?;
            child.wait_with_output()
        })
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err("failed to decode GitHub content".to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn assert_gh_available() -> Result<(), String> {
    gh(&["--version"]).map(|_| ())
}

fn gh(args: &[&str]) -> Result<String, String> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn print_config() -> Result<(), String> {
    println!("config: {}", config_file().display());
    println!(
        "{}",
        serde_json::to_string_pretty(&read_config()).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn doctor() -> Result<(), String> {
    let Some(path) = docs_path() else {
        println!("px-docs path: not configured");
        println!("remote fallback: available through gh");
        return Ok(());
    };
    println!("px-docs path: {}", path.display());
    let status = repo_status(&path, true)?;
    mark_docs_fetched(&path)?;
    println!("upstream: {}", status.upstream);
    println!("behind: {}", status.behind);
    println!("ahead: {}", status.ahead);
    Ok(())
}

fn parse_options(args: Vec<String>) -> Result<Options, String> {
    let mut options = Options {
        limit: 20,
        ..Options::default()
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--remote" => options.remote = true,
            "--refresh" => options.refresh = true,
            "--no-fetch" => options.no_fetch = true,
            "--guild" => {
                index += 1;
                options.guild = args.get(index).cloned();
            }
            "--status" => {
                index += 1;
                options.status = args.get(index).cloned();
            }
            "--type" => {
                index += 1;
                options.doc_type = args.get(index).cloned();
            }
            "--limit" => {
                index += 1;
                options.limit = args
                    .get(index)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(20);
            }
            value => options.positionals.push(value.to_string()),
        }
        index += 1;
    }
    Ok(options)
}

fn config_file() -> PathBuf {
    home_dir()
        .join(".config")
        .join("pxdocs")
        .join("config.json")
}

fn read_config() -> Config {
    fs::read_to_string(config_file())
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn write_config(config: &Config) -> Result<(), String> {
    let file = config_file();
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
    fs::write(file, format!("{content}\n")).map_err(|error| error.to_string())
}

fn save_docs_path(path: &Path) -> Result<(), String> {
    let mut config = read_config();
    config.docs_path = Some(path.to_string_lossy().to_string());
    config.remote = Some(DEFAULT_REMOTE.to_string());
    write_config(&config)
}

fn docs_path() -> Option<PathBuf> {
    read_config().docs_path.map(PathBuf::from)
}

fn remote() -> String {
    read_config()
        .remote
        .unwrap_or_else(|| DEFAULT_REMOTE.to_string())
}

fn should_fetch(path: &Path) -> bool {
    let config = read_config();
    let Some(map) = config.last_fetch_at_by_path else {
        return true;
    };
    let Some(last) = map.get(&path.to_string_lossy().to_string()) else {
        return true;
    };
    now_ms().saturating_sub(*last) > FETCH_TTL_MS
}

fn mark_docs_fetched(path: &Path) -> Result<(), String> {
    let mut config = read_config();
    let mut map = config.last_fetch_at_by_path.unwrap_or_default();
    map.insert(path.to_string_lossy().to_string(), now_ms());
    config.last_fetch_at_by_path = Some(map);
    write_config(&config)
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn expand_path(value: &str) -> PathBuf {
    if value == "~" {
        return home_dir();
    }
    if let Some(rest) = value.strip_prefix("~/") {
        return home_dir().join(rest);
    }
    PathBuf::from(value)
}

fn home_dir() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_string()))
}

fn print_help() {
    println!(
        "pxdocs - discover PX docs from local files or GitHub\n\nUsage:\n  pxdocs setup [path]             configure local px-docs path\n  pxdocs doctor                   check config and whether repo is behind remote\n  pxdocs decisions [options]      list decision docs\n  pxdocs search <query> [options] search docs metadata\n  pxdocs show <path|id> [options] print a doc\n  pxdocs config                   print config\n\nOptions:\n  --guild <front|back|qa>         filter guild docs\n  --status <status>               filter by status text\n  --type <decision|adr|rfc|guide> filter doc type for show\n  --limit <number>                max results, default 20\n  --remote                        use gh CLI instead of local files\n  --refresh                       fetch after local command, ignoring TTL\n  --no-fetch                      skip fetch after local command\n\nFreshness:\n  Local commands warn from known remote state before output and fetch after output at most once every 10 minutes.\n"
    );
}
