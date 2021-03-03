// use std::process::Command;
// use std::collections::HashMap;
// use std::collections::Array;
// use std::collections::Vector;
use serde::{Deserialize, Serialize};
use std::env;
use structopt::StructOpt;

// The Foreman API for /api/hosts can be found here:

// https://theforeman.org/api/2.3/apidoc/v2/hosts/index.html
// The documentation is rather lacking in examples of complex structures, and
// instead they opt for "null" in the examples. This documentation has no formal
// structure listed. The structures below are guesses based on real queries in
// private infrastructure, so they won't be shared here. These structures will
// likely need revision to match what the APIs truly use.
#[derive(Serialize, Deserialize, Debug)]
struct ForemanApiProxy {
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
struct ForemanApiSortBy {
    by: Option<String>,
    order: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ForemanApiHost {
    architecture_id: Option<i64>,
    architecture_name: Option<String>,
    build: bool,
    capabilities: Vec<String>,
    certname: Option<String>,
    comment: Option<String>,
    compute_profile_id: Option<i64>,
    compute_profile_name: Option<String>,
    compute_resource_id: Option<i64>,
    compute_resource_name: Option<String>,
    created_at: Option<String>,
    disk: Option<String>,
    domain_id: Option<i64>,
    domain_name: Option<String>,
    enabled: bool,
    environment_id: Option<i64>,
    environment_name: Option<String>,
    global_status: i64,
    global_status_label: Option<String>,
    hostgroup_id: Option<i64>,
    hostgroup_name: Option<String>,
    hostgroup_title: Option<String>,
    id: i64,
    image_file: String,
    image_id: Option<i64>,
    image_name: Option<String>,
    installed_at: Option<String>,
    ip6: Option<String>,
    ip: Option<String>,
    last_compile: Option<String>,
    last_report: Option<String>,
    location_id: Option<i64>,
    location_name: Option<String>,
    mac: Option<String>,
    managed: bool,
    medium_id: Option<i64>,
    medium_name: Option<String>,
    model_id: Option<i64>,
    model_name: Option<String>,
    name: String,
    operatingsystem_id: Option<i64>,
    operatingsystem_name: Option<String>,
    organization_id: Option<i64>,
    organization_name: Option<String>,
    owner_id: i64,
    owner_name: String,
    owner_type: String,
    provision_method: Option<String>,
    ptable_id: Option<i64>,
    ptable_name: Option<String>,
    puppet_ca_proxy: Option<ForemanApiProxy>,
    puppet_ca_proxy_id: Option<i64>,
    puppet_ca_proxy_name: Option<String>,
    puppet_proxy: Option<ForemanApiProxy>,
    puppet_proxy_id: Option<i64>,
    puppet_proxy_name: Option<String>,
    puppet_status: i64,
    pxe_loader: Option<String>,
    realm_id: Option<i64>,
    realm_name: Option<String>,
    registration_token: Option<String>,
    sp_ip: Option<String>,
    sp_mac: Option<String>,
    sp_name: Option<String>,
    sp_subnet_id: Option<i64>,
    subnet6_id: Option<i64>,
    subnet6_name: Option<String>,
    subnet_id: Option<i64>,
    subnet_name: Option<String>,
    updated_at: Option<String>,
    uptime_seconds: Option<String>,
    use_image: Option<String>,
    uuid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ForemanApiPage<R> {
    page: i64,
    per_page: i64,
    search: Option<String>,
    results: Vec<R>,
    sort: ForemanApiSortBy,
    subtotal: i64,
    total: i64,
}

#[derive(StructOpt)]
#[structopt(
    name = "hammer-sickle",
    about = "Run commands across Foreman controlled hosts.",
)]
struct Cli {
    #[structopt(short, long)]
    command: String,
    #[structopt(short, long)]
    search: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let username = env::var("FOREMAN_USER")
        .unwrap_or(env::var("USER").unwrap());
    let password = env::var("FOREMAN_PASS").unwrap();
    let url_base = env::var("FOREMAN_URL_BASE").unwrap();
    let args = Cli::from_args();
    let client = reqwest::blocking::Client::new()
        ;
    let resp = client.get(
        format!(
            "{url_base}/api/hosts?search={search}",
            url_base = url_base,
            search = args.search,
        )
    )
    // Foreman supports both OAuth and basic auth. Use basic for simplicity for
    // now. See https://projects.theforeman.org/projects/foreman/wiki/API_OAuth
    // when that fateful date arrives.
      .basic_auth(username, Some(password))
      .send()?
      .json::<ForemanApiPage<ForemanApiHost>>()?
    ;
    let hosts = resp.results.iter().map(|x| x.name.clone());
    hosts.map(|name| host_command_send(host, command))
    Ok(())
}

fn host_command_send(hostname: String, command: String) {

}
