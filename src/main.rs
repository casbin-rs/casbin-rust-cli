use casbin::{
    prelude::*,
    rhai::{Dynamic, Map},
};
use clap::{CommandFactory, Parser, Subcommand};
use serde_json::{json, Value};
use std::{hash::Hash, str::FromStr, sync::LazyLock};

build_info::build_info!(fn build_info);

static VERSION: LazyLock<String> = LazyLock::new(|| {
    let info = build_info();
    let cli_version = option_env!("VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    let casbin_version = info
        .crate_info
        .dependencies
        .iter()
        .find_map(|dep| {
            if dep.name == "casbin" {
                Some(dep.version.to_string())
            } else {
                None
            }
        })
        .expect("casbin version not found");
    format!("{}\ncasbin-rs v{}", cli_version, casbin_version)
});

#[derive(Parser, Debug, Clone)]
#[command(author, about, long_about, version=VERSION.as_str())]
struct Args {
    /// The command to execute
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "camelCase")]
pub enum Cmd {
    /// Generate the autocompletion script for the specified shell
    Completion {
        /// The shell to generate the completions for
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
    /// Check permissions
    Enforce {
        /// The path of the model file or model text
        #[arg(short, long)]
        model: String,

        /// The path of the policy file or policy text
        #[arg(short, long)]
        policy: String,

        /// The arguments for the enforcer
        command_args: Vec<String>,
    },
    /// Check permissions and get which policy it is
    EnforceEx {
        /// The path of the model file or model text
        #[arg(short, long)]
        model: String,

        /// The path of the policy file or policy text
        #[arg(short, long)]
        policy: String,

        /// The arguments for the enforcer
        command_args: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Cmd::Enforce {
            model,
            policy,
            command_args,
        } => {
            println!("{}", enforce(&model, &policy, &command_args).await);
        }
        Cmd::EnforceEx {
            model,
            policy,
            command_args,
        } => {
            println!("{}", enforce_ex(&model, &policy, &command_args).await);
        }
        Cmd::Completion { shell } => {
            shell.generate(&mut Args::command(), &mut std::io::stdout());
        }
    };
}

async fn enforce(model: &str, policy: &str, command_args: &[String]) -> String {
    let model = DefaultModel::from_file(model.to_owned())
        .await
        .expect("failed to load model");
    let adapter = FileAdapter::new(policy.to_owned());

    let e = Enforcer::new(model, adapter)
        .await
        .expect("failed to create enforcer");

    let allow = e
        .enforce(parse_args(command_args))
        .expect("failed to enforce");

    json!({
        "allow": allow,
        "explain": Vec::<String>::new(),
    })
    .to_string()
}

async fn enforce_ex(model: &str, policy: &str, command_args: &[String]) -> String {
    let model = DefaultModel::from_file(model.to_owned())
        .await
        .expect("failed to load model");
    let adapter = FileAdapter::new(policy.to_owned());

    let e = Enforcer::new(model, adapter)
        .await
        .expect("failed to create enforcer");

    let (allow, explain) = e
        .enforce_ex(parse_args(command_args))
        .expect("failed to enforce");

    json!({
        "allow": allow,
        "explain": explain.first().unwrap_or(&Vec::<String>::new()),
    })
    .to_string()
}

#[derive(Clone, Hash, Debug)]
pub struct CommandArg(Value);

impl From<CommandArg> for Dynamic {
    fn from(arg: CommandArg) -> Self {
        match arg.0 {
            Value::Object(map) => {
                let mut rhai_map = Map::new();
                for (k, v) in map {
                    let v = Dynamic::from(match v {
                        Value::String(s) => s,
                        _ => v.to_string(),
                    });
                    rhai_map.insert(k.into(), v);
                }
                Dynamic::from(rhai_map)
            }
            Value::String(s) => Dynamic::from(s),
            Value::Bool(b) => Dynamic::from(b),
            Value::Number(n) => n.as_f64().map(Dynamic::from).unwrap_or(Dynamic::UNIT),
            Value::Array(arr) => {
                Dynamic::from(arr.into_iter().map(Dynamic::from).collect::<Vec<_>>())
            }
            Value::Null => Dynamic::UNIT,
        }
    }
}

fn parse_args(command_args: &[String]) -> Vec<CommandArg> {
    command_args
        .iter()
        .map(|arg| {
            CommandArg(
                serde_json::Value::from_str(arg)
                    .unwrap_or(serde_json::Value::String(arg.to_string())),
            )
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_enforce() {
        let response = enforce(
            "examples/basic_model.conf",
            "examples/basic_policy.csv",
            &vec!["alice".to_owned(), "data1".to_owned(), "read".to_owned()],
        )
        .await;

        let expected = json!({
            "allow": true,
            "explain": [],
        })
        .to_string();

        assert_eq!(response, expected);
    }

    #[tokio::test]
    async fn test_enforce_explain() {
        let response = enforce_ex(
            "examples/basic_model.conf",
            "examples/basic_policy.csv",
            &vec!["alice".to_owned(), "data1".to_owned(), "read".to_owned()],
        )
        .await;

        let expected = json!({
            "allow": true,
            "explain": vec!["alice", "data1" ,"read"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        })
        .to_string();

        assert_eq!(response, expected);
    }

    #[tokio::test]
    async fn test_abac() {
        let response = enforce_ex(
            "examples/abac_model.conf",
            "examples/abac_policy.csv",
            &vec!["alice".to_owned(), json!({"Owner": "alice"}).to_string()],
        )
        .await;

        let expected = json!({
            "allow": true,
            "explain": [],
        })
        .to_string();

        assert_eq!(response, expected);
    }
}
