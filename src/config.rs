use std::{collections::HashMap, error::Error, fs::File, io::BufReader, path};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct BucketConfig {
    pub bucket: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub s3_region: String,
    pub s3_endpoint: String,
    pub sites: HashMap<String, BucketConfig>,
}

pub fn read_config<P: AsRef<path::Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let config = serde_json::from_reader(reader)?;

    Ok(config)
}
