//! `create-rust-app` — Rust-native scaffolding CLI.
//!
//! Project scaffolding commands mirror the sibling CLIs (`create-vlang-app`,
//! `create-python-app`, `create-node-app`): positional project name,
//! `--template` / `--addons` selection from the `cra-templates` catalog,
//! `--list-templates` / `--list-addons` browsing, and a `cache` subcommand.

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use create_rust_app_core::{
    brand_banner, cache_dir, clean_cache, env_info, format_catalog_list, list_addon_names,
    list_template_names, list_template_names_in_category, load_catalog, parse_set_override,
    resolve_fixture_catalog_path, scaffold, validate_project_name, ScaffoldOptions,
};
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(
    name = "create-rust-app",
    version,
    about = "Scaffold Rust projects from templates and extensions\nExamples:\n  create-rust-app my-app --template web-server --addons github-setup --no-interactive"
)]
struct Cli {
    /// Project directory to create.
    project: Option<String>,

    /// Print environment info.
    #[arg(short, long)]
    info: bool,

    /// Verbose output.
    #[arg(short, long)]
    verbose: bool,

    /// Template URL, file://, or slug.
    #[arg(short, long, default_value = "")]
    template: String,

    /// Comma-separated addon slugs or URLs.
    #[arg(short, long, default_value = "")]
    addons: String,

    /// Alias for --addons (single value; prefer --addons).
    #[arg(long, default_value = "")]
    extend: String,

    /// Set key=value (repeatable).
    #[arg(long = "set")]
    sets: Vec<String>,

    /// Use cra.config.json from a custom path (base for --set overlay).
    #[arg(long, default_value = "")]
    config: String,

    /// Allow non-empty target directory / skip clean confirm.
    #[arg(short, long)]
    force: bool,

    /// Skip the post-scaffold `cargo check`.
    #[arg(long)]
    no_install: bool,

    /// Skip the post-scaffold `cargo check` (alias for --no-install).
    #[arg(long)]
    skip_install: bool,

    /// Interactive prompts.
    #[arg(long, default_value_t = true)]
    interactive: bool,

    /// Disable interactive prompts.
    #[arg(long)]
    no_interactive: bool,

    /// List templates from catalog.
    #[arg(long)]
    list_templates: bool,

    /// List addons from catalog.
    #[arg(long)]
    list_addons: bool,

    /// Filter --list-templates by category slug (matches entry tags).
    #[arg(long, default_value = "")]
    category: String,

    /// Offline mode.
    #[arg(long)]
    offline: bool,

    /// Bypass catalog cache.
    #[arg(long)]
    no_cache: bool,

    /// Override CRA_CACHE_DIR.
    #[arg(long, default_value = "")]
    cache_dir: String,

    /// Pin git ref.
    #[arg(long, default_value = "")]
    pin: String,

    /// Cache refresh policy: always|stale|manual.
    #[arg(long, default_value = "")]
    refresh: String,

    /// Strict version checks.
    #[arg(long)]
    strict_version: bool,

    /// Keep project dir on failure.
    #[arg(long)]
    keep_on_failure: bool,

    /// Override templates.json URL.
    #[arg(long, default_value = "")]
    catalog_url: String,

    /// Local templates.json path.
    #[arg(long, default_value = "")]
    catalog_path: String,

    /// Use local fixtures/catalog templates.json.
    #[arg(long)]
    fixture: bool,

    /// Fixture catalog directory (implies --fixture).
    #[arg(long, default_value = "")]
    fixture_dir: String,

    /// Print completion script: bash|zsh|fish.
    #[arg(long, value_name = "SHELL", default_value = "")]
    add_completion: String,

    /// JSON output for cache subcommands and catalog lists.
    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Inspect or clean the local catalog cache.
    Cache {
        /// Cache action: status | path | clean.
        action: Option<String>,
    },
}

fn print_completion(shell: &str) -> Result<(), String> {
    let mut cmd = Cli::command();
    let name = "create-rust-app";
    match shell {
        "bash" => generate(Shell::Bash, &mut cmd, name, &mut io::stdout()),
        "zsh" => generate(Shell::Zsh, &mut cmd, name, &mut io::stdout()),
        "fish" => generate(Shell::Fish, &mut cmd, name, &mut io::stdout()),
        other => return Err(format!("unknown shell '{other}': expected bash|zsh|fish")),
    }
    Ok(())
}

