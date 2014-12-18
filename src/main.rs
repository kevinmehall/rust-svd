#![feature(globs)]
#![feature(macro_rules)]

extern crate xml;

use std::io::{File, BufferedReader};
use std::any::{Any, AnyMutRefExt};
use std::default::Default;

use xml::reader::EventReader;
use xml::reader::Events;
use xml::reader::events::XmlEvent::*;

fn indent(size: uint) -> String {
    let mut result = String::with_capacity(size*4);
    for _ in range(0, size) {
        result.push_str("    ");
    }
    result
}

#[macro_export]
macro_rules! deriving_fromxml {
    (
        $($(#[$attr:meta])* struct $Id:ident {
            $($(#[$Flag_field:meta])* $Flag:ident:$T:ty),+,
        })+
    ) => {
        $($(#[$attr])*
        #[deriving(Default, Show)]
        struct $Id {
            $($(#[$Flag_field])* $Flag:$T,)+
        }

        impl FromXml for $Id {
			fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<$Id> {
				let mut obj = $Id { ..Default::default() };

				fn inner<'a> (iter:&'a mut XmlIter, arg:&mut $Id, name:&str) {
			        match name {
			        	$($(#[$Flag_field])* stringify!($Flag) => arg.$Flag  = FromXml::from_xml(iter),)+
			        	_ => skip_node(iter),
			        };
				}

			    return collect(iter, obj, inner);
			}
        })+
    };
}

deriving_fromxml! {
	struct Device  {
		vendor:Option<String>,
		vendorID:Option<String>,
		licenseText:Option<String>,
		cpu:Option<CPU>,
	}
	
	struct CPU {
		name:Option<String>,
		revision:Option<String>,
	}
}

type XmlIter<'a> = Events<'a, BufferedReader<File>>;

fn collect<'a, T>(iter:&'a mut XmlIter, mut arg:T, back:for<'b> fn(&'b mut XmlIter, &mut T, &str)) -> Option<T> {
	loop {
    	let next = iter.next();
    	if let Some(e) = next {
	        match e {
	            StartElement { name, attributes, namespace } => {
	            	back(iter, &mut arg, name.local_name.as_slice());
	            },
	            EndElement { name } => {
	            	break;
	            },
	            Error(e) => {
	                println!("Error: {}", e);
	                return None;
	            }
	            _ => {}
	        }
    	} else {
    		return None;
    	}
    }
    return Some(arg);
}

fn skip_node<'a>(iter:&'a mut XmlIter) {
	let mut depth:uint = 1;
	loop {
    	let next = iter.next();
    	if let Some(e) = next {
	        match e {
	            StartElement { name, attributes, namespace } => {
	            	depth = depth + 1;
	            },
	            EndElement { name } => {
	            	depth = depth - 1;
	            	if depth == 0 {
	            		return;
	            	}
	            	break;
	            },
	            Error(e) => {
	                println!("Error: {}", e);
	                return;
	            }
	            _ => {}
	        }
    	} else {
    		return;
    	}
    }
}

trait FromXml {
	fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<Self>;
}

impl FromXml for String {
	fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<String> {
		let mut str = "".to_string();
		loop {
	    	let next = iter.next();
	    	if let Some(e) = next {
		        match e {
		        	Characters(text) => {
		        		str.push_str(text.as_slice());
		        	},
		        	_ => {
		        		break;
		        	}
		        }
		    } else {
		    	return None;
		    }
		}
		return Some(str);
	}
}

fn main() {
    let file = File::open(&Path::new("file.xml")).unwrap();
    let reader = BufferedReader::new(file);

    let mut stack:Vec<String> = vec![];

    let mut device = Device { ..Default::default() };

    let mut parser = EventReader::new(reader);
    let mut iter = parser.events();
    loop {
    	let next = iter.next();
    	if let Some(e) = next {
	        match e {
	            StartElement { name, attributes, namespace } => {
	            	stack.push(name.local_name.clone());
	                // println!("{}{}/", indent(stack.len() - 1), name);

	                if name.local_name == "device" {
	                	device = FromXml::from_xml(&mut iter).unwrap();
	                }
	            },
	            Characters(text) => {
	            },
	            EndElement { name } => {
	            	stack.pop();
	                // println!("{}/{}", indent(stack.len()), name);
	            },
	            Error(e) => {
	                // println!("Error: {}", e);
	                break;
	            }
	            _ => {}
	        }
    	} else {
    		break;
    	}
    }

    println!("{}", device);
}

fn start_event(list:&Vec<String>, text:String) {
}

fn text_event(list:&Vec<String>, text:String) {
	if list[list.len() - 1] == "name" {
		if list[list.len() - 2] == "peripheral" {
			println!("{} {{", text);
		}
	}
	if list[list.len() - 1] == "baseAddress" {
		// println!("base = {},", text);
	}
	if list[list.len() - 1] == "addressOffset" {
		print!("{} => reg", text);
	}
	if list[list.len() - 1] == "size" {
		println!("{} {{", text);
	}
}

fn end_event(list:&Vec<String>, text:String) {
	if text == "peripheral" {
		println!("}}");
		println!("");
	}
}
