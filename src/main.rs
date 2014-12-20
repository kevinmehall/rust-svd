#![feature(globs)]
#![feature(macro_rules)]

extern crate xml;

use std::io::{File, BufferedReader};
use std::default::Default;
use std::num::from_str_radix;

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

fn parse_num(s: &str) -> Option<u32> {
    if s.slice_to(2) == "0x" {
        from_str_radix(s.slice_from(2), 16)
    } else {
        from_str_radix(s, 10)
    }
}

fn write_doc_comment(doc: Option<&String>) {
    if let Some(d) = doc {
        print!(" //= {}", d);
    }
}

fn write_device(device: &Device) {
    for peripheral in device.peripherals.iter() {
        write_peripheral(peripheral);
    }
}

fn write_peripheral(peripheral: &Peripheral) {
    let mut registers: Vec<_> = peripheral.registers.iter().collect();

    registers.as_mut_slice().sort_by(|a, b| {
        let a = a.addressOffset.as_ref().map(|x| parse_num(x.as_slice()));
        let b = b.addressOffset.as_ref().map(|x| parse_num(x.as_slice()));
        a.cmp(&b)
    });

    println!("ioregs!({} = {{", peripheral.name);
    for &register in registers.iter() {
        write_register(register);
    }
    println!("}}")
}

fn write_register(register: &Register) {
    if let (Some(dim), Some(dim_increment)) = (register.dim, register.dimIncrement.as_ref()) {
        println!("    // repeat: {} increment {}", dim, dim_increment);
    }
    let offset = register.addressOffset.as_ref().map_or("", |x| x.as_slice());
    let size = register.size.unwrap_or(0);
    print!("    {} => reg{} {} {{", offset, size, register.name);
    write_doc_comment(register.description.as_ref());
    print!("\n");

    for field in register.fields.iter() {
        write_field(field);
    }
    println!("    }}");
}

fn write_field(field: &Field) {
    let lsb = field.bitOffset.unwrap();
    let width = field.bitWidth.unwrap();

    print!("         {}", lsb);

    if width > 1 {
        print!("..{}", lsb + width - 1);
    }

    print!(" => {}", field.name);

    if field.enumeratedValues.len() == 0 {
        print!(",");
    } else {
        print!(" {{");
    }

    write_doc_comment(field.description.as_ref());
    print!("\n");

    if field.enumeratedValues.len() != 0 {
        for en in field.enumeratedValues.iter() {
            if en.name.len() == 0 { continue } // TODO: this is a <name>, not an <enumeratedValues>
            print!("             {} => {}", en.name, en.value);
            write_doc_comment(en.description.as_ref());
            print!("\n");
        }
        println!("         }}");
    }
}

fn main() {
    let file = File::open(&Path::new("file.xml")).unwrap();
    let reader = BufferedReader::new(file);

    let mut parser = EventReader::new(reader);
    let mut iter = parser.events();
    let device: Device = parse_root(&mut iter).unwrap();

    write_device(&device);
}
