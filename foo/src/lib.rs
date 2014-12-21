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

fn is_brackets(t:Option<&&ast::TokenTree>) -> bool {
	match t {
		Some(&&TtDelimited(..)) => true,
		_ => false,
	}
}

fn read_comment<'a>(iter:&mut AstIterator<'a>) -> Result<String, String> {
	match iter.peek() {
		Some(&&TtToken(_, Token::DocComment(value))) => {
			iter.next();
			Ok(value.to_string())
		},
		_ => {
			Err(format!("expecting comment"))
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
		(&Token::Eq, &Token::Eq) |
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
	Err(format!("expecting simple token {}, got {}", t, iter.peek()))
}

fn read_ident_match<'a>(iter:&mut AstIterator<'a>, select:&[&str]) -> Result<String, String> {
	let ident = try!(read_ident(iter));
	match select.iter().position(|p| &ident.as_slice() == p) {
		None => Err(format!("unexpected identifier {}", ident)),
		_ => Ok(ident),
	}
}


#[deriving(Show, Clone)]
struct Field {
	name:String,
	width:uint,
	enumerate:Option<Vec<(String, uint)>>
}

#[deriving(Show, Clone)]
struct RegisterAst {
	name:String,
	fields:VecMap<Field>,
	width:uint,
}

fn parse_field<'a>(iter:&mut AstIterator<'a>) -> Result<Field, String> {
	let access = try!(read_ident_match(iter, &["rw", "r", "w"]));
	let delimiter = try!(read_brackets(iter));
	let sub = &mut read_tree(&*delimiter);
	let subscript = try!(read_int(sub));
	let name = try!(read_ident(iter));

	// println!("{} {} {}", access, subscript, name);

	let mut enumerate = None;
	if is_brackets(iter.peek()) {
		let braces = try!(read_brackets(iter));
		let enumset = &mut read_tree(&*braces);
		let mut enumvec:Vec<(String, uint)> = vec![];
		ignore_comments(enumset);
		while is_ident(enumset.peek()) {
			let name = try!(read_ident(enumset));
			try!(read_token(enumset, Token::Eq));
			let value = try!(read_int(enumset)) as uint;
			enumvec.push((name, value));
			match read_token(enumset, Token::Comma) {
				Ok(..) => (),
				Err(..) => { break }
			}
			ignore_comments(enumset);
		}
		if !enumset.is_empty() {
			return Err("Unexpected content in enum definition".to_string());
		}
		enumerate = Some(enumvec);
	}

	let a = Field {
		name: name,
		width: subscript as uint,
		enumerate: enumerate,
	};

	return Ok(a);
}

fn parse_fields<'a>(iter:&mut AstIterator<'a>)
	-> Result<VecMap<Field>, String>
{
	let mut out = VecMap::new();
	ignore_comments(iter);
	while is_int(iter.peek()) {
		let pos = try!(read_int(iter)) as uint;
		try!(read_token(iter, Token::FatArrow));
		out.insert(pos, try!(parse_field(iter)));
		match read_token(iter, Token::Comma) {
			Ok(..) => (),
			Err(..) => { break }
		}
		ignore_comments(iter);
	}
	Ok(out)
}

fn parse_reg<'a>(iter:&mut AstIterator<'a>)
	-> Result<RegisterAst, String>
{
	let regmap = try!(read_ident_match(iter, &["reg8", "reg16", "reg32"]));
	let name = try!(read_ident(iter));
	let delimiter = try!(read_brackets(iter));
	let sub = &mut read_tree(&*delimiter);

	let fields = try!(parse_fields(sub));
	if !sub.is_empty() {
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
	})
}

struct PeripheralAst {
	name:String,
	regs:VecMap<RegisterAst>,
}

fn ignore_comments<'a>(iter:&mut AstIterator<'a>) {
	while let Ok(..) = read_comment(iter) {
		continue;
	}
}

fn parse_peripheral<'a>(iter:&mut AstIterator<'a>)
	-> Result<PeripheralAst, String>
{
	ignore_comments(iter);

	let regmap = try!(read_ident_match(iter, &["peripheral"]));
	let name = try!(read_ident(iter));
	let delimiter = try!(read_brackets(iter));
	let sub = &mut read_tree(&*delimiter);

	let mut regs = VecMap::new();
	ignore_comments(sub);
	while is_int(sub.peek()) {
		let pos = try!(read_int(sub)) as uint;
		try!(read_token(sub, Token::FatArrow));
		regs.insert(pos, try!(parse_reg(sub)));
		let _ = read_token(sub, token::Comma); // optional
		ignore_comments(sub);
	}
	if !sub.is_empty() {
		return Err(format!("Unexpected content at end of peripheral definition"));
	}
	Ok(PeripheralAst {
		name: name,
		regs: regs
	})
}