fn split_addons(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn apply_env(cli: &Cli) {
    if cli.offline {
        std::env::set_var("CRA_OFFLINE", "1");
    }
    if cli.no_cache {
        std::env::set_var("CRA_NO_CACHE", "1");
    }
    if !cli.cache_dir.is_empty() {
        std::env::set_var("CRA_CACHE_DIR", &cli.cache_dir);
    }
    if !cli.refresh.is_empty() {
        std::env::set_var("CRA_REFRESH", &cli.refresh);
    }
    if cli.strict_version {
        std::env::set_var("CRA_STRICT_VERSION", "1");
    }
}

fn resolved_catalog_path(cli: &Cli) -> String {
    if cli.fixture || !cli.fixture_dir.is_empty() {
        std::env::set_var("CRA_CATALOG_FIXTURE", "1");
        let dir = if cli.fixture_dir.is_empty() {
            None
        } else {
            Some(cli.fixture_dir.as_str())
        };
        return resolve_fixture_catalog_path(dir)
            .to_string_lossy()
            .to_string();
    }
    cli.catalog_path.clone()
}

fn run_cache(action: Option<&str>, as_json: bool) -> ExitCode {
    match action.unwrap_or("status") {
        "status" => {
            let dir: PathBuf = cache_dir();
            if as_json {
                let payload = serde_json::json!({
                    "cache_dir": dir.to_string_lossy(),
                    "exists": dir.exists(),
                });
                println!("{payload}");
            } else if dir.exists() {
                println!("cache dir: {}", dir.to_string_lossy());
            } else {
                println!("cache dir: {} (absent)", dir.to_string_lossy());
            }
            ExitCode::SUCCESS
        }
        "path" => {
            let dir = cache_dir();
            if as_json {
                let payload = serde_json::json!({ "cache_dir": dir.to_string_lossy() });
                println!("{payload}");
            } else {
                println!("{}", dir.to_string_lossy());
            }
            ExitCode::SUCCESS
        }
        "clean" => match clean_cache() {
            Ok(()) => {
                if as_json {
                    println!("{{\"cleaned\":true}}");
                } else {
                    println!("cache cleaned");
                }
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("ERR {err}");
                ExitCode::FAILURE
            }
        },
        other => {
            eprintln!("ERR unknown cache action '{other}': expected status|path|clean");
            ExitCode::from(2)
        }
    }
}

fn run(cli: &Cli) -> ExitCode {
    if !cli.add_completion.is_empty() {
        return match print_completion(&cli.add_completion) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("ERR {err}");
                ExitCode::from(2)
            }
        };
    }

    if cli.info {
        println!("{}\n{}", brand_banner(), env_info());
        return ExitCode::SUCCESS;
    }

    apply_env(cli);
    let catalog_path = resolved_catalog_path(cli);
    let catalog_path_arg = if catalog_path.is_empty() {
        None
    } else {
        Some(catalog_path.as_str())
    };
    let catalog_url_arg = if cli.catalog_url.is_empty() {
        None
    } else {
        Some(cli.catalog_url.as_str())
    };

    if cli.list_templates || cli.list_addons {
        let catalog = match load_catalog(catalog_path_arg, catalog_url_arg) {
            Ok(catalog) => catalog,
            Err(err) => {
                eprintln!("ERR {err}");
                return ExitCode::FAILURE;
            }
        };
        let mut templates = list_template_names(&catalog);
        if cli.list_templates && !cli.category.is_empty() {
            templates = list_template_names_in_category(&catalog, &cli.category);
            if templates.is_empty() {
                eprintln!("no templates found for category '{}'", cli.category);
                return ExitCode::FAILURE;
            }
        }
        if cli.json && cli.list_templates && cli.list_addons {
            let payload = serde_json::json!({
                "templates": templates,
                "addons": list_addon_names(&catalog),
            });
            println!("{payload}");
            return ExitCode::SUCCESS;
        }
        if cli.list_templates {
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string(&templates).expect("names serialise")
                );
            } else {
                println!("{}", format_catalog_list("Templates", &templates));
            }
        }
        if cli.list_addons {
            let addons = list_addon_names(&catalog);
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string(&addons).expect("names serialise")
                );
            } else {
                println!("{}", format_catalog_list("Addons", &addons));
            }
        }
        return ExitCode::SUCCESS;
    }

    if let Some(Commands::Cache { action }) = &cli.command {
        return run_cache(action.as_deref(), cli.json);
    }

    let Some(project) = &cli.project else {
        eprintln!("ERR missing <project>: usage: create-rust-app <project> --template <slug>");
        return ExitCode::from(2);
    };

    if let Err(err) = validate_project_name(project) {
        eprintln!("ERR {err}");
        return ExitCode::from(2);
    }

    let mut addons = split_addons(&cli.addons);
    for extra in split_addons(&cli.extend) {
        if !addons.contains(&extra) {
            addons.push(extra);
        }
    }

    let mut sets = Vec::with_capacity(cli.sets.len());
    for raw in &cli.sets {
        match parse_set_override(raw) {
            Ok(parsed) => sets.push(parsed),
            Err(err) => {
                eprintln!("ERR {err}");
                return ExitCode::from(2);
            }
        }
    }

    if !cli.config.is_empty() && cli.verbose {
        println!("using config base: {}", cli.config);
    }
    if !cli.pin.is_empty() && cli.verbose {
        println!("pin: {}", cli.pin);
    }

    let catalog = match load_catalog(catalog_path_arg, catalog_url_arg) {
        Ok(catalog) => catalog,
        Err(err) => {
            if cli.offline {
                if cli.verbose {
                    eprintln!("warning: {err}; continuing offline");
                }
                create_rust_app_core::Catalog::default()
            } else {
                eprintln!("ERR {err}");
                return ExitCode::FAILURE;
            }
        }
    };

    let use_interactive = cli.interactive && !cli.no_interactive;
    if use_interactive && cli.verbose {
        println!("interactive prompts enabled");
    }

    let options = ScaffoldOptions {
        project: project.clone(),
        template: cli.template.clone(),
        addons,
        sets,
        force: cli.force,
        offline: cli.offline,
        keep_on_failure: cli.keep_on_failure,
    };
    match scaffold(&options, &catalog) {
        Ok(target) => {
            println!("OK  Scaffolded {}", target.to_string_lossy());
            let skip_install = cli.no_install || cli.skip_install;
            if !skip_install && !cli.offline && !options.template.is_empty() {
                // Best-effort post-scaffold check; never fails the run.
                let _ = std::process::Command::new("cargo")
                    .arg("check")
                    .arg("--quiet")
                    .current_dir(&target)
                    .output();
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("ERR {err}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    run(&Cli::parse())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_addon_lists() {
        assert_eq!(
            split_addons("a, b,,c "),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        assert!(split_addons("").is_empty());
    }
}
