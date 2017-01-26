#[macro_use] extern crate serde_derive;
extern crate helm_api;
extern crate serde_json;

mod concourse_api;

use std::env::args;
use std::collections::{
    HashMap,
};
use serde_json::Value;
use concourse_api::{
    CheckRequest,
    InRequest,
    InResponse,
    OutRequest,
    OutResponse,
    Version,
};
use helm_api::{
    Helm,
    Chart,
    Charts,
};

fn main() {
    match args().nth(1).as_ref().map(|s| s as &str) {
        Some("check") => request_check(),
        Some("in") => request_in(),
        Some("out") => request_out(),
        _ => panic!("Must provide either check, in, or out as first argument!"),
    }
}

fn request_check() {
    // get request from concourse
    let check_request: CheckRequest = concourse_api::receive_message().unwrap();

    // set up helm to connect to our cluster
    let helm = Helm::configure(check_request.source.into()).unwrap();

    // get a digest of the current state of installed packages
    let response = vec![Version {
        digest: helm.digest().unwrap(),
    }];

    // reply with a message
    concourse_api::send_message(&response).unwrap();
}

fn request_in() {
    // get request from concourse
    let in_request: InRequest = concourse_api::receive_message().unwrap();

    // set up helm to connect to our cluster
    let helm = Helm::configure(in_request.source.into()).unwrap();

    // get the list of deployed charts
    let deployed_charts = helm.list().unwrap();

    // get the digest
    let digest = helm.digest().unwrap();

    // reply with a message
    let response = InResponse {
        version: Version {
            digest: digest,
        },
        metadata: deployed_charts,
    };
    concourse_api::send_message(&response).unwrap();
}

fn request_out() {
    // get request from concourse
    let mut in_request: OutRequest<Params> = concourse_api::receive_message().unwrap();

    // set up helm to connect to our cluster
    let helm = Helm::configure(in_request.source.into()).unwrap();

    // get the list of deployed charts
    let deployed_charts = helm.list().unwrap();

    // morph the charts rep into a friendly format
    let target_charts: Charts = in_request.params.charts
        .drain()
        .map(|(k, v)| Chart {
            release: k,
            name: v.name,
            version: v.version,
            overrides: v.overrides,
        })
        .collect();

    // find which charts are deleted
    let removed_charts = deployed_charts.into_iter().filter(|chart| {
        !target_charts.iter().any(|c| c.release == chart.release)
    });

    // run upgrade for added, changed and unchanged charts.
    // this is because its hard to know what overrides were used
    // during the initial install, and what the current version is,
    // e.g. is it 'latest'?
    // upgrading a chart that is not installed will install it.
    for to_install in &target_charts {
        helm.upgrade(to_install).unwrap();
    }

    for deleted in removed_charts {
        helm.delete(&deleted.release).unwrap();
    }

    // send back a response
    // get the list of deployed charts
    let deployed_charts = helm.list().unwrap();

    // get the digest
    let digest = helm.digest().unwrap();

    // reply with a message
    let response = OutResponse {
        version: Version {
            digest: digest,
        },
        metadata: deployed_charts,
    };
    concourse_api::send_message(&response).unwrap();
}


#[derive(Deserialize)]
struct ChartSpec {
    name: String,
    version: Option<String>,
    overrides: Option<HashMap<String, Value>>,
}

#[derive(Deserialize)]
struct Params {
    charts: HashMap<String, ChartSpec>,
}

