use casbin::prelude::*;
use clap::Parser;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// The command to execute
    #[arg(value_enum)]
    command: Cmd,

    /// The path of the model file or model text
    #[arg(short, long)]
    model: String,

    /// The path of the policy file or policy text
    #[arg(short, long)]
    policy: String,

    /// The arguments for the enforcer
    command_args: Vec<String>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
#[clap(rename_all = "camelCase")]
pub enum Cmd {
    /// Check permissions
    Enforce,
    // /// Check permissions and get which policy it is
    // EnforceEx,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let model = DefaultModel::from_file(args.model)
        .await
        .expect("failed to load model");
    let adapter = FileAdapter::new(args.policy);

    let e = Enforcer::new(model, adapter)
        .await
        .expect("failed to create enforcer");
    let allow = e.enforce(args.command_args).expect("failed to enforce");

    let response = json!({
        "allow": allow,
        "explain": Vec::<String>::new(),
    });

    println!("{}", response);
}

#[tokio::test]
async fn test_enforce() {
    let args = Args {
        command: Cmd::Enforce,
        model: "examples/basic_model.conf".to_owned(),
        policy: "examples/basic_policy.csv".to_owned(),
        command_args: vec!["alice".to_owned(), "data1".to_owned(), "read".to_owned()],
    };

    let model = DefaultModel::from_file(args.model)
        .await
        .expect("failed to load model");
    let adapter = FileAdapter::new(args.policy);

    let e = Enforcer::new(model, adapter)
        .await
        .expect("failed to create enforcer");
    let allow = e.enforce(args.command_args).expect("failed to enforce");

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
