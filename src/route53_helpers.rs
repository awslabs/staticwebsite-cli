use crate::SdkError;
use aws_sdk_route53::error::ListHostedZonesByNameError;

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
    let zones = zones_response
        .hosted_zones()
        .expect("A zone with this name exists");

    // Extract the zone ID portion
    let zone_id_full = zones
        .first()
        .expect("We have a zone with this ID")
        .id()
        .expect("A zone has a zone ID")
        .to_string();
    let zone_id = zone_id_full.split("/");
    let zone_id_parts: Vec<&str> = zone_id.collect();
    Ok(zone_id_parts[2].to_string())
}
