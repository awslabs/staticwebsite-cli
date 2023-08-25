use aws_sdk_cloudformation::error::SdkError;
use aws_sdk_cloudformation::operation::create_stack::CreateStackError;
use aws_sdk_cloudformation::operation::describe_stacks::DescribeStacksError;
use aws_sdk_cloudformation::operation::update_stack::UpdateStackError;
use aws_sdk_cloudfront::operation::create_invalidation::CreateInvalidationError;
use aws_sdk_cloudfront::operation::get_invalidation::GetInvalidationError;
use aws_sdk_route53::operation::list_hosted_zones_by_name::ListHostedZonesByNameError;
use aws_sdk_s3::operation::put_object::PutObjectError;
use aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityError;
use tracing::dispatcher::SetGlobalDefaultError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    ///
    /// Custom errors
    ///

    #[error("Stack output not found")]
    StackOutputNotFound,

    #[error("Encountered unusuable stack status")]
    UnusableStackStatus,

    ///
    /// AWS SDK errors
    ///

    #[error("UpdateStack failed")]
    UpdateStackError {
        #[from]
        source: SdkError<UpdateStackError>,
    },

    #[error("ListHostedZonesByName failed")]
    ListHostedZonesByNameError {
        #[from]
        source: SdkError<ListHostedZonesByNameError>,
    },

    #[error("CreateStack failed")]
    CreateStackError {
        #[from]
        source: SdkError<CreateStackError>,
    },

    #[error("DescribeStacks failed")]
    DescribeStacksError {
        #[from]
        source: SdkError<DescribeStacksError>,
    },

    #[error("GetCallerIdentity failed")]
    GetCallerIdentityError {
        #[from]
        source: SdkError<GetCallerIdentityError>,
    },

    #[error("PutObject failed")]
    PutObjectError {
        #[from]
        source: SdkError<PutObjectError>,
    },

    #[error("CreateInvalidation failed")]
    CreateInvalidationError {
        #[from]
        source: SdkError<CreateInvalidationError>,
    },

    #[error("GetInvalidation failed")]
    GetInvalidationError {
        #[from]
        source: SdkError<GetInvalidationError>,
    },

    ///
    /// Other errors
    ///
    #[error("Upload stream error")]
    UploadStreamError {
        #[from]
        source: aws_smithy_http::byte_stream::error::Error,
    },

    #[error("Couldn't configure tracing provider")]
    TracingConfigurationError {
        #[from]
        source: SetGlobalDefaultError,
    },

    #[error("IO Error")]
    IOError {
        #[from]
        source: std::io::Error,
    },

    #[error("Timed out")]
    TimedOut {
        #[from]
        source: tokio::time::error::Elapsed,
    },
}
