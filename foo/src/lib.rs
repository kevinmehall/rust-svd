#![feature(phase, plugin_registrar, macro_rules, quote)]

#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate syntax;
extern crate rustc;

use std::iter::Peekable;
use std::slice::Items;
use std::rc::Rc;
use std::num::{FromStrRadix};
use std::collections::VecMap;
use std::iter::Map;

use syntax::ast;
use syntax::ast::Delimited;
use syntax::ast::TokenTree::{TtToken, TtDelimited};
use syntax::codemap;
use syntax::parse::token::Token;
use syntax::parse::token::Lit;
use syntax::parse::token::{str_to_ident};
use syntax::ext::base::{ExtCtxt, ItemModifier, DummyResult, MacResult, MacExpr, MacItems};
// NB. this is important or the method calls don't work
use syntax::parse::token;
use syntax::util::small_vector::SmallVector;

pub mod regs;
pub mod util;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut rustc::plugin::Registry) {
    reg.register_macro("foo", expand);
}

type AstIterator<'a> = Peekable<&'a ast::TokenTree, Items<'a, ast::TokenTree>>;

fn is_ident(t:Option<&&ast::TokenTree>) -> bool {
	match t {
		Some(&&TtToken(_, Token::Ident(..))) => true,
		_ => false,
	}
}

fn read_ident<'a>(iter:&mut AstIterator<'a>) -> Result<String, String> {
	match iter.peek() {
		Some(&&TtToken(_, Token::Ident(value, kind))) => {
			iter.next();
			Ok(value.as_str().to_string())
		},
		_ => {
			Err(format!("expecting identifier"))
		}
	}
}

fn read_brackets<'a>(iter:&mut AstIterator<'a>) -> Result<Rc<Delimited>, String> {
	match iter.peek() {
		Some(&&TtDelimited(_, ref treeptr)) => {
			iter.next();
			Ok(treeptr.clone())
		},
		_ => {
			Err(format!("expecting brackets"))
		}
	}
}

fn read_tree<'a>(item:&'a Delimited) -> AstIterator<'a> {
	item.tts.iter().peekable()
}

fn is_int(t:Option<&&ast::TokenTree>) -> bool {
	match t {
		Some(&&TtToken(_, Token::Literal(Lit::Integer(..), _))) => true,
		_ => false,
	}
}

fn read_int<'a>(iter:&mut AstIterator<'a>) -> Result<int, String> {
	match iter.peek() {
		Some(&&TtToken(_, Token::Literal(Lit::Integer(value), kind))) => {
			iter.next();
			match if value.as_str().contains_char('x') {
                FromStrRadix::from_str_radix(value.as_str().slice_from(2), 16)
            } else {
                from_str(value.as_str())
            } {
				Some(value) => Ok(value),
				None => Err(format!("invalid integer")),
			}
		},
		_ => {
			Err(format!("expecting integer"))
		}
	}
}

fn is_token(t:&Token, u:&Token) -> bool {
	match (t, u) {
		(&Token::FatArrow, &Token::FatArrow) |
		(&Token::Comma, &Token::Comma) => {
			true
		}
		_ => false
	}
}

fn read_token<'a>(iter:&mut AstIterator<'a>, t:Token) -> Result<(), String> {
	match iter.peek() {
		Some(&&TtToken(_, ref u)) => {
			if is_token(&t, u) {
				iter.next();
				return Ok(())
			}
		}
		_ => {}
	}
	Err(format!("expecting simple token {}", t))
}

#[deriving(Show)]
struct Field {
	name:String,
	width:uint,
}

#[deriving(Show)]
struct RegisterAst {
	name:String,
	fields:VecMap<Field>,
}

fn parse_field<'a>(iter:&mut AstIterator<'a>) -> Result<Field, String> {
	let access = try!(read_ident_match(iter, &["rw", "r", "w"]));
	let delimiter = try!(read_brackets(iter));
	let sub = &mut read_tree(&*delimiter);
	let subscript = try!(read_int(sub));
	let name = try!(read_ident(iter));

	// println!("{} {} {}", access, subscript, name);

	let a = Field {
		name: name,
		width: subscript as uint,
	};

	return Ok(a);
}

fn parse_fields<'a>(iter:&mut AstIterator<'a>) -> Result<VecMap<Field>, String> {
	let mut out = VecMap::new();
	while is_int(iter.peek()) {
		let pos = try!(read_int(iter)) as uint;
		try!(read_token(iter, Token::FatArrow));
		out.insert(pos, try!(parse_field(iter)));
		match read_token(iter, Token::Comma) {
			Ok(..) => (),
			Err(..) => { break }
		}
	}
	Ok(out)
}

