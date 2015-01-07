//! Peripheral macro parser into AST.

#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate syntax;
extern crate rustc;

use std::collections::VecMap;

use syntax::ast;
use syntax::codemap;
use syntax::parse::token::Token;
use syntax::parse::token::{str_to_ident};
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr, MacItems};
use syntax::parse::token;
use syntax::util::small_vector::SmallVector;

use reader::{Reader, TreeUnpack};
use ast::*;

// Temporary placement for comment parsing.
pub fn ignore_comments<'a>(reader:&mut Reader<'a>) {
	while let Ok(..) = reader.read_comment() {
		continue;
	}
}

pub fn parse_field<'a>(reader:&mut Reader<'a>)
  -> Result<FieldAst, String>
{
	let access = try!(reader.read_ident_match(&["rw", "r", "w"]));
	let delimiter = try!(reader.read_brackets());
	let sub = &mut delimiter.unpack_tree();
	let subscript = try!(sub.read_int());
	let name = try!(reader.read_ident());

	let mut enumerate = None;
	if reader.has_brackets() {
		let braces = try!(reader.read_brackets());
		let enumset = &mut braces.unpack_tree();
		let mut enumvec:Vec<(String, uint)> = vec![];
		ignore_comments(enumset);
		while enumset.has_ident() {
			let name = try!(enumset.read_ident());
			try!(enumset.read_token(Token::Eq));
			let value = try!(enumset.read_int()) as uint;
			enumvec.push((name, value));
			match enumset.read_token(Token::Comma) {
				Ok(..) => (),
				Err(..) => { break }
			}
			ignore_comments(enumset);
		}
		if !enumset.is_done() {
			return Err("Unexpected content in enum definition".to_string());
		}
		enumerate = Some(enumvec);
	}

	Ok(FieldAst {
		name: name,
		width: subscript as uint,
		enumerate: enumerate,
	})
}

pub fn parse_fields<'a>(reader:&mut Reader<'a>)
	-> Result<VecMap<FieldAst>, String>
{
	let mut out = VecMap::new();
	ignore_comments(reader);
	while reader.has_int() {
		let pos = try!(reader.read_int()) as uint;
		try!(reader.read_token(Token::FatArrow));
		out.insert(pos, try!(parse_field(reader)));
		match reader.read_token(Token::Comma) {
			Ok(..) => (),
			Err(..) => { break }
		}
		ignore_comments(reader);
	}

	Ok(out)
}

pub fn parse_reg<'a>(reader:&mut Reader<'a>)
	-> Result<RegisterAst, String>
{
	let regmap = try!(reader.read_ident_match(&["reg8", "reg16", "reg32"]));
	let name = try!(reader.read_ident());
	let _sub = try!(reader.read_brackets());
	let sub = &mut _sub.unpack_tree();

	let fields = try!(parse_fields(sub));
	if !sub.is_done() {
		return Err("Unexpected content in register definition".to_string());
	}
	
	Ok(RegisterAst {
		name: name,
		fields: fields,
		width: match regmap.as_slice() {
			"reg16" => 2,
			"reg32" => 4,
			"reg8" | _ => 1,
		},
		dim: None,
	})
}

pub fn parse_peripheral<'a>(reader:&mut Reader<'a>)
	-> Result<PeripheralAst, String>
{
	ignore_comments(reader);

	let regmap = try!(reader.read_ident_match(&["peripheral"]));
	let name = try!(reader.read_ident());
	let delimiter = try!(reader.read_brackets());
	let sub = &mut delimiter.unpack_tree();

	let mut regs = VecMap::new();
	ignore_comments(sub);
	while sub.has_int() {
		let pos = try!(sub.read_int()) as uint;
		try!(sub.read_token(Token::FatArrow));
		regs.insert(pos, try!(parse_reg(sub)));
		let _ = sub.read_token(token::Comma); // optional
		ignore_comments(sub);
	}
	if !sub.is_done() {
		return Err(format!("Unexpected content at end of peripheral definition"));
	}

	Ok(PeripheralAst {
		name: name,
		address: 0,
		regs: RegList::Registers(regs),
		derives: None,
	})
}
