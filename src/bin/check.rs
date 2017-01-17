#[macro_use]
extern crate serde_derive;

#[path="../concourse_api.rs"]
mod concourse_api;
#[path="../helm_api.rs"]
mod helm_api;

use concourse_api::{
    CheckRequest,
    Version,
};
use helm_api::Helm;

fn main() {
    // get request from concourse
    let check_request: CheckRequest = concourse_api::receive_message().unwrap();

    // set up helm to connect to our cluster
    let helm = Helm::configure(check_request.source).unwrap();

    // get a digest of the current state of installed packages
    let response = vec![Version {
        digest: helm.digest().unwrap(),
    }];

    // reply with a message
    concourse_api::send_message(&response).unwrap();
}
