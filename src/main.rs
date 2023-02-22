mod cloudformation_helpers;
mod cloudfront_helpers;
mod error;
mod route53_helpers;
mod s3_helpers;

use crate::cloudfront_helpers::{invalidate_distribution, wait_for_invalidation};
use crate::error::Error;
use crate::s3_helpers::upload_directory;
use aws_sdk_cloudformation::types::SdkError;
use clap::Parser;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::time::Duration;
use thiserror::Error;
use tokio::time::timeout;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Domain host. If this isn't specified, we will deploy to the apex.
    #[arg(long, default_value = "")]
    domain_name: String,

    /// Domain zone - the zone name into which we should deploy the domain.
    #[arg(long)]
    domain_zone: String,

    /// The directory to deploy
    #[arg(long)]
    deploy: String,
}

#[tokio::main]
async fn main() -> () {
    let args: Args = Args::parse();
    match run(&args).await {
        Ok(_) => {
            info!("All done!");
        }
        Err(err) => {
            error!(msg = err.to_string(), "Failed");
            match std::error::Error::source(&err) {
                Some(source) => {
                    error!(source, "Failed");
                }
                None => {}
            }

            exit(-1);
        }
    }
}

async fn run(args: &Args) -> Result<(), Error> {
    // Setup tracing to write out to the console
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    // Setup AWS Clients
    let shared_config = aws_config::from_env().region("us-east-1").load().await;
    let cfn_client = aws_sdk_cloudformation::Client::new(&shared_config);
    let r53_client = aws_sdk_route53::Client::new(&shared_config);
    let sts_client = aws_sdk_sts::Client::new(&shared_config);
    let s3_client = aws_sdk_s3::Client::new(&shared_config);
    let cloudfront_client = aws_sdk_cloudfront::Client::new(&shared_config);

    // Make sure the directory exists
    let path = Path::new(&args.deploy);
    let _ = fs::read_dir(path);

    // Make sure we've got access
    info!("Checking AWS access");
    let _caller_identity = sts_client.get_caller_identity().send().await?;
    info!("AWS access looks good, continuing");

    // Try find the zone ID
    let zone_id = route53_helpers::find_zone(&args.domain_zone, &r53_client).await?;
    info!(zone = zone_id, "Found zone");

    // Try find the stack
    let fqdn = if args.domain_name.eq("") {
        args.domain_zone.clone()
    } else {
        format!("{}.{}", args.domain_name, args.domain_zone)
    };

    // If the stack doesn't exist yet, let's deploy it
    let stack_template = include_str!("cfn_template.yaml").to_string();
    let stack_name = format!("StaticSite--{}", fqdn.replace(".", "-"));
    info!(name = &stack_name, "Using Cloudformation stack");
    if !cloudformation_helpers::stack_exists_and_is_complete(&stack_name, &cfn_client).await? {
        info!("Stack doesn't exist; creating");
        let stack_id = cloudformation_helpers::create_stack(
            &stack_name,
            &cfn_client,
            &stack_template,
            &zone_id,
            &fqdn,
        )
        .await?;
        info!(stack_id = &stack_id, "Stack created");
    } else {
        info!("Stack exists; updating");
        cloudformation_helpers::update_stack(
            &stack_name,
            &cfn_client,
            &stack_template,
            &zone_id,
            &fqdn,
        )
        .await?;
    }

    info!("Waiting for stack deployment to complete");
    timeout(
        Duration::from_secs(60 * 15),
        cloudformation_helpers::wait_for_stack(&stack_name, &cfn_client),
    )
    .await??;
    info!("Stack deploy complete");

    // Upload the site
    info!("Finding website bucket");
    let bucket_name = cloudformation_helpers::get_stack_output(
        &stack_name,
        &cfn_client,
        &"StaticWebsiteBucket".to_string(),
    )
    .await?;
    info!(bucket = &bucket_name, "Uploading");
    upload_directory(&path, &bucket_name, &s3_client).await?;

    // Invalidate the distribution
    let distribution_id = cloudformation_helpers::get_stack_output(
        &stack_name,
        &cfn_client,
        &"Distribution".to_string(),
    )
    .await?;
    info!(
        distribution_id = distribution_id,
        "Invalidating distribution"
    );
    let invalidation_id = invalidate_distribution(&distribution_id, &cloudfront_client).await?;
    timeout(
        Duration::from_secs(60 * 15),
        wait_for_invalidation(&invalidation_id, &distribution_id, &cloudfront_client),
    )
    .await??;

    info!("Distribution invalidated. Ready to go!");
    info!(href = format!("https://{}", fqdn), "Link");
    Ok(())
}
