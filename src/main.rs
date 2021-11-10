use futures::prelude::*;
use kube::{
    runtime::watcher::Event,
    api::{Api, ListParams},
    runtime::{utils::try_flatten_applied, watcher},
    Client,
};
use simple_mdns::ServiceDiscovery;
use std::net::SocketAddr;
use std::thread;

use std::net::IpAddr;

use std::collections::HashMap;
use k8s_openapi::{Metadata, api::networking::v1::Ingress};


fn setup_mdns(hostname: String, ip:IpAddr) {
    println!("hostname: {}", hostname.clone());

    // let mut discovery = ServiceDiscovery::new(&hostname.clone(), 80).expect("Invalid Service Name");
    // discovery.add_ip_address(ip);
}


fn on_applied (ingress: Ingress, ip: IpAddr,  reg: &mut ServiceRegistry) {
    println!("Ingress created: {:?}", &ingress.metadata.name);

    if let Some(spec) = ingress.spec {
        if let Some(rules) = spec.rules{
            for rule in rules {
                // Start mDNS responder if ends with .local
                if let Some(hostname) = rule.host {
                    if hostname.ends_with(".local") {

                        let key = &hostname.clone();

                        let mut sd = reg.discovery_for(key);
                        sd.add_ip_address(ip);
                    }
                }
            }
        }
    }
}

fn on_delete (ingress: Ingress, ip: IpAddr, reg: &mut ServiceRegistry) {
    println!("Ingress deleted: {:?}", &ingress.metadata.name);

    if let Some(spec) = ingress.spec {
        if let Some(rules) = spec.rules{
            for rule in rules {
                // Stop mDNS responder
                if let Some(hostname) = rule.host {
                    if hostname.ends_with(".local") {
                        //setup_mdns( hostname, ip);
                    }

                }
            }
        }
    }
}

#[derive (Default)]
struct ServiceRegistry {
    state: HashMap<String, ServiceDiscovery>
}

impl ServiceRegistry {

    fn new() ->Self{
        Self::default()
    }

    fn discovery_for(&mut self, hostname: &'static str) -> &mut ServiceDiscovery {
        let key = hostname.to_owned();

        if ! self.state.contains_key(&key) {
            let sd = ServiceDiscovery::new( hostname, 80).expect("Invalid Service Name");
            self.state.insert(key.clone(), sd);
        }
        
        let mut ret = self.state.get(&key).unwrap();
        &mut ret
    }

}


#[tokio::main]
async fn main() -> anyhow::Result<()> {


    let mut host_registry = ServiceRegistry::new();

    let client = Client::try_default().await?;

    let ing_list: Api<Ingress> = Api::all(client);

    let mut watcher = watcher(ing_list, ListParams::default()).boxed();

    let ip = "192.168.1.11".parse().unwrap();


    while let Some(event) = watcher.try_next().await? {
        match event {
            Event::Applied(ingress) => on_applied(ingress, ip, &mut host_registry),
            Event::Deleted(ingress) => on_delete(ingress, ip, &mut host_registry),
            _ => {},
        }
    }

    Ok(())
}
