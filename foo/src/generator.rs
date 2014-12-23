//! Peripheral struct generator from AST.

#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate syntax;
extern crate rustc;

use syntax::ast;
use syntax::ptr;
use syntax::parse::token::{str_to_ident};
use syntax::ext::base::ExtCtxt;

use ast::*;
use parser::*;

// Register or space filler.
enum StructMatch<'a> {
	Reg(&'a RegisterAst),
	Space(uint),
}

pub fn generate_peripheral<'a>(cx: &'a mut ExtCtxt, peripheral:&PeripheralAst) -> Vec<ast::TokenTree> {
	let mut reg_mod:Vec<ast::TokenTree> = vec![];
	let mut reg_structfields:Vec<ast::TokenTree> = vec![];
	let mut reg_defaults:Vec<ast::TokenTree> = vec![];

	// Create actual register mapping.
	let mut byte_idx:uint = 0;
	let mut regs:Vec<StructMatch> = vec![];
	for (pos, reg) in peripheral.regs.iter() {
		if pos > byte_idx {
			let mut len = pos - byte_idx;
			// space by 32 bytes so we can implement Show()
			// TODO no?
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

	// Generate registers.
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

		let mut const_fields:Vec<ast::TokenTree> = vec![];
		for (pos, field) in reg.fields.iter() {
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
			#[repr(C)]
			pub struct Reg {
				pub field:UnsafeCell<$r>,
			}

			pub const INIT:Reg = Reg { field: UnsafeCell { value: 0 } };
		);

		let mut enum_fields:Vec<ast::TokenTree> = vec![];
		let mut set_fields:Vec<ast::TokenTree> = vec![];
		let mut update_fields:Vec<ast::TokenTree> = vec![];
		let mut read_fields:Vec<ast::TokenTree> = vec![];

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
					read_fields.push_all(quote_tokens!(cx,
						pub fn $field_name (&self) -> uint {
							$field_name.read(self.value as uint >> $pos)
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
						#[deriving(Copy, Show, PartialEq, FromPrimitive)]
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
					read_fields.push_all(quote_tokens!(cx,
						pub fn $field_name (&self) -> Option<$field_name> {
							FromPrimitive::from_uint($field_name.read(self.value as uint >> $pos))
						}
					).as_slice());
				}
			}
		}

		let def_update = quote_tokens!(cx, 
			pub struct Update {
				pub origin:&'static Reg,
				pub diff:(uint, uint),
			}

			impl Drop for Update {
				fn drop(&mut self) {
					self.origin.modify(self.diff);
				}
			}

			impl Update {
				fn apply(&mut self, pos:uint, diff:(uint, uint)) -> &mut Update {
					self.diff = ::bar::util::or_tuples(self.diff, ::bar::util::shift_tuple(pos, diff));
					self
				}

				$update_fields
			}
		);

		let def_set = quote_tokens!(cx, 
			pub struct Set {
				pub origin:&'static Reg,
				pub diff:(uint, uint),
			}

			impl Drop for Set {
				fn drop(&mut self) {
					self.origin.modify(self.diff);
				}
			}
			
			impl Set {
				fn apply(&mut self, pos:uint, diff:(uint, uint)) -> &mut Set {
					self.diff = ::bar::util::or_tuples(self.diff, ::bar::util::shift_tuple(pos, diff));
					self
				}

				$set_fields
			}
		);

		let def_read = quote_tokens!(cx, 
			#[deriving(Copy)]
			pub struct Read {
				pub value:$r,
			}
			
			impl Read {
				$read_fields
			}
		);

		let struct_impl = quote_tokens!(cx,
			impl Reg {
				pub fn update(&'static self) -> Update {
					Update { origin: self, diff: (0, 0) }
				}

				pub fn write(&'static self) -> Set {
					Set { origin: self, diff: (0, 0) }
				}				

				pub fn modify(&self, diff: (uint, uint)) {
					let (c, s) = diff;
					if c != 0 {
						unsafe {
						    let val = volatile_load(self.field.get() as *const $r);
						    volatile_store(self.field.get() as *mut $r, val & !(c as $r) | (s as $r));
						}
					} else {
						unsafe {
						    volatile_store(self.field.get() as *mut $r, s as $r);
						}
					}
				}

				pub fn read(&self) -> Read {
					unsafe {
					    let val = volatile_load(self.field.get() as *const $r);
						Read { value: val }
					}
				}
			}
		);

		reg_mod.push_all(quote_tokens!(cx, 
			#[allow(non_snake_case)]
			pub mod $name {
				extern crate bar;

				use std::intrinsics::{volatile_load, volatile_store};
				use std::cell::UnsafeCell;

				$const_fields
				$enum_fields
				$struct_def
				$struct_impl
				$def_update
				$def_set
				$def_read
			}
		).as_slice());
	}

	// Generate peripheral module.
	let peripheral_name = str_to_ident(peripheral.name.as_slice());
	return quote_tokens!(cx,
		#[allow(non_snake_case)]
		pub mod $peripheral_name {
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
}

pub fn generate_system<'a>(cx: &'a mut ExtCtxt, name:String, peripherals:&Vec<PeripheralAst>)
  -> Box<Vec<ptr::P<ast::Item>>>
{
	let mut fields_extern = vec![];
	let mut fields_system = vec![];
	let mut fields_init = vec![];

	for p in peripherals.iter() {
		let pname = str_to_ident(vec![name.as_slice(), "_", p.name.as_slice()].concat().as_slice());
		let name = str_to_ident(p.name.as_slice());
		fields_extern.push(quote_tokens!(cx, static $pname: $name::Peripheral; ));
		fields_system.push(quote_tokens!(cx, pub $name: &'static $name::Peripheral,));
		fields_init.push(quote_tokens!(cx, $name: &$pname, ));
	}

	box vec![
		quote_item!(cx,
			#[allow(improper_ctypes)]
			extern {
				$fields_extern
			}
		).unwrap(),

		quote_item!(cx,
			#[allow(non_snake_case)]
			#[deriving(Copy)]
			pub struct System {
				$fields_system
			}
		).unwrap(),

		quote_item!(cx,
			pub static SYS:System = System {
				$fields_init
			};
		).unwrap(),
	]
}
