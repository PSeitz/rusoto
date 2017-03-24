#![cfg_attr(feature = "nightly-testing", feature(plugin))]
#![cfg_attr(feature = "nightly-testing", plugin(clippy))]

extern crate inflector;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate hyper;

#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use botocore::Service as BotocoreService;
use generator::generate_source;

pub mod botocore;
pub mod generator;
mod serialization;
mod util;

const BOTOCORE_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/botocore/botocore/data/");

pub struct Service {
    pub name: String,
    protocol_date: String,
}

impl Service {
    pub fn new<S>(name: S, protocol_date: S) -> Self
        where S: Into<String> {
        Service {
            name: name.into(),
            protocol_date: protocol_date.into(),
        }
    }
}

pub fn generate(service: Service, output_path: &Path) -> i32 {
    let destination_path = output_path.join(format!("{}.rs", service.name));
    let botocore_service_data_path = Path::new(BOTOCORE_DIR)
        .join(format!("{}/{}/service-2.json", service.name, service.protocol_date));

    botocore_generate(botocore_service_data_path.as_path(),
                                        destination_path.as_path());

    return 1;
}

fn botocore_generate(input_path: &Path, output_path: &Path) {
    let input_file = File::open(input_path).expect(&format!(
        "{:?} not found",
        input_path,
    ));

    let service_data_as_reader = BufReader::new(input_file);

    let mut service: BotocoreService = serde_json::from_reader(service_data_as_reader).expect(&format!(
        "Could not convert JSON in {:?} to Service",
        input_path,
    ));

    let service2: BotocoreService = serde_json::from_reader(BufReader::new(File::open(input_path).unwrap())).expect(&format!(
        "Could not convert JSON in {:?} to Service",
        input_path,
    ));

    for (shape_name, ref mut shape) in &mut service.shapes {
        if shape.location_name.is_none() {
            for (_, shape2) in &service2.shapes {
                if shape2.members.is_none(){continue;}
                for (_, ref member) in shape2.members.as_ref().unwrap() {
                    if &member.shape == shape_name{
                        shape.location_name = member.location_name.clone();
                    }
                }
            }
        }
    }

    match generate_source(&service, output_path) {
        Ok(()) => {},
        _ => panic!("Failed to write file at {:?}", output_path)
    }
}