fn read_ident_match<'a>(iter:&mut AstIterator<'a>, select:&[&str]) -> Result<String, String> {
	let ident = try!(read_ident(iter));
	match select.iter().position(|p| &ident.as_slice() == p) {
		None => Err(format!("unexpected identifier {}", ident)),
		_ => Ok(ident),
	}
}

fn parse_reg<'a>(iter:&mut AstIterator<'a>) -> Result<RegisterAst, String> {
	try!(read_ident_match(iter, &["reg"]));
	let name = try!(read_ident(iter));
	let delimiter = try!(read_brackets(iter));
	let sub = &mut read_tree(&*delimiter);

	let fields = try!(parse_fields(sub));
	
	Ok(RegisterAst {
		name: name,
		fields: fields,
	})
}

fn expand<'a>(cx: &'a mut ExtCtxt, sp: codemap::Span, tokens: &[ast::TokenTree]) -> Box<MacResult + 'a> {
	let mut result = vec![];

	// cx.span_err(sp, "dummy is only permissible on functions");

	let mut iter = tokens.iter().peekable();

    // for i in tokens.iter() {
    // 	println!("{}", i);
    // }

    while !iter.is_empty() {
	    match parse_reg(&mut iter) {
	    	Err(err) => cx.span_err(sp, err.as_slice()),
	    	Ok(reg) => {
	    		// println!("wow {}", reg);
	    		let mut const_fields:Vec<ast::TokenTree> = vec![];
	    		for (pos, field) in reg.fields.iter() {
					// println!("{}", field);
					let n = str_to_ident(field.name.as_slice());
					let width = field.width;
					const_fields.push_all(quote_tokens!(cx, const $n: ::foo::regs::RegField = ::foo::regs::RegField { width: $width };).as_slice());
				}

				let name = str_to_ident(reg.name.as_slice());
				let name_update = str_to_ident((reg.name + "Update").as_slice());

				let struct_def = quote_tokens!(cx,
#[deriving(Show, Copy)]
#[repr(C)]
pub struct Reg {
    pub field:u32,
}

pub const INIT:Reg = Reg { field: 0 };

				);

				let struct_update_def = quote_tokens!(cx, 
#[deriving(Show)]
pub struct Update {
    pub origin:&'static mut Reg,
    pub diff:(uint, uint),
}
				);

				let struct_update_drop = quote_tokens!(cx,
impl Drop for Update {
    fn drop(&mut self) {
        self.origin.modify(self.diff);
    }
}
				);

				let mut fields:Vec<ast::TokenTree> = vec![];
				for (pos, field) in reg.fields.iter() {
					let lowerenable = field.name.chars().map(|a| a.to_lowercase()).collect::<String>();
					let field_name = str_to_ident(field.name.as_slice());
					let set_field_name = str_to_ident(vec!["set_", lowerenable.as_slice()].concat().as_slice());
					fields.push_all(quote_tokens!(cx, 
pub fn $set_field_name (&mut self) -> &mut Update {
	self.apply($pos, $field_name.set());
	self
}
					).as_slice());
				}

				let struct_update_impl = quote_tokens!(cx,
impl Update {
    fn apply(&mut self, pos:uint, diff:(uint, uint)) -> &mut Update {
        self.diff = ::foo::util::or_tuples(self.diff, ::foo::util::shift_tuple(pos, diff));
        self
    }

    $fields
}
				);

				let struct_impl = quote_tokens!(cx,
impl Reg {
    pub fn update(&'static mut self) -> Update {
        Update { origin: self, diff: (0, 0) }
    }

    pub fn modify(&mut self, diff: (uint, uint)) {
        let (c, s) = diff;
        if c != 0 {
            unsafe {
                let val = volatile_load(&self.field as *const u32);
                volatile_store(&mut self.field as *mut u32, val & !(c as u32) | (s as u32));
            }
        } else {
            unsafe {
                volatile_store(&mut self.field as *mut u32, s as u32);
            }
        }
    }
}
				);

				result.push(quote_item!(cx, 
#[allow(non_snake_case)]
pub mod $name {
	extern crate foo;

	use std::intrinsics::{volatile_load, volatile_store};

	$const_fields
	$struct_def
	$struct_impl
	$struct_update_def
	$struct_update_drop
	$struct_update_impl
}
).unwrap());
	    	},
	    }
	}

	if !iter.is_empty() {
		cx.span_err(sp, "unexpected content at end of macro");
	}

	// if false {

	return MacItems::new(SmallVector::many(result).into_iter());


	//     // result.push(quote_item!(cx, struct CustomStruct;).unwrap());
	// // }

	// return MacItems::new(SmallVector::many(result).into_iter());
    // MacExpr::new(quote_expr!(&mut *cx,
    // 	println!("fix me");
    // ))
}
