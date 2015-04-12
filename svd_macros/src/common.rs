//! Peripheral macro.

#![allow(unused_variables)]
#![allow(dead_code)]

use fromxml::{parse_root};
use std::io::{File, BufferedReader};
use xml::reader::EventReader;
use ast;

use std::collections::VecMap;
use std::num::FromStrRadix;
use std::default::Default;

// mod fromxml;

derive_fromxml! {
    struct Device {
        vendor:Option<String>,
        vendorID:Option<String>,
        licenseText:Option<String>,
        series:Option<String>,
        version:Option<String>,
        description:Option<String>,
        access:Option<String>,
        cpu:Option<CPU>,
        peripherals:Option<Peripherals>,
    }

    struct Peripherals {
        peripheral:Vec<Peripheral>,
    }

    struct Peripheral {
        name:String,
        baseAddress:u32,
        derivedFrom:Option<String>,
        description:Option<String>,
        version:Option<String>,
        groupName:Option<String>,
        prependToName:Option<String>,
        access:Option<String>,
        interrupt:Option<Interrupt>,
        registers:Option<Registers>,
    }

    struct Registers {
        register:Vec<Register>,
        cluster:Vec<Cluster>,
    }

    struct Cluster {
        name:String,
        register:Vec<Register>,
    }

    struct Interrupt {
        name:String,
        value:usize,
    }

    struct Register {
        name:String,
        addressOffset:u32,
        dim:Option<u32>,
        dimIncrement:Option<u32>,
        description:Option<String>,
        size:Option<usize>,
        access:Option<String>,
        fields:Option<Fields>,
    }

    struct Fields {
        field:Vec<Field>,
    }

    struct Field {
        name:String,
        bitOffset:usize,
        bitWidth:usize,
        description:Option<String>,
        access:Option<String>,
        enumeratedValues:Option<EnumeratedValues>,
    }

    struct EnumeratedValues {
        enumeratedValue:Vec<EnumeratedValue>,
    }

    struct EnumeratedValue {
        name:String,
        value:String,
        description:Option<String>,
    }

    struct CPU {
        name:String,
        revision:String,
    }
}
    
pub fn load_file(input:&str) -> Device {
    let file = File::open(&Path::new(input)).unwrap();
    let reader = BufferedReader::new(file);

    let mut parser = EventReader::new(reader);
    let mut iter = parser.events();
    parse_root::<Device, BufferedReader<File>>(&mut iter).unwrap()
}

// Enums might be defined with leading numbers. Fix that.
pub fn make_nice_enum(val:&EnumeratedValue) -> String {
    let label = val.name.clone();
    match val.description {
        None => label,
        Some(ref desc) => {
            if !label.char_at(0).is_alphabetic() {
                let mut out = String::new();
                let mut camel:bool = true;
                for mut a in desc.chars() {
                    if out.len() == 0 && a.is_digit(16) {
                        out.push_str("USE");
                    }
                    if a == '-' {
                        a = '_';
                    }
                    if !a.is_whitespace() {
                        if camel {
                            out.push('_');
                        }
                        out.push(a.to_uppercase());
                        camel = false;
                    } else {
                        camel = true;
                    }
                }
                return out;
            }
            label
        }
    }
}

fn gen_regs(registers:&Vec<Register>) -> VecMap<ast::RegisterAst> {
    let mut regmap = VecMap::new();
    for register in registers.iter() {
        let dim = register.dim.unwrap_or(1);
        let dim_increment = register.dimIncrement.unwrap_or(0) as u32;
        for dim_i in range(0, dim) {
            let mut fieldmap = VecMap::new();

            if let Some(ref fields) = register.fields {
                for field in fields.field.iter() {
                    let mut enummapopt = None;
                    if let Some(ref enum_vals) = field.enumeratedValues {
                        let mut enummap = vec![];
                    
                        for val in enum_vals.enumeratedValue.iter() {
                            let printname = make_nice_enum(val);
                            let s = val.value.as_slice();
                            enummap.push((printname, if s.contains_char('x') {
                                FromStrRadix::from_str_radix(&*s.slice_from(2), 16)
                            } else {
                                s.parse()
                            }.unwrap()));
                        }

                        enummapopt = Some(enummap);
                    }

                    fieldmap.insert(field.bitOffset, ast::FieldAst {
                        name: field.name.clone(),
                        width: field.bitWidth,
                        enumerate: enummapopt,
                    });
                }
            }

            regmap.insert((register.addressOffset + (dim_increment * dim_i as u32)) as usize, ast::RegisterAst {
                name: register.name.replace("%s", dim_i.to_string().as_slice()),
                fields: fieldmap,
                width: *register.size.as_ref().unwrap_or(&0) / 8,
                dim: match register.dim {
                    Some(..) => Some((register.dim.unwrap(), register.dimIncrement.unwrap())),
                    _ => None,
                },
            });
        }
    }

    regmap
}

pub fn parse_xml(device:Device) -> Vec<ast::PeripheralAst> {
    let mut peripheral_asts = vec![];
    if let Some(ref peripherals) = device.peripherals {
        for peripheral in peripherals.peripheral.iter() {
            let regs = if let Some(ref derive) = peripheral.derivedFrom {
                match peripheral_asts.iter().find(|x:&&ast::PeripheralAst| x.name == derive.as_slice()) {
                    Some(derived) => {
                        derived.regs.clone()
                    }
                    None => {
                        panic!("Could not find peripheral '{}' derived by '{}'", derive, peripheral.name);
                    }
                }
            } else {
                if let Some(ref registers) = peripheral.registers {
                    let regmap = gen_regs(&registers.register);

                    if regmap.len() > 0 {
                        ast::RegList::Registers(regmap)
                    } else {
                        let mut clusters = vec![];
                        for cluster in registers.cluster.iter() {
                            clusters.push(ast::Cluster {
                                name: cluster.name.clone(),
                                registers: gen_regs(&cluster.register)
                            });
                        }
                        ast::RegList::Cluster(clusters)
                    }
                } else {
                    ast::RegList::Registers(Default::default())
                }
            };

            peripheral_asts.push(ast::PeripheralAst {
                name: peripheral.name.clone(),
                address: peripheral.baseAddress,
                regs: regs,
                derives: peripheral.derivedFrom.clone(),
            });
        }
    }
    peripheral_asts
}
