# Static Website CLI

This CLI tool makes it easy to deploy a static website to AWS. It builds and hosts the website, sets up a CDN and DNS,
and provisions an SSL certificate.

## Prerequisites

In order to manage DNS, the tool needs to be able to configure endpoints in your DNS zone. You must have an existing
Route53 zone corresponding to the `--domain-zone` specified. To purchase a new domain name and setup the corresponding
zone, follow the [Register a domain name](https://aws.amazon.com/getting-started/hands-on/get-a-domain/) instructions here.

This will have cost implications!

### AWS Access
The tool needs AWS CLI credentials setup in order to deploy to AWS. The minimum policy required is available in a
sample [policy.json](policy.json) document. 

Although the AWS CLI itself isn't required, staticwebsite-cli uses the same configuration system. The CLI's 
[credential file configuration](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html)
documentation will help you get started.

You will need to either:

* Setup a profile in your AWS credentials file
* Setup environment variables `AWS_ACCESS_KEY` and `AWS_SECRET_ACCESS_KEY` 

## Usage
Simply specify:
```
--domain-zone The zone to deploy the website into - e.g. - mydomain.com
--domain-name The name of the host. If this isn't specified, we'll deploy at the apex
--deploy The directory containing the static website to deploy
```
Your website will then be accessible at `https://{domain-name}.{domain-zone}` or simply `https://{domain-zone}` if you 
didn't specify a `domain-name` within the zone

```bash
> staticwebsite_cli --domain-zone demo.com --domain-name rustacean --deploy sample

💻 ➜  staticwebsite-cli git:(main) ✗ target/debug/staticwebsite_cli --domain-zone demo.com --domain-name rustacean --deploy test
INFO staticwebsite_cli: Checking AWS access
INFO staticwebsite_cli: AWS access looks good, continuing
INFO staticwebsite_cli: Found zone zone="ABCDEFG"
INFO staticwebsite_cli: Using Cloudformation stack name="StaticSite--rustacean-demo-com"
INFO staticwebsite_cli: Stack doesn't exist; creating
INFO staticwebsite_cli: Stack created stack_id="..."
INFO staticwebsite_cli: Waiting for stack deployment to complete
...
INFO staticwebsite_cli::cloudformation_helpers: Stack status status="CREATE_IN_PROGRESS"
INFO staticwebsite_cli::cloudformation_helpers: Stack status status="CREATE_COMPLETE"
INFO staticwebsite_cli: Stack deploy complete
INFO staticwebsite_cli: Finding website bucket
INFO staticwebsite_cli: Uploading bucket="..."
INFO staticwebsite_cli: Invalidating distribution distribution_id="E3EF9EZ9CV2KGJ"
INFO staticwebsite_cli::cloudfront_helpers: Waiting for invalidation to complete
INFO staticwebsite_cli::cloudfront_helpers: Invalidation status="InProgress"
INFO staticwebsite_cli::cloudfront_helpers: Invalidation status="Completed"
INFO staticwebsite_cli: Distribution invalidated. Ready to go!
INFO staticwebsite_cli: Link href="https://rustacean.demo.com"
INFO staticwebsite_cli: All done!
```

## Updates
Simply re-run `staticwebsite_cli` with the same arguments to replace the contents of the website. The CLI will invalidate
the CDN distribution and the changes should become immediately available.

## Removing the stack

1. Login to the AWS console
1. Visit the [Cloudformation console in us-east-1](https://us-east-1.console.aws.amazon.com/cloudformation/home?region=us-east-1#/stacks)
1. Find the stack named `StaticSite--your-domain-name` and delete it.