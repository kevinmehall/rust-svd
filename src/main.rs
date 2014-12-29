#![feature(globs)]
#![feature(macro_rules)]

extern crate xml;

use std::io::{File, BufferedReader};
use std::default::Default;
use std::num::from_str_radix;

use xml::reader::EventReader;
use fromxml::{FromXml, XmlIter};

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
}

deriving_fromxml! {
    struct Peripheral {
        name: String,
        version:Option<String>,
        description:Option<String>,
        groupName:Option<String>,
        prependToName:Option<String>,
        baseAddress:Option<String>,
        access:Option<String>,
        interrupt:Option<Interrupt>,
        registers: Vec<Regs>,
    }
}

deriving_fromxml! {
    struct Interrupt {
        name: String,
        value:Option<uint>,
    }
}

deriving_fromxml! {
    enum Regs {
        register(Register),
        cluster(Cluster),
    }
}

deriving_fromxml! {
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
}

#[deriving(Default, Show)]
struct Cluster {
    name: String,
    registers: Vec<Register>,
}

impl ::fromxml::FromXml for Cluster {
    fn from_xml(iter: &mut ::fromxml::XmlIter) -> Result<Cluster, ()> {
        let mut obj = Cluster{..Default::default()};
        try!(iter.each_child(|iter| {
            match iter.tag_name() {
                    "name" => obj.name = try!(::fromxml::FromXml::from_xml(iter)),
                    "register" => obj.registers.push(try!(::fromxml::FromXml::from_xml(iter))),
                    _ => try!(iter.skip_node())
            }
            Ok(())
        }));
        Ok(obj)
    }
}

deriving_fromxml! {
    struct Field {
        name: String,
        description:Option<String>,
        bitOffset:Option<uint>,
        bitWidth:Option<uint>,
        bitRange:Option<String>,
        access:Option<String>,
        enumeratedValues: Vec<EnumeratedValue>,
    }
}

deriving_fromxml! {
    struct EnumeratedValue {
        name: String,
        value: String,
        description:Option<String>,
    }
}

deriving_fromxml! {
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
    let mut registers = vec![];

    println!("ioregs!({} = {{", peripheral.name);
    for register in peripheral.registers.iter() {
        match *register {
            Regs::register(ref r) => registers.push(r),
            Regs::cluster(ref c) => {
                let mut cluster_regs: Vec<_> = c.registers.iter().collect();

                println!("    // cluster: {}", c.name);
                write_registers(&mut *cluster_regs);
            }
        }
    }
    write_registers(&mut *registers);
    println!("}});")
}

fn write_registers(registers: &mut[&Register]) {
    registers.as_mut_slice().sort_by(|a, b| {
        let a = a.addressOffset.as_ref().map(|x| parse_num(x.as_slice()));
        let b = b.addressOffset.as_ref().map(|x| parse_num(x.as_slice()));
        a.cmp(&b)
    });

    for &r in registers.iter() {
        write_register(r);
    }
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
    if field.name == "RESERVED" { return }

    let (lsb, width) = if let Some(ref br) = field.bitRange {
        let split: Vec<&str> = br.as_slice().split(':').collect();
        assert_eq!(split.len(), 2);
        let end: uint = split[0].slice_from(1).parse().unwrap();
        let start: uint = split[1].slice_to(split[1].len()-1).parse().unwrap();
        (start, end-start+1)
    } else {
        (field.bitOffset.unwrap(), field.bitWidth.unwrap())
    };

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
            print!("             {} => {},", en.value, en.name);
            write_doc_comment(en.description.as_ref());
            print!("\n");
        }
        println!("         }}");
    }
}

fn main() {
    let args = std::os::args();
    let filename = args.get(1).expect("No SVD filename provided");
    let file = File::open(&Path::new(filename)).unwrap();
    let reader = BufferedReader::new(file);

    let mut parser = EventReader::new(reader);
    let mut iter = XmlIter::new(parser.events()).unwrap();
    let device: Device = FromXml::from_xml(&mut iter).unwrap();

    write_device(&device);
}
