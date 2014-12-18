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
        peripherals:Option<Vec<Peripheral>>,
    }

    struct Peripheral {
        name:Option<String>,
        version:Option<String>,
        description:Option<String>,
        groupName:Option<String>,
        prependToName:Option<String>,
        baseAddress:Option<String>,
        access:Option<String>,
        interrupt:Option<Interrupt>,
        registers:Option<Vec<Register>>,
    }

    struct Interrupt {
        name:Option<String>,
        value:Option<uint>,
    }

    struct Register {
        dim:Option<uint>,
        dimIncrement:Option<uint>,
        name:Option<String>,
        description:Option<String>,
        addressOffset:Option<uint>,
        size:Option<uint>,
        access:Option<String>,
        fields:Option<Vec<Field>>,
    }

    struct Field {
        name:Option<String>,
        description:Option<String>,
        bitOffset:Option<uint>,
        bitWidth:Option<uint>,
        access:Option<String>,
        enumeratedValues:Option<Vec<EnumeratedValue>>,
    }

    struct EnumeratedValue {
        name:Option<String>,
        description:Option<String>,
        value:Option<String>,
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
