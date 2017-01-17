#[macro_use]
extern crate serde_derive;

#[path="../concourse_api.rs"]
mod concourse_api;
#[path="../helm_api.rs"]
mod helm_api;

use concourse_api::{
    InRequest,
    InResponse,
    Version,
};
use helm_api::{
    Helm,
};

fn main() {
    // get request from concourse
    let in_request: InRequest = concourse_api::receive_message().unwrap();

    // set up helm to connect to our cluster
    let helm = Helm::configure(in_request.source).unwrap();

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
