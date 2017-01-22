#[macro_use] extern crate serde_derive;
extern crate helm_api;

#[path="../concourse_api.rs"]
mod concourse_api;

use std::collections::{
    HashMap,
};
use std::hash::Hash;
use concourse_api::{
    OutRequest,
    OutResponse,
    Version,
};
use helm_api::{
    Helm,
    Chart,
    Charts,
};

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

fn main() {
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
