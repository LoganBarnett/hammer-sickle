use crate::config::Config;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// The Foreman API for /api/hosts can be found here:
// https://theforeman.org/api/2.3/apidoc/v2/hosts/index.html
//
// The documentation is rather lacking in examples of complex structures, and
// instead they opt for "null" in the examples. This documentation has no
// formal structure listed. The structures below are guesses based on real
// queries in private infrastructure, so they won't be shared here. These
// structures will likely need revision to match what the APIs truly use.

#[derive(Debug, Error)]
pub enum ForemanError {
  #[error("Failed to fetch hosts from Foreman at {url}: {source}")]
  HostFetch {
    url: String,
    #[source]
    source: reqwest::Error,
  },

  #[error("Failed to parse hosts response from Foreman at {url}: {source}")]
  ResponseParse {
    url: String,
    #[source]
    source: reqwest::Error,
  },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForemanApiProxy {
  pub id: i64,
  pub name: Option<String>,
  pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForemanApiSortBy {
  pub by: Option<String>,
  pub order: Option<String>,
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
  pub page: i64,
  pub per_page: i64,
  pub search: Option<String>,
  pub results: Vec<R>,
  pub sort: ForemanApiSortBy,
  pub subtotal: i64,
  pub total: i64,
}

pub fn fetch_hosts(config: &Config) -> Result<Vec<String>, ForemanError> {
  let url = format!(
    "{url_base}/api/hosts?search={search}",
    url_base = config.foreman_url,
    search = config.search,
  );

  // Foreman supports both OAuth and basic auth. Use basic for simplicity for
  // now. See https://projects.theforeman.org/projects/foreman/wiki/API_OAuth
  // when that fateful day arrives.
  let resp = reqwest::blocking::Client::new()
    .get(&url)
    .basic_auth(&config.foreman_user, Some(&config.foreman_password))
    .send()
    .map_err(|source| ForemanError::HostFetch {
      url: url.clone(),
      source,
    })?
    .json::<ForemanApiPage<ForemanApiHost>>()
    .map_err(|source| ForemanError::ResponseParse { url, source })?;

  Ok(resp.results.into_iter().map(|h| h.name).collect())
}
