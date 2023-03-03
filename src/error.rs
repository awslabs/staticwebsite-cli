use crate::SdkError;
use aws_sdk_cloudformation::error::{CreateStackError, DescribeStacksError, UpdateStackError};
use aws_sdk_cloudfront::error::{CreateInvalidationError, GetInvalidationError};
use aws_sdk_route53::error::ListHostedZonesByNameError;
use aws_sdk_s3::error::PutObjectError;
use aws_sdk_sts::error::GetCallerIdentityError;
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
