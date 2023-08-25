use crate::{Error};
use aws_sdk_cloudformation::operation::create_stack::CreateStackError;
use aws_sdk_cloudformation::operation::update_stack::UpdateStackError;
use aws_sdk_cloudformation::types::{Parameter, StackStatus};
use std::time::Duration;
use aws_sdk_cloudformation::error::SdkError;
use tracing::{error, event, Level};

///
/// Waits indefinitely for a stack to be complete - either following an update, or an
/// initial creation. If the stack enters an error or rollback state, an error will be
/// returned.
///
/// This call should be wrapped in a timeout to ensure we don't spin our wheels indefinitely.
///
pub async fn wait_for_stack(
    stack_name: &String,
    cfn_client: &aws_sdk_cloudformation::Client,
) -> Result<(), Error> {
    loop {
        match cfn_client
            .describe_stacks()
            .stack_name(stack_name)
            .send()
            .await
        {
            Ok(stacks) => {
                let stack_status = stacks
                    .stacks()
                    .expect("stack body should be present")
                    .first()
                    .expect("stack should be present")
                    .stack_status()
                    .expect("stack should have a status");

                event!(Level::INFO, status = stack_status.as_str(), "Stack status");

                // If it's an "end state" status, then we can return now based on what the status
                // is. If the stack is still transitioning, we can pause a bit and check again.
                let status: Option<Result<(), Error>> = match stack_status {
                    // Either of these means our change was approved
                    StackStatus::CreateComplete | StackStatus::UpdateComplete => Some(Ok(())),

                    // Any of these mean something bad has happened
                    StackStatus::CreateFailed
                    | StackStatus::UpdateFailed
                    | StackStatus::UpdateRollbackInProgress
                    | StackStatus::UpdateRollbackFailed
                    | StackStatus::DeleteFailed
                    | StackStatus::DeleteInProgress
                    | StackStatus::DeleteComplete
                    | StackStatus::RollbackFailed
                    | StackStatus::RollbackInProgress
                    | StackStatus::UpdateRollbackComplete
                    | StackStatus::UpdateRollbackCompleteCleanupInProgress => {
                        Some(Err(Error::UnusableStackStatus))
                    }

                    // Any other status we're not done yet, and we should loop and wait
                    _ => None,
                };

                if let Some(status) = status {
                    return status;
                }

                // Pause before we check again
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Err(err) => {
                return Err(Error::DescribeStacksError { source: err });
            }
        }
    }
}

///
/// Checks if a given stack exists and is a complete, successful state, returning true/false accordingly.
/// If the function fails to find the stack, or the API call fails for some other reason, an Error will be returned.
///
pub async fn stack_exists_and_is_complete(
    stack_name: &String,
    cfn_client: &aws_sdk_cloudformation::Client,
) -> Result<bool, Error> {
    match cfn_client
        .describe_stacks()
        .stack_name(stack_name)
        .send()
        .await
    {
        Ok(stacks) => {
            // Get the stack, and check that it is complete. If it is in a different state we can't
            // update it.
            let stack = stacks
                .stacks()
                .expect("DescribeStacks for an existing stack should return the stack")
                .first()
                .expect("DescribeStacks with a response body should contain at least one stack");
            match stack.stack_status().unwrap() {
                StackStatus::CreateComplete |
                StackStatus::UpdateComplete |
                StackStatus::UpdateRollbackComplete => Ok(true),
                other => {
                    error!(
                        stack_status = other.as_str(),
                        "Got status and can't proceed"
                    );
                    Err(Error::UnusableStackStatus)
                }
            }
        }
        Err(e) => {
            // TODO - clean this up, once the 'kind' on describe_stats structures the errors properly, rather
            // than just string matching
            // https://github.com/awslabs/aws-sdk-rust/issues/678
            return if e.to_string().contains("does not exist") {
                Ok(false)
            } else {
                Err(Error::DescribeStacksError { source: e })
            };
        }
    }
}

///
/// Creates the static website stack. This function returns once the CreateStack call has been made to the API,
/// but does not wait for the stack to settle into a Complete state.
///
pub async fn create_stack(
    stack_name: &String,
    cfn_client: &aws_sdk_cloudformation::Client,
    stack_body: &String,
    zone_id: &String,
    domain_name: &String,
) -> Result<String, SdkError<CreateStackError>> {
    let create_stack_response = cfn_client
        .create_stack()
        .stack_name(stack_name)
        .template_body(stack_body)
        .parameters(
            Parameter::builder()
                .parameter_key("HostedZoneId")
                .parameter_value(zone_id)
                .build(),
        )
        .parameters(
            Parameter::builder()
                .parameter_key("DomainName")
                .parameter_value(domain_name)
                .build(),
        )
        .send()
        .await?;

    Ok(create_stack_response.stack_id().unwrap().to_string())
}

///
/// Updates the stack. This function returns once the UpdateStack call has been made to the API,
/// but does not wait for the stack to settle into a Complete state.
pub async fn update_stack(
    stack_name: &String,
    cfn_client: &aws_sdk_cloudformation::Client,
    stack_body: &String,
    zone_id: &String,
    domain_name: &String,
) -> Result<(), SdkError<UpdateStackError>> {
    let update_stack_response = cfn_client
        .update_stack()
        .stack_name(stack_name)
        .template_body(stack_body)
        .parameters(
            Parameter::builder()
                .parameter_key("HostedZoneId")
                .parameter_value(zone_id)
                .build(),
        )
        .parameters(
            Parameter::builder()
                .parameter_key("DomainName")
                .parameter_value(domain_name)
                .build(),
        )
        .send()
        .await;

    match update_stack_response {
        Ok(_) => Ok(()),
        Err(e) => {
            let error_txt = format!("{:?}", e);
            return if error_txt.to_string().contains("No updates are to be performed.") {
                Ok(())
            } else {
                Err(e)
            };
        }
    }
}

///
/// Retrieves the value of the given output from the given stack, or an error, if the output is missing
/// or the API call fails.
///
pub async fn get_stack_output(
    stack_name: &String,
    cfn_client: &aws_sdk_cloudformation::Client,
    output_name: &String,
) -> Result<String, Error> {
    let describe_result = cfn_client
        .describe_stacks()
        .stack_name(stack_name)
        .send()
        .await?;

    let outputs = describe_result
        .stacks()
        .expect("DescribeStacks looking for a stack output should return a stack")
        .first()
        .expect("DescribeStacks looking for a stack output should contain a non-nullable stack")
        .outputs()
        .expect("DescribeStacks looking for a stack output for our stack should contain outputs");

    for o in outputs {
        if o.output_key().unwrap().eq(output_name) {
            return Ok(o.output_value().unwrap().to_string());
        }
    }

    return Err(Error::StackOutputNotFound);
}