enum StructMatch<'a> {
	Reg(&'a RegisterAst),
	Space(uint),
}

fn expand<'a>(cx: &'a mut ExtCtxt, sp: codemap::Span, tokens: &[ast::TokenTree]) -> Box<MacResult + 'a> {
	let mut result = vec![];

	// cx.span_err(sp, "dummy is only permissible on functions");

	let mut iter = tokens.iter().peekable();

    // for i in tokens.iter() {
    // 	println!("{}", i);
    // }

    while !iter.is_empty() {
	    match parse_peripheral(&mut iter) {
	    	Err(err) => {
	    		cx.span_err(sp, err.as_slice());
	    		break;
	    	},
	    	Ok(peripheral) => {
	    		let mut reg_mod:Vec<ast::TokenTree> = vec![];
	    		let mut reg_structfields:Vec<ast::TokenTree> = vec![];
	    		let mut reg_defaults:Vec<ast::TokenTree> = vec![];

	    		// regs.values().map(|a| (*a).clone()).collect()

	    		// Create actual register mapping.
	    		let mut byte_idx:uint = 0;
	    		let mut regs:Vec<StructMatch> = vec![];
	    		for (pos, reg) in peripheral.regs.iter() {
	    			if pos > byte_idx {
	    				let mut len = pos - byte_idx;
	    				// space by 32 bytes so we can implement Show()
	    				while len > 0 {
	    					let sub_len = if len > 32 { 32 } else { len };
		    				regs.push(StructMatch::Space(sub_len));
		    				byte_idx += sub_len;
		    				len = len - sub_len;
		    			}
	    			}
	    			regs.push(StructMatch::Reg(reg));
	    			byte_idx += reg.width;
	    		}
	    		// regpairs.sort_by(|a, b| a.addressOffset.as_ref().unwrap().cmp(b.addressOffset.as_ref().unwrap()));

	    		let mut reserved_idx:uint = 0;
	    		for item in regs.iter() {
	    			let mut reg;
	    			match item {
	    				&StructMatch::Space(len) => {
	    					let idxnum = reserved_idx.to_string();
	    					let reserved = str_to_ident(vec!["reserved_", idxnum.as_slice()].concat().as_slice());
	    					reserved_idx += 1;
	    					reg_structfields.push_all(quote_tokens!(cx, $reserved:[u8, ..$len],).as_slice());
	    					reg_defaults.push_all(quote_tokens!(cx, $reserved:[0, ..$len],).as_slice());
	    					continue;
	    				}
	    				&StructMatch::Reg(item) => {
	    					reg = item
	    				}
	    			}

		    		// println!("wow {}", reg);
		    		let mut const_fields:Vec<ast::TokenTree> = vec![];
		    		for (pos, field) in reg.fields.iter() {
						// println!("{}", field);
						let n = str_to_ident(field.name.as_slice());
						let width = field.width;
						const_fields.push_all(quote_tokens!(cx, const $n: ::bar::regs::RegField = ::bar::regs::RegField { width: $width };).as_slice());
					}

					let name = str_to_ident(reg.name.as_slice());
					let name_update = str_to_ident((reg.name.clone() + "Update").as_slice());

					reg_structfields.push_all(quote_tokens!(cx, pub $name:$name::Reg,).as_slice());
					reg_defaults.push_all(quote_tokens!(cx, $name: $name ::INIT,).as_slice());

					let r = str_to_ident(match reg.width {
						4 => "u32",
						2 => "u16",
						1 | _ => "u8"
					});

					let struct_def = quote_tokens!(cx,
#[deriving(Show, Copy)]
#[repr(C)]
pub struct Reg {
    pub field:$r,
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

					let struct_set_def = quote_tokens!(cx, 
#[deriving(Show)]
pub struct Set {
    pub origin:&'static mut Reg,
    pub diff:(uint, uint),
}
					);

					let struct_set_drop = quote_tokens!(cx,
impl Drop for Set {
    fn drop(&mut self) {
        self.origin.modify(self.diff);
    }
}
					);

					let mut enum_fields:Vec<ast::TokenTree> = vec![];

					let mut set_fields:Vec<ast::TokenTree> = vec![];
					let mut update_fields:Vec<ast::TokenTree> = vec![];
					for (pos, field) in reg.fields.iter() {
						let lowerenable = field.name.chars().map(|a| a.to_lowercase()).collect::<String>();
						let field_name = str_to_ident(field.name.as_slice());
						let set_field_name = str_to_ident(vec!["set_", lowerenable.as_slice()].concat().as_slice());
						let update_field_name = str_to_ident(vec!["update_", lowerenable.as_slice()].concat().as_slice());
						let clear_field_name = str_to_ident(vec!["clear_", lowerenable.as_slice()].concat().as_slice());
						match field.enumerate {
							None => {
								update_fields.push_all(quote_tokens!(cx, 
pub fn $field_name (&mut self, value:uint) -> &mut Update {
	self.apply($pos, $field_name.update_value(value));
	self
}
								).as_slice());
								set_fields.push_all(quote_tokens!(cx, 
pub fn $field_name (&mut self) -> &mut Set {
	self.apply($pos, $field_name.set());
	self
}
								).as_slice());
							},
							Some(ref choose) => {
								let field_enum = str_to_ident(vec![field.name.as_slice(), "Enum"].concat().as_slice());
								let mut enum_opts:Vec<ast::TokenTree> = vec![];
								for &(ref name, val) in choose.iter() {
									let name_ident = str_to_ident(name.as_slice());
									let val = val as int;
									enum_opts.push_all(quote_tokens!(cx, $name_ident = $val,).as_slice());
								}
								enum_fields.push_all(quote_tokens!(cx, 
#[deriving(Copy)]
#[allow(non_camel_case_types)] 
pub enum $field_name {
	$enum_opts
}
								).as_slice());
								update_fields.push_all(quote_tokens!(cx, 
pub fn $field_name (&mut self, choice:$field_name) -> &mut Update {
	self.apply($pos, $field_name.update_value(choice as uint));
	self
}
								).as_slice());
								set_fields.push_all(quote_tokens!(cx, 
pub fn $field_name (&mut self, choice:$field_name) -> &mut Set {
	self.apply($pos, $field_name.set_value(choice as uint));
	self
}
								).as_slice());
							}
						}
					}

					let struct_update_impl = quote_tokens!(cx,
impl Update {
    fn apply(&mut self, pos:uint, diff:(uint, uint)) -> &mut Update {
        self.diff = ::bar::util::or_tuples(self.diff, ::bar::util::shift_tuple(pos, diff));
        self
    }

    $update_fields
}
					);
					let struct_set_impl = quote_tokens!(cx,
impl Set {
    fn apply(&mut self, pos:uint, diff:(uint, uint)) -> &mut Set {
        self.diff = ::bar::util::or_tuples(self.diff, ::bar::util::shift_tuple(pos, diff));
        self
    }

    $set_fields
}
					);

					let struct_impl = quote_tokens!(cx,
impl Reg {
    pub fn update(&'static mut self) -> Update {
        Update { origin: self, diff: (0, 0) }
    }

    pub fn write(&'static mut self) -> Set {
        Set { origin: self, diff: (0, 0) }
    }

    pub fn modify(&mut self, diff: (uint, uint)) {
        let (c, s) = diff;
        if c != 0 {
            unsafe {
                let val = volatile_load(&self.field as *const $r);
                volatile_store(&mut self.field as *mut $r, val & !(c as $r) | (s as $r));
            }
        } else {
            unsafe {
                volatile_store(&mut self.field as *mut $r, s as $r);
            }
        }
    }
}
					);

					reg_mod.push_all(quote_tokens!(cx, 
#[allow(non_snake_case)]
pub mod $name {
	extern crate bar;

	use std::intrinsics::{volatile_load, volatile_store};

	$const_fields
	$enum_fields
	$struct_def
	$struct_impl
	$struct_update_def
	$struct_update_drop
	$struct_update_impl
	$struct_set_def
	$struct_set_drop
	$struct_set_impl
}
					).as_slice());
	    		}

				let peripheral_name = str_to_ident(peripheral.name.as_slice());
				let peripheral_struct = quote_tokens!(cx,
#[allow(non_snake_case)]
pub mod $peripheral_name {
	#[deriving(Copy, Show)]
	#[allow(non_snake_case)]
	#[repr(C)]
	pub struct Peripheral {
		$reg_structfields
	}

	pub const INIT:Peripheral = Peripheral {
		$reg_defaults
	};
	$reg_mod
}
				);
				result.push(quote_item!(cx, 
$peripheral_struct
				).unwrap());
	    	}
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
