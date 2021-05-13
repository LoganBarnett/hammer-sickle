use serde::{Deserialize, Serialize};
// The Foreman API for /api/hosts can be found here:
// https://theforeman.org/api/2.3/apidoc/v2/hosts/index.html
//
// The documentation is rather lacking in examples of complex structures, and
// instead they opt for "null" in the examples. This documentation has no formal
// structure listed. The structures below are guesses based on real queries in
// private infrastructure, so they won't be shared here. These structures will
// likely need revision to match what the APIs truly use.
#[derive(Serialize, Deserialize, Debug)]
pub struct ForemanApiProxy {
    id: i64,
    name: Option<String>,
    url: Option<String>,
}

// Don't actually use this - Foremand doesn't exhaustively document this, and
// doesn't indicate that the values could be anything.
// String BUILD = "build",
// String IMAGE = "image",
// String SNAPSHOTS = "snapshots",

#[derive(Serialize, Deserialize, Debug)]
pub struct ForemanApiSortBy {
    by: Option<String>,
    order: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForemanApiHost {
    pub architecture_id: Option<i64>,
    pub architecture_name: Option<String>,
    pub build: bool,
    pub capabilities: Vec<String>,
    pub certname: Option<String>,
    pub comment: Option<String>,
    pub compute_profile_id: Option<i64>,
    pub compute_profile_name: Option<String>,
    pub compute_resource_id: Option<i64>,
    pub compute_resource_name: Option<String>,
    pub created_at: Option<String>,
    pub disk: Option<String>,
    pub domain_id: Option<i64>,
    pub domain_name: Option<String>,
    pub enabled: bool,
    pub environment_id: Option<i64>,
    pub environment_name: Option<String>,
    pub global_status: i64,
    pub global_status_label: Option<String>,
    pub hostgroup_id: Option<i64>,
    pub hostgroup_name: Option<String>,
    pub hostgroup_title: Option<String>,
    pub id: i64,
    pub image_file: String,
    pub image_id: Option<i64>,
    pub image_name: Option<String>,
    pub installed_at: Option<String>,
    pub ip6: Option<String>,
    pub ip: Option<String>,
    pub last_compile: Option<String>,
    pub last_report: Option<String>,
    pub location_id: Option<i64>,
    pub location_name: Option<String>,
    pub mac: Option<String>,
    pub managed: bool,
    pub medium_id: Option<i64>,
    pub medium_name: Option<String>,
    pub model_id: Option<i64>,
    pub model_name: Option<String>,
    pub name: String,
    pub operatingsystem_id: Option<i64>,
    pub operatingsystem_name: Option<String>,
    pub organization_id: Option<i64>,
    pub organization_name: Option<String>,
    pub owner_id: i64,
    pub owner_name: String,
    pub owner_type: String,
    pub provision_method: Option<String>,
    pub ptable_id: Option<i64>,
    pub ptable_name: Option<String>,
    pub puppet_ca_proxy: Option<ForemanApiProxy>,
    pub puppet_ca_proxy_id: Option<i64>,
    pub puppet_ca_proxy_name: Option<String>,
    pub puppet_proxy: Option<ForemanApiProxy>,
    pub puppet_proxy_id: Option<i64>,
    pub puppet_proxy_name: Option<String>,
    pub puppet_status: i64,
    pub pxe_loader: Option<String>,
    pub realm_id: Option<i64>,
    pub realm_name: Option<String>,
    pub registration_token: Option<String>,
    pub sp_ip: Option<String>,
    pub sp_mac: Option<String>,
    pub sp_name: Option<String>,
    pub sp_subnet_id: Option<i64>,
    pub subnet6_id: Option<i64>,
    pub subnet6_name: Option<String>,
    pub subnet_id: Option<i64>,
    pub subnet_name: Option<String>,
    pub updated_at: Option<String>,
    pub uptime_seconds: Option<String>,
    pub use_image: Option<String>,
    pub uuid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForemanApiPage<R> {
    page: i64,
    per_page: i64,
    search: Option<String>,
    pub results: Vec<R>,
    sort: ForemanApiSortBy,
    subtotal: i64,
    total: i64,
}
