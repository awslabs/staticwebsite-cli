use aws_sdk_route53::error::SdkError;
use aws_sdk_route53::operation::list_hosted_zones_by_name::ListHostedZonesByNameError;

///
/// Finds the Zone ID for the given zone name.
///
pub async fn find_zone(
    domain_zone: &String,
    r53_client: &aws_sdk_route53::Client,
) -> Result<String, SdkError<ListHostedZonesByNameError>> {
    let zones_response = r53_client
        .list_hosted_zones_by_name()
        .dns_name(domain_zone)
        .send()
        .await?;
    let zone = zones_response
        .hosted_zones()
        .first()
        .expect("A zone with this name exists");

    // Extract the zone ID portion
    let zone_id_full = zone
        .id()
        .to_string();
    let zone_id = zone_id_full.split("/");
    let zone_id_parts: Vec<&str> = zone_id.collect();
    Ok(zone_id_parts[2].to_string())
}
