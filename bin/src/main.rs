#[macro_use] extern crate serde_derive;
extern crate helm_api;

mod concourse_api;

use std::hash::Hash;
use std::env::args;
use std::collections::{
    HashMap,
};
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
        })
        .collect();

    // get a diff of everything
    let chart_diff = diff(deployed_charts, target_charts, |c| c.release.to_string());

    for upgraded in chart_diff.changed {
        helm.upgrade(&upgraded).unwrap();
    }

    for deleted in chart_diff.removed {
        helm.delete(&deleted.release).unwrap();
    }

    for added in chart_diff.added {
        helm.install(&added).unwrap();
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
}

#[derive(Deserialize)]
struct Params {
    charts: HashMap<String, ChartSpec>,
}

#[derive(Debug)]
struct Diff<E> {
    added: Vec<E>,
    changed: Vec<E>,
    removed: Vec<E>,
}

fn diff<E, K, F>(initial: Vec<E>, next: Vec<E>, key: F) -> Diff<E>
where E: Eq + PartialEq,
      K: Hash + Eq,
      F: Fn(&E) -> K,
{
    let mut added = Vec::new();
    let mut changed = Vec::new();

    let mut original = HashMap::new();
    for item in initial {
        original.insert(key(&item), item);
    }

    for item in next {
        match original.remove(&key(&item)) {
            Some(ref old) if *old != item => changed.push(item),
            None => added.push(item),
            _ => continue,
        }
    }

    let removed = original.drain().map(|(_, v)| v).collect();

    Diff {
        added: added,
        changed: changed,
        removed: removed,
    }
}

