use clap::Parser;
use dfir_rs::tokio_util::codec::LinesCodec;
use hydro_deploy::Deployment;
use hydro_lang::deploy::TrybuildHost;
use hydro_lang::graph::config::GraphConfig;
use hydro_lang::location::{Location, NetworkHint};
use hydro_lang::nondet::nondet;

#[derive(Parser, Debug)]
struct Args {
    #[clap(flatten)]
    graph: GraphConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut deployment = Deployment::new();
    let flow = hydro_lang::compile::builder::FlowBuilder::new();

    let process = flow.process::<()>();
    let external = flow.external::<()>();

    let (port, input, membership, output_ref) =
        process.bidi_external_many_bytes::<_, _, LinesCodec>(&external, NetworkHint::Auto);

    output_ref.complete(hydro_test::external_client::tic_tac_toe::tic_tac_toe_server(
        &process,
        input,
        membership,
        nondet!(/** game synchronization */),
    ));

    // Extract the IR BEFORE the builder is consumed by deployment methods
    let built = flow.finalize();

    // Generate graph visualizations based on command line arguments
    built.generate_graph_with_config(&args.graph, None)?;

    // Now use the built flow for deployment with optimization
    let nodes = built
        .with_default_optimize()
        .with_process(&process, TrybuildHost::new(deployment.Localhost()))
        .with_external(&external, deployment.Localhost())
        .deploy(&mut deployment);

    deployment.deploy().await.unwrap();

    let raw_port = nodes.raw_port(port);
    let server_port = raw_port.server_port().await;

    deployment.start().await.unwrap();

    println!("\n{}", "=".repeat(60));
    println!("🎮 Tic-Tac-Toe Game Server Started! 🎮");
    println!("{}", "=".repeat(60));
    println!("\nServer is listening on: {:?}", server_port);
    println!("\nHow to play:");
    println!("  1. Connect two clients using netcat:");
    println!("     Player 1: nc localhost {:?}", server_port);
    println!("     Player 2: nc localhost {:?}", server_port);
    println!("\n  2. First player will be assigned 'X', second player 'O'");
    println!("  3. Enter a number (0-8) to make your move:");
    println!("     0 | 1 | 2");
    println!("     ---------");
    println!("     3 | 4 | 5");
    println!("     ---------");
    println!("     6 | 7 | 8");
    println!("\n  4. Players take turns until someone wins or it's a draw!");
    println!("\n{}", "=".repeat(60));
    println!("\nPress Ctrl+C to stop the server...\n");

    tokio::signal::ctrl_c().await.unwrap();
    println!("\nShutting down server...");
    Ok(())
}

#[test]
fn test() {
    use example_test::run_current_example;

    let mut run = run_current_example!();
    run.read_regex(r"Tic-Tac-Toe Game Server Started");
}
