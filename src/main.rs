extern crate rusoto_core;
extern crate rusoto_ec2;
#[macro_use]
extern crate prettytable;
extern crate clap;

use clap::{Arg, App, SubCommand};

mod simple_ec2 {
    use rusoto_core::Region;
    use rusoto_ec2::{Ec2, Ec2Client};

    pub struct Instance {
        name: Option<String>,
        id: String,
        instance_type: String,
        pri_ip: String,
        pub_ip: Option<String>,
        state: String,
    }

    impl Instance {
        /// Starts the instance.
        pub fn start(&self) -> rusoto_ec2::InstanceStateChange {
            let ec2 = Ec2Client::new(Region::default());
            let mut req = rusoto_ec2::StartInstancesRequest::default();
            req.instance_ids = vec![self.id.clone()];
            ec2
                .start_instances(req)
                .sync()
                .expect("Failed to start instances")
                .starting_instances
                .unwrap()[0].clone()
        }
    
        /// Stops the instance.
        pub fn stop(&self) -> rusoto_ec2::InstanceStateChange {
            let ec2 = Ec2Client::new(Region::default());
            let mut req = rusoto_ec2::StopInstancesRequest::default();
            req.instance_ids = vec![self.id.clone()];
            ec2
                .stop_instances(req)
                .sync()
                .expect("Failed to stop instances")
                .stopping_instances
                .unwrap()[0].clone()
        }
    
        // TODO: implement the function pull, which pulls/updates the instance profile from AWS
        // fn pull(&mut self) 
    }

    pub fn list() -> Vec<Instance> {
        let ec2 = Ec2Client::new(Region::default());
        let res = ec2
            .describe_instances(rusoto_ec2::DescribeInstancesRequest::default())
            .sync()
            .expect("Failed to list instances");

        let mut ret = Vec::new();

        if let Some(reservations) = res.reservations {
            for reserv in reservations {
                if let Some(instances) = reserv.instances {
                    for ins in instances {
                        ret.push(Instance {
                            name: || -> Option<String> {
                                if let Some(tags) = ins.tags.as_ref() {
                                    for tag in tags {
                                        if tag.key == Some("Name".to_string()) {
                                            return Some(tag.value.as_ref().unwrap().clone())
                                        }
                                    }
                                }
                                None
                            }(),
                            id: ins.instance_id.unwrap(),
                            instance_type: ins.instance_type.unwrap(),
                            state: ins.state.unwrap().name.unwrap(),
                            pri_ip: ins.private_ip_address.unwrap_or("".to_string()), 
                            pub_ip: ins.public_ip_address,
                        })
                    }
                }
            }
        }

        return ret;
    }

    pub fn print_instances(instances: &[Instance]) {
        let mut table = prettytable::Table::new();
        println!("Region: {}", Region::default().name());
        table.add_row(row!["Name", "ID", "Type", "Private IP", "Public IP", "State"]);

        for i in instances {
            table.add_row(row![i.name.clone().unwrap_or(String::default()), i.id, i.instance_type, i.pri_ip, i.pub_ip.clone().unwrap_or(String::default()), i.state]);
        }

        table.printstd();
    }

    pub fn print_instance_state_change(sc: rusoto_ec2::InstanceStateChange) {
        let prev = sc.previous_state.unwrap().name.unwrap();
        let curr = sc.current_state.unwrap().name.unwrap();
        println!("{} -> {}", prev, curr);
    }

    pub fn find_instance_by_name<'a>(instances: &'a [Instance], name: &str) -> Option<&'a Instance> {
        for i in instances {
            if i.name == Some(name.to_string()) {
                return Some(i);
            }
        }
        return None;
    }

    #[test]
    fn test_list() {
        let instances = list();
        // presume the test account has instances
        assert!(instances.len() > 0);
    }
}


fn main() {
    let matches = App::new("ah")
        .about("The AWS Cli Helper")
        .subcommand(SubCommand::with_name("ls")
            .about("Lists all instances"))
        .subcommand(SubCommand::with_name("start") 
            .about("Start the instance")
            .arg(Arg::with_name("NAME")
                .required(true)))
        .subcommand(SubCommand::with_name("stop")
            .about("Stop the instance")
            .arg(Arg::with_name("NAME")
                .required(true)))
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("ls") {
        let instances = simple_ec2::list();
        simple_ec2::print_instances(&instances);
    }

    if let Some(matches) = matches.subcommand_matches("start") {
        let name = matches.value_of("NAME").unwrap();
        let instances = simple_ec2::list();
        let i = simple_ec2::find_instance_by_name(&instances, name)
            .expect(&format!("Cannot find the instance {}", name));
        simple_ec2::print_instance_state_change(i.start());

    }

    if let Some(matches) = matches.subcommand_matches("stop") {
        let name = matches.value_of("NAME").unwrap();
        let instances = simple_ec2::list();
        let i = simple_ec2::find_instance_by_name(&instances, name)
            .expect(&format!("Cannot find the instance {}", name));
        simple_ec2::print_instance_state_change(i.stop());
    }
}
