use atty::Stream;
use clap::Parser;
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{env, fs, io, path::Path};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// List available system prompts
    #[arg(short, long, action)]
    list: bool,

    /// Set system command by specifying predefined system e.g. -s copywright
    /// for copywright.md system in $HOME/.ohl/systems/
    #[arg(short, long, value_name = "SYSTEM")]
    system: Option<String>,

    /// Set user prompt
    user_prompt: Option<String>,
}
#[derive(Deserialize, Serialize, Debug)]
struct Config {
    /// Ollama server url, default: http://localhost
    url: String,
    /// Ollama server port, default: 11434
    port: u16,
    /// Model to use by ollama (has to be installed), default: llama3
    model: String,
}

enum Response {
    Ok(String),
    Err(String),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: "http://localhost".to_string(),
            port: 11434,
            model: "dolphin-llama3".to_string(),
        }
    }
}

const SYSTEM_PROMPT: &str = "System:";
const USER_PROMPT: &str = "User:";
const USER_HOME_ENV: &str = "HOME";
const OHLLAMA_DIR: &str = ".ohllama";
const SYSTEMS_DIR: &str = "systems";
const CONFIG_FILE: &str = "config.toml";
const DEFUALT_SYSTEM_FILE: &str = "default.md";
const DEFAULT_SYSTEM_PRMOPT: &str = 
    "You are an AI working with cli input, an expert at processing instructions and command output. 
    Your return output in format that is easily processable by other tools. 
    You return no additional comments or remarks.
    Just return the output of what you were asked to do.";

#[tokio::main]
async fn main() -> Result<(), String> {
    let user_home = &env::var(USER_HOME_ENV).expect("Missing $HOME env.");
    let base_path = format!("{user_home}/{OHLLAMA_DIR}");

    let config_path_string = format!("{base_path}/{CONFIG_FILE}");
    let config_path = Path::new(&config_path_string);
    let config: Config;

    setup(&base_path)?;

    let file = fs::read_to_string(config_path).expect("Failed reading config file");
    config = toml::from_str(&file).expect("Failed parsing TOML config");

    let cli = Cli::parse();
    let stdin = load_stdin();
    let system_prompt = load_system(cli.system)?;
    if cli.list {
        list_system_prompts(&base_path);
        return Ok(());
    }

    let user_prompt = cli.user_prompt.expect("Missing User prompt!");
    let prompt = format!(
        "{} {}.\n{} {}{}",
        SYSTEM_PROMPT, system_prompt, USER_PROMPT, user_prompt, stdin
    );

    let ollama = Ollama::new(config.url, config.port);
    let res = prompt_ollama(prompt, &ollama, config.model).await;
    match res {
        Response::Err(err) => Err(format!(
            "unable to request Ollama, is Ollama running?: {}",
            err
        )),
        Response::Ok(res) => {
            output_to_stdout(res.as_str());
            Ok(())
        }
    }
}

fn list_system_prompts(base_path: &str) {
    let system_prompts_path = format!("{base_path}/{SYSTEMS_DIR}");
    let paths = fs::read_dir(system_prompts_path).unwrap();
    for path in paths {
        let entry = path.unwrap();
        let system_file = entry.file_name().into_string().unwrap();
        let system = system_file.split("/").last().unwrap();
        println!("System: {}", system);
        let system_prompt: String =
            fs::read_to_string(entry.path()).expect("Failed reading system file: {system}");
        println!("{system_prompt}\n");
    }
}

fn setup(base_path: &str) -> Result<(), String> {
    let config_path = format!("{base_path}/{CONFIG_FILE}");
    let system_prompts_path = format!("{base_path}/{SYSTEMS_DIR}");
    let default_system_prompt_path = format!("{system_prompts_path}/{DEFUALT_SYSTEM_FILE}");

    if Path::new(&base_path).exists()
        && Path::new(&config_path).exists()
        && Path::new(&default_system_prompt_path).exists()
    {
        return Ok(());
    }

    let config_string = toml::to_string_pretty(&Config::default()).unwrap();
    fs::create_dir_all(system_prompts_path.clone()).expect("Failed creating Ohllama's directories");
    fs::write(config_path, config_string)
        .expect("Failed creating default config file at: {config_path}");
    fs::write(default_system_prompt_path, DEFAULT_SYSTEM_PRMOPT)
        .expect("Failed creating default system prompt file at: {default_system_prompt_path}");
    Ok(())
}

async fn prompt_ollama(prompt: String, ollama: &Ollama, model: String) -> Response {
    let res = ollama.generate(GenerationRequest::new(model, prompt)).await;
    match res {
        Ok(resp) => Response::Ok(resp.response),
        Err(err) => Response::Err(err.to_string()),
    }
}

fn load_stdin() -> String {
    if atty::is(Stream::Stdin) {
        return String::new();
    }
    io::stdin()
        .lines()
        .map(|line| line.unwrap_or_default())
        .collect::<Vec<String>>()
        .join("\n")
}

fn output_to_stdout(output: &str) {
    let mut stdout = io::stdout().lock();
    stdout.write_all(output.as_bytes()).unwrap();
    stdout.flush().unwrap();
}

fn load_system(system_name: Option<String>) -> Result<String, String> {
    let system = system_name.unwrap_or("default".to_string());
    let user_home = &env::var(USER_HOME_ENV).expect("Missing $HOME env.");
    let format_path = format!("{user_home}/{OHLLAMA_DIR}/{SYSTEMS_DIR}/{system}.md");
    let system_path = Path::new(&format_path);
    if system_path.exists() {
        let system_prompt: String =
            fs::read_to_string(system_path).expect("Failed reading config file");
        Ok(system_prompt)
    } else {
        Err("Missing system, no file found: {system_path}".to_string())
    }
}
