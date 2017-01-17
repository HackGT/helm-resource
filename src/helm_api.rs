extern crate rustache;
extern crate serde;

use concourse_api::{
    Source,
};
use self::rustache::{
    HashBuilder,
    Render,
};
use std::io::{
    Result,
    Error,
    ErrorKind,
    Write,
    self,
};
use std::fs::File;
use std::process::Command;


const KUBE_CONFIG: &'static str = include_str!("../templates/kube-config.mo");
const KUBE_CONFIG_PATH: &'static str = "/tmp/kube-config";
const BASH_PATH: &'static str = "/bin/bash";

#[derive(Debug, Serialize)]
pub struct Chart {
    pub release: String,
    pub name: String,
    pub version: String,
}

pub struct Helm;

impl Helm {
    pub fn configure(config: Source) -> Result<Self> {
        // we'll store this config file for helm to use
        let mut file = try!(File::create(KUBE_CONFIG_PATH));

        // check invariants
        if config.ca_data.is_none() && !config.skip_tls_verify.unwrap_or(false) {
            return Err(Error::new(ErrorKind::Other,
                "must either set 'skip_tls_verify: true' or provide ca_data!"));
        }

        // generate k8s config file so helm can connect to our server
        if HashBuilder::new()
            .insert("skip_tls_verify", config.skip_tls_verify.unwrap_or(false))
            .insert("url", config.url)
            .insert("namespace", config.namespace)
            .insert("username", config.username)
            .insert("password", config.password)
            .insert("ca_data", config.ca_data.unwrap_or(String::new()))
            .render(KUBE_CONFIG, &mut file).is_err()
        {
            return Err(Error::new(ErrorKind::Other, "error populating kube config template"));
        }

        // make sure we wrote the file
        try!(file.flush());

        // init help
        let mut init_helm_ps = try!(Command::new(BASH_PATH)
            .env("KUBECONFIG", KUBE_CONFIG_PATH)
            .arg("-c")
            .arg("helm init --client-only 1>&2")
            .spawn());

        try!(init_helm_ps.wait());

        Ok(Helm)
    }

    pub fn get_digest(&self) -> Result<String> {
        let output = try!(Command::new(BASH_PATH)
            .env("KUBECONFIG", KUBE_CONFIG_PATH)
            .arg("-c")
            .arg("helm list | md5sum | cut -d' ' -f 1")
            .output());

        // log things to stderr since stdout is reserved
        try!(io::stderr().write(&output.stderr));

        if !output.status.success() {
            return Err(Error::new(ErrorKind::Other,
                "failed to run 'helm list' as part of the check step"));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn charts(&self) -> Result<Vec<Chart>> {
        let output = try!(Command::new(BASH_PATH)
            .env("KUBECONFIG", KUBE_CONFIG_PATH)
            .arg("-c")
            .arg("helm list")
            .output());

        let text_out = String::from_utf8_lossy(&output.stdout);

        Ok(text_out
            .lines()
            .skip(1)
            .map(|line| {
                let tokens: Vec<&str> = line.split_whitespace().collect();
                let mut name_vers = tokens.last().unwrap().rsplitn(2, '-');

                Chart {
                    release: tokens.first().unwrap().to_string(),
                    version: name_vers.next().unwrap().to_string(),
                    name: name_vers.last().unwrap().to_string(),
                }
            })
            .collect())
    }
}
