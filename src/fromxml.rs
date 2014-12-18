#![macro_escape]

use std::io::{File, BufferedReader};

use xml::reader::Events;
use xml::reader::events::XmlEvent::*;

#[macro_export]
macro_rules! deriving_fromxml {
    (
        $($(#[$attr:meta])* struct $Id:ident {
            $($(#[$Flag_field:meta])* $Flag:ident:$T:ty),+,
        })+
    ) => {
        $($(#[$attr])*
        #[deriving(Default, Show)]
        #[allow(non_snake_case)]
        struct $Id {
            $($(#[$Flag_field])* $Flag:$T,)+
        }

        impl ::fromxml::FromXml for $Id {
            fn from_xml<'a>(iter:&'a mut ::xml::reader::Events<::std::io::BufferedReader<::std::io::File>>) -> Option<$Id> {
                let obj = $Id { ..Default::default() };

                fn inner<'a> (iter:&'a mut ::xml::reader::Events<::std::io::BufferedReader<::std::io::File>>, arg:&mut $Id, name:&str) {
                    match name {
                        $($(#[$Flag_field])* stringify!($Flag) => arg.$Flag  = ::fromxml::FromXml::from_xml(iter),)+
                        _ => ::fromxml::skip_node(iter),
                    };
                }

                return ::fromxml::collect(iter, obj, inner);
            }
        })+
    };
}

type XmlIter<'a> = Events<'a, BufferedReader<File>>;

pub fn collect<'a, T>(iter:&'a mut XmlIter, mut arg:T, back:for<'b> fn(&'b mut XmlIter, &mut T, &str)) -> Option<T> {
    loop {
        let next = iter.next();
        if let Some(e) = next {
            match e {
                StartElement { name, attributes: _, namespace: _ } => {
                    back(iter, &mut arg, name.local_name.as_slice());
                },
                EndElement { name: _ } => {
                    break;
                },
                Error(e) => {
                    println!("Error: {}", e);
                    return None;
                }
                _ => {},
            }
        } else {
            return None;
        }
    }
    return Some(arg);
}

pub fn skip_node<'a>(iter:&'a mut XmlIter) {
    let mut depth:uint = 1;
    loop {
        let next = iter.next();
        if let Some(e) = next {
            match e {
                StartElement { name: _, attributes: _, namespace: _ } => {
                    depth = depth + 1;
                },
                EndElement { name: _ } => {
                    depth = depth - 1;
                    if depth == 0 {
                        return;
                    }
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

pub trait FromXml {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<Self>;
}

impl<T:FromXml> FromXml for Vec<T> {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<Vec<T>> {
        let mut ret:Vec<T> = vec![];
        loop {
            let next = iter.next();
            if let Some(e) = next {
                match e {
                    StartElement { name: _, attributes: _, namespace: _ } => {
                        ret.push(FromXml::from_xml(iter).unwrap());
                    },
                    EndElement { name: _ } => {
                        break;
                    },
                    _ => (),
                }
            } else {
                return None;
            }
        }
        return Some(ret);
    }
}

impl FromXml for uint {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<uint> {
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
        return from_str(str.as_slice());
    }
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

pub fn parse_root<'a, T:FromXml>(iter:&'a mut XmlIter) -> Option<T> {
    loop {
        let next = iter.next();
        if let Some(e) = next {
            match e {
                StartElement { name: _, attributes: _, namespace: _ } => {
                    return FromXml::from_xml(iter);
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
}
