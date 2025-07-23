use maxminddb::geoip2;
use maxminddb::Reader;
use std::net::IpAddr;
use std::str::FromStr;
use log::{info, error};
use std::sync::Arc;

#[derive(Clone)]
pub struct GeoIpService {
    reader: Arc<Reader<Vec<u8>>>,
}

impl GeoIpService {
    pub fn new(db_path: &str) -> Result<Self, maxminddb::MaxMindDBError> {
        info!("Initializing GeoIP service with database: {}", db_path);
        let reader = Reader::open_readfile(db_path)?;
        Ok(Self { 
            reader: Arc::new(reader) 
        })
    }

    pub fn get_country_code(&self, ip: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let ip_addr = IpAddr::from_str(ip)?;
        match self.reader.lookup::<geoip2::Country>(ip_addr) {
            Ok(country) => {
                let country_code = country
                    .country
                    .and_then(|c| c.iso_code)
                    .map(|code| code.to_string());
                
                info!("IP {} resolved to country code: {:?}", ip, country_code);
                Ok(country_code)
            }
            Err(e) => {
                error!("Failed to lookup IP {}: {}", ip, e);
                Err(Box::new(e))
            }
        }
    }
} 