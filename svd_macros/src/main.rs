#![feature(globs)]
#![feature(macro_rules)]
#![feature(phase)]
#![feature(plugin_registrar, quote)]

extern crate serialize;
extern crate xml;
extern crate getopts;
extern crate syntax;
#[phase(plugin, link)] extern crate fromxml;

use std::os;
use getopts::{optflag,getopts,OptGroup};
use common::{load_file, Device, Register, make_nice_enum, parse_xml};

mod common;
mod ast;

fn access(label:&str) -> &str {
    match label {
        "read-only" => "r",
        "write-only" => "w",
        "read-write" | _ => "rw",
    }
}

fn print_usage(program: &str, _opts: &[OptGroup]) {
    println!("Usage: {} [options] [svd]", program);
    println!("-s --symbols\t\tGenerate symbol definitions");
}

fn main() {
    let args: Vec<String> = os::args();
    let program = args[0].clone();

    let opts = &[
        optflag("s", "symbols", "generate symbol definitions"),
        optflag("h", "help", "print this help menu")
    ];
    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(program.as_slice(), opts);
        return;
    }
    let symbols = matches.opt_present("s");
    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(program.as_slice(), opts);
        return;
    };

    let device = load_file(input.as_slice());
    if symbols {
        dump_symbols(parse_xml(device));
    } else {
        dump_macro(device);
    }
}

fn dump_symbols(peripherals:Vec<ast::PeripheralAst>) {
    for peripheral in peripherals.iter() {
        println!("\"SAMD21_{}\" = 0x{:x};", peripheral.name, peripheral.address);
        if let ast::RegList::Cluster(ref clusters) = peripheral.regs {
            for cluster in clusters.iter() {
                println!("\"SAMD21_{}_{}\" = 0x{:x};", peripheral.name, cluster.name, peripheral.address);
            }
        }
    }
}

fn dump_macro(device:Device) {
    if let Some(ref peripherals) = device.peripherals {
        for peripheral in peripherals.peripheral.iter() {
            println!("/// {}", peripheral.description);
            println!("peripheral {} {{",
                peripheral.name,
                // peripheral.baseAddress.clone().unwrap());
                );

            if let Some(ref registers) = peripheral.registers {
                let mut sortedregs:Vec<&Register> = registers.register.iter().collect();
                // println!("{}", sortedregs);
                sortedregs.sort_by(|a, b| a.addressOffset.cmp(&b.addressOffset));
                for register in sortedregs.iter() {
                    if let Some(ref description) = register.description {
                        println!("  /// {}", description);
                    }
                    println!("  0x{:x} => reg{} {} {{",
                        // access(register.access.as_ref().or(device.access.as_ref()).unwrap().as_slice()),
                        register.addressOffset,
                        *register.size.as_ref().unwrap_or(&0),
                        register.name);

                    if let Some(ref fields) = register.fields {
                        for field in fields.field.iter() {
                            if let Some(desc) = field.description.as_ref() {
                                println!("      /// {}", desc);
                            }
                            print!("    {} => {}[{}] {}",
                                field.bitOffset,
                                access(field.access.as_ref().or(device.access.as_ref()).unwrap().as_slice()),
                                field.bitWidth,
                                field.name);
                            if let Some(enum_vals) = field.enumeratedValues.as_ref() {
                                println!(" {{");

                                for val in enum_vals.enumeratedValue.iter() {
                                    if let Some(desc) = val.description.as_ref() {
                                        println!("      /// {}", desc);
                                    }
                                    println!("      {} = {},", make_nice_enum(val), val.value);
                                }
                                print!("    }}")
                            }
                            println!(",");
                        }
                    }
                    println!("  }}");
                    println!("");
                    //fields.as_ref().unwrap_or(&vec![]).len());
                }
            }

            println!("}}");
            println!("");
        }
    }
}
