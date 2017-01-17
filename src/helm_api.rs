#![allow(dead_code)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Chart {
    pub release: String,
    pub name: String,
    pub version: Option<String>,
}

pub type Charts = Vec<Chart>;

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

    fn run(&self, cmd: &str) -> Result<String> {
        // log the command we're running
        try!(io::stderr().write(format!("Running `{}`.\n", cmd).as_bytes()));

        let output = try!(Command::new(BASH_PATH)
            .env("KUBECONFIG", KUBE_CONFIG_PATH)
            .arg("-c")
            .arg(cmd)
            .output());

        // log things to stderr since stdout is reserved
        try!(io::stderr().write(&output.stdout));
        try!(io::stderr().write(&output.stderr));
        try!(io::stderr().flush());

        if !output.status.success() {
            return Err(Error::new(ErrorKind::Other, format!("failed to run `{}`", cmd)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn digest(&self) -> Result<String> {
        self.run("helm list | md5sum | cut -d' ' -f 1")
    }

    pub fn list(&self) -> Result<Vec<Chart>> {
        let charts = self
            .run("helm list")
            .unwrap()
            .lines()
            .skip(1)
            .map(|line| {
                let tokens: Vec<&str> = line.split_whitespace().collect();
                let mut name_vers = tokens.last().unwrap().rsplitn(2, '-');

                Chart {
                    release: tokens.first().unwrap().to_string(),
                    version: Some(name_vers.next().unwrap().to_string()),
                    name: name_vers.last().unwrap().to_string(),
                }
            })
            .collect();

        Ok(charts)
    }

    pub fn upgrade(&self, chart: &Chart) -> Result<()> {
        let cmd = if let Some(ref version) = chart.version {
            format!("helm upgrade --install --version {} {} stable/{}",
                version, chart.release, chart.name)
        } else {
            format!("helm upgrade --install {} stable/{}", chart.release, chart.name)
        };
        self.run(&cmd).map(|_| { () })
    }

    pub fn install(&self, chart: &Chart) -> Result<()> {
        let cmd = if let Some(ref version) = chart.version {
            format!("helm install --replace --version {} --name {} stable/{}",
                version, chart.release, chart.name)
        } else {
            format!("helm install --replace --name {} stable/{}", chart.release, chart.name)
        };
        self.run(&cmd).map(|_| { () })
    }

    pub fn delete(&self, release: &str) -> Result<()> {
        let cmd = format!("helm delete {}", release);
        self.run(&cmd).map(|_| { () })
    }
}
