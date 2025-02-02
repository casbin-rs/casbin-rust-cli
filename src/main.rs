use casbin::prelude::*;
use clap::{CommandFactory, Parser, Subcommand};
use serde_json::json;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about)]
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
            let model = DefaultModel::from_file(model)
                .await
                .expect("failed to load model");
            let adapter = FileAdapter::new(policy);

            let e = Enforcer::new(model, adapter)
                .await
                .expect("failed to create enforcer");

            let allow = e.enforce(command_args).expect("failed to enforce");

            let response = json!({
                "allow": allow,
                "explain": Vec::<String>::new(),
            });

            println!("{}", response);
        }
        Cmd::EnforceEx {
            model,
            policy,
            command_args,
        } => {
            let model = DefaultModel::from_file(model)
                .await
                .expect("failed to load model");
            let adapter = FileAdapter::new(policy);

            let e = Enforcer::new(model, adapter)
                .await
                .expect("failed to create enforcer");

            let (allow, explain) = e.enforce_ex(command_args).expect("failed to enforce");

            let response = json!({
                "allow": allow,
                "explain": explain.first().unwrap_or(&Vec::<String>::new()),
            });

            println!("{}", response);
        }
        Cmd::Completion { shell } => {
            shell.generate(&mut Args::command(), &mut std::io::stdout());
        }
    };
}

#[tokio::test]
async fn test_enforce() {
    let model = DefaultModel::from_file("examples/basic_model.conf".to_owned())
        .await
        .expect("failed to load model");
    let adapter = FileAdapter::new("examples/basic_policy.csv".to_owned());

    let e = Enforcer::new(model, adapter)
        .await
        .expect("failed to create enforcer");

    let allow = e
        .enforce(vec![
            "alice".to_owned(),
            "data1".to_owned(),
            "read".to_owned(),
        ])
        .expect("failed to enforce");

    let response = json!({
        "allow": allow,
        "explain": Vec::<String>::new(),
    });

    let expected = json!({
        "allow": true,
        "explain": [],
    });

    assert_eq!(response, expected);
}

#[tokio::test]
async fn test_enforce_explain() {
    let model = DefaultModel::from_file("examples/basic_model.conf".to_owned())
        .await
        .expect("failed to load model");
    let adapter = FileAdapter::new("examples/basic_policy.csv".to_owned());

    let e = Enforcer::new(model, adapter)
        .await
        .expect("failed to create enforcer");

    let (allow, explain) = e
        .enforce_ex(vec![
            "alice".to_owned(),
            "data1".to_owned(),
            "read".to_owned(),
        ])
        .expect("failed to enforce");

    let response = json!({
        "allow": allow,
        "explain": explain.first().unwrap_or(&Vec::<String>::new()),
    });

    let expected = json!({
        "allow": true,
        "explain": ["alice", "data1", "read"],
    });

    assert_eq!(response, expected);
}
