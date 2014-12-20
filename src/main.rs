#![feature(globs)]
#![feature(macro_rules)]

extern crate xml;

use std::io::{File, BufferedReader};
use std::default::Default;
use xml::reader::EventReader;
use fromxml::{parse_root};

mod fromxml;

deriving_fromxml! {
    struct Device {
        vendor:Option<String>,
        vendorID:Option<String>,
        licenseText:Option<String>,
        series:Option<String>,
        version:Option<String>,
        description:Option<String>,
        cpu:Option<CPU>,
        peripherals: Vec<Peripheral>,
    }

    struct Peripheral {
        name: String,
        version:Option<String>,
        description:Option<String>,
        groupName:Option<String>,
        prependToName:Option<String>,
        baseAddress:Option<String>,
        access:Option<String>,
        interrupt:Option<Interrupt>,
        registers: Vec<Register>,
    }

    struct Interrupt {
        name: String,
        value:Option<uint>,
    }

    struct Register {
        name: String,
        dim:Option<uint>,
        dimIncrement:Option<String>,
        description:Option<String>,
        addressOffset:Option<String>,
        size:Option<uint>,
        access:Option<String>,
        fields: Vec<Field>,
    }

    struct Field {
        name: String,
        description:Option<String>,
        bitOffset:Option<uint>,
        bitWidth:Option<uint>,
        access:Option<String>,
        enumeratedValues: Vec<EnumeratedValue>,
    }

    struct EnumeratedValue {
        name: String,
        value: String,
        description:Option<String>,
    }

    struct CPU {
        name:Option<String>,
        revision:Option<String>,
    }
}

fn main() {
    let file = File::open(&Path::new("file.xml")).unwrap();
    let reader = BufferedReader::new(file);

    let mut parser = EventReader::new(reader);
    let mut iter = parser.events();
    let device:Device = parse_root::<Device>(&mut iter).unwrap();

    println!("{}", device);
}
