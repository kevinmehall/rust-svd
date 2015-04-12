//! Peripheral struct generator from AST.

#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate syntax;
extern crate rustc;

use std::collections::VecMap;
use std::borrow::ToOwned;

use syntax::ast;
use syntax::ptr;
use syntax::parse::token::{str_to_ident};
use syntax::ext::base::ExtCtxt;

use ast::*;
use parser::*;

// Register or space filler.
enum StructMatch<'a> {
	Reg(&'a RegisterAst),
	Space(usize),
}

fn generate_regdefs<'a>(cx: &'a mut ExtCtxt, regmap:&VecMap<RegisterAst>)
	-> (Vec<ast::TokenTree>, Vec<ast::TokenTree>, Vec<ast::TokenTree>, Vec<ast::TokenTree>)
{
	let mut reg_mod:Vec<ast::TokenTree> = vec![];
	let mut reg_structfields:Vec<ast::TokenTree> = vec![];
	let mut reg_impl:Vec<ast::TokenTree> = vec![];
	let mut reg_defaults:Vec<ast::TokenTree> = vec![];

	// Create actual register mapping.
	let mut byte_idx:usize = 0;
	let mut regs:Vec<StructMatch> = vec![];
	for (pos, reg) in regmap.iter() {
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
	let mut reserved_idx:usize = 0;
	for item in regs.iter() {
		let mut reg;
		match item {
			&StructMatch::Space(len) => {
				let idxnum = reserved_idx.to_string();
				let reserved = str_to_ident(vec!["reserved_".to_owned(), idxnum].concat().as_slice());
				reserved_idx += 1;
				reg_structfields.push_all(quote_tokens!(cx, $reserved:[u8; $len],).as_slice());
				reg_defaults.push_all(quote_tokens!(cx, $reserved:[0; $len],).as_slice());
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
			const_fields.push_all(quote_tokens!(cx, const $n: ::svd::regs::RegField = ::svd::regs::RegField { width: $width };).as_slice());
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

			unsafe impl Sync for Reg {
			}

			pub const INIT:Reg = Reg { field: UnsafeCell { value: 0 } };
		);

		let mut enum_fields:Vec<ast::TokenTree> = vec![];
		let mut set_fields:Vec<ast::TokenTree> = vec![];
		let mut update_fields:Vec<ast::TokenTree> = vec![];
		let mut read_fields:Vec<ast::TokenTree> = vec![];

		// Dimension
		if let Some((dim, dim_incr)) = reg.dim {
			let namematch = str_to_ident(vec!["of_".to_owned(), reg.name.to_owned()].concat().as_slice());
			reg_impl.push_all(quote_tokens!(cx,
				#[inline(always)]
				pub fn $namematch (&self, i:u32) -> &'static $name::Reg {
	  				unsafe {
	    				&*((&self.$name as *const _).offset((($dim_incr/4) * i) as int))
	  				}
				}
			).as_slice());
		}

		for (pos, field) in reg.fields.iter() {
			let lowerenable = field.name.chars().map(|a| a.to_lowercase()).collect::<String>();
			let field_name = str_to_ident(field.name.as_slice());
			let set_field_name = str_to_ident(vec!["set_".to_owned(), lowerenable.clone()].concat().as_slice());
			let update_field_name = str_to_ident(vec!["update_".to_owned(), lowerenable.clone()].concat().as_slice());
			let clear_field_name = str_to_ident(vec!["clear_".to_owned(), lowerenable.clone()].concat().as_slice());
			match field.enumerate {
				None => {
					update_fields.push_all(quote_tokens!(cx, 
						#[inline(always)]
						pub fn $field_name (&mut self, value:usize) -> &mut Update {
							self.apply($pos, $field_name.update_value(value));
							self
						}
					).as_slice());
					set_fields.push_all(quote_tokens!(cx, 
						#[inline(always)]
						pub fn $field_name (&mut self, value:usize) -> &mut Set {
							self.apply($pos, $field_name.set_value(value));
							self
						}
					).as_slice());
					read_fields.push_all(quote_tokens!(cx,
						#[inline(always)]
						pub fn $field_name (&self) -> usize {
							$field_name.read(self.value as usize >> $pos)
						}
					).as_slice());
				},
				Some(ref choose) => {
					let field_enum = str_to_ident(vec![field.name.clone(), "Enum".to_owned()].concat().as_slice());
					let mut enum_opts:Vec<ast::TokenTree> = vec![];
					for &(ref name, val) in choose.iter() {
						let name_ident = str_to_ident(name.as_slice());
						let val = val as int;
						enum_opts.push_all(quote_tokens!(cx, $name_ident = $val,).as_slice());
					}
					enum_fields.push_all(quote_tokens!(cx, 
						#[derive(Copy, PartialEq, FromPrimitive)]
						#[allow(non_camel_case_types)] 
						pub enum $field_name {
							$enum_opts
						}
					).as_slice());
					update_fields.push_all(quote_tokens!(cx, 
						#[inline(always)]
						pub fn $field_name (&mut self, choice:$field_name) -> &mut Update {
							self.apply($pos, $field_name.update_value(choice as usize));
							self
						}
					).as_slice());
					set_fields.push_all(quote_tokens!(cx, 
						#[inline(always)]
						pub fn $field_name (&mut self, choice:$field_name) -> &mut Set {
							self.apply($pos, $field_name.set_value(choice as usize));
							self
						}
					).as_slice());
					read_fields.push_all(quote_tokens!(cx,
						#[inline(always)]
						pub fn $field_name (&self) -> Option<$field_name> {
							::std::num::FromPrimitive::from_usize($field_name.read(self.value as usize >> $pos))
						}
					).as_slice());
				}
			}
		}

		let def_update = quote_tokens!(cx, 
			pub struct Update {
				pub origin:&'static Reg,
				pub diff:(usize, usize),
			}

			impl Drop for Update {
				#[inline(always)]
				fn drop(&mut self) {
					self.origin.modify(self.diff);
				}
			}

			impl Update {
				#[inline(always)]
				fn apply(&mut self, pos:usize, diff:(usize, usize)) -> &mut Update {
					self.diff = ::svd::util::or_tuples(self.diff, ::svd::util::shift_tuple(pos, diff));
					self
				}

				$update_fields
			}
		);

		let def_set = quote_tokens!(cx, 
			pub struct Set {
				pub origin:&'static Reg,
				pub diff:(usize, usize),
			}

			impl Drop for Set {
				#[inline(always)]
				fn drop(&mut self) {
					self.origin.modify(self.diff);
				}
			}
			
			impl Set {
				#[inline(always)]
				fn apply(&mut self, pos:usize, diff:(usize, usize)) -> &mut Set {
					self.diff = ::svd::util::or_tuples(self.diff, ::svd::util::shift_tuple(pos, diff));
					self
				}

				$set_fields
			}
		);

		let def_read = quote_tokens!(cx, 
			#[derive(Copy)]
			pub struct Read {
				pub value:$r,
			}
			
			impl Read {
				$read_fields
			}
		);

		let struct_impl = quote_tokens!(cx,
			impl Reg {
				#[inline(always)]
				pub fn update(&'static self) -> Update {
					Update { origin: self, diff: (0, 0) }
				}

				#[inline(always)]
				pub fn write(&'static self) -> Set {
					Set { origin: self, diff: (0, 0) }
				}				

				#[inline(always)]
				pub fn modify(&self, diff: (usize, usize)) {
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

				#[inline(always)]
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
				extern crate svd;

				use std::prelude::*;
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

	(reg_mod, reg_structfields, reg_impl, reg_defaults)
}


fn generate_peripheral_cluster<'a>(cx: &'a mut ExtCtxt, peripheral:&PeripheralAst, clusters:&Vec<Cluster>) -> Vec<ast::TokenTree> {
	let mut cluster_structs = vec![];
	let mut clusters_tokens = vec![];

	for cluster in clusters.iter() {
		let (reg_mod, reg_structfields, reg_impl, reg_defaults) = generate_regdefs(cx, &cluster.registers);
		let cluster_name = str_to_ident(cluster.name.as_slice());

		cluster_structs.push_all(quote_tokens!(cx,
			#[allow(non_snake_case)]
			pub mod $cluster_name {
				use std::ptr::PtrExt;

				#[allow(non_snake_case)]
				#[repr(C)]
				pub struct Cluster {
					$reg_structfields
				}

				impl Cluster {
					$reg_impl
				}

				pub const INIT:Cluster = Cluster {
					$reg_defaults
				};

				$reg_mod
			}
		).as_slice());

		clusters_tokens.push_all(quote_tokens!(cx, 
			pub $cluster_name: &'static $cluster_name::Cluster,
		).as_slice());
	}

	// Generate peripheral module.
	let peripheral_name = str_to_ident(peripheral.name.as_slice());
	quote_tokens!(cx,
		#[allow(non_snake_case)]
		#[allow(unused_imports)]
		#[allow(dead_code)]
		pub mod $peripheral_name {
			use std::ptr::PtrExt;

			$cluster_structs

			#[allow(non_snake_case)]
			#[derive(Copy)]
			pub struct Clusters {
				$clusters_tokens
			}
		}
	)
}

fn generate_peripheral_registers<'a>(cx: &'a mut ExtCtxt, peripheral:&PeripheralAst, registers:&VecMap<RegisterAst>) -> Vec<ast::TokenTree> {
	let (reg_mod, reg_structfields, reg_impl, reg_defaults) = generate_regdefs(cx, registers);

	// Generate peripheral module.
	let peripheral_name = str_to_ident(peripheral.name.as_slice());
	quote_tokens!(cx,
		#[allow(non_snake_case)]
		#[allow(unused_imports)]
		#[allow(dead_code)]
		pub mod $peripheral_name {
			use std::ptr::PtrExt;

			#[allow(non_snake_case)]
			#[repr(C)]
			pub struct Peripheral {
				$reg_structfields
			}

			impl Peripheral {
				$reg_impl
			}

			pub const INIT:Peripheral = Peripheral {
				$reg_defaults
			};

			$reg_mod
		}
	)
}

pub fn generate_peripheral<'a>(cx: &'a mut ExtCtxt, peripheral:&PeripheralAst) -> Vec<ast::TokenTree> {
	match peripheral.regs {
		RegList::Registers(ref registers) => {
			generate_peripheral_registers(cx, peripheral, registers)
		},
		RegList::Cluster(ref clusters) => {
			generate_peripheral_cluster(cx, peripheral, clusters)
		}
	}
}

pub fn generate_system<'a>(cx: &'a mut ExtCtxt, name:String, peripherals:&Vec<PeripheralAst>)
  -> Box<Vec<ptr::P<ast::Item>>>
{
	let mut fields_extern = vec![];
	let mut fields_system = vec![];
	let mut fields_init = vec![];

	for p in peripherals.iter() {
		let pname_const = str_to_ident(vec![name.clone(), "_".to_owned(), p.name.clone()].concat().as_slice());
		let pname = str_to_ident(p.name.as_slice());

		match p.derives {
			Some(ref derives) => {
				let pname2_const = str_to_ident(vec![name.clone(), "_".to_owned(), derives.clone()].concat().as_slice());
				let pname2 = str_to_ident(derives.as_slice());

				match p.regs {
					RegList::Registers(ref regs) => {
						fields_extern.push(quote_tokens!(cx, static $pname_const: $pname2::Peripheral; ));
						fields_system.push(quote_tokens!(cx, pub $pname: &'static $pname2::Peripheral,));
						fields_init.push(quote_tokens!(cx, $pname: &$pname_const, ));
					},
					RegList::Cluster(ref clusters) => {
						let mut cluster_tokens = vec![];
						for c in clusters.iter() {
							let cname = str_to_ident(c.name.as_slice());
							let cname_const = str_to_ident(vec![name.clone(), p.name.clone(), c.name.clone()].connect("_").as_slice());

							fields_extern.push(quote_tokens!(cx, static $cname_const: $pname2::$cname::Cluster; ));
							cluster_tokens.push_all(quote_tokens!(cx,
								$cname: &$cname_const,
							).as_slice())
						}
						fields_system.push(quote_tokens!(cx, pub $pname: $pname2::Clusters,));
						fields_init.push(quote_tokens!(cx, $pname: $pname2::Clusters { $cluster_tokens }, ));
					}
				}
			},
			_ => {
				match p.regs {
					RegList::Registers(ref regs) => {
						fields_extern.push(quote_tokens!(cx, static $pname_const: $pname::Peripheral; ));
						fields_system.push(quote_tokens!(cx, pub $pname: &'static $pname::Peripheral,));
						fields_init.push(quote_tokens!(cx, $pname: &$pname_const, ));
					},
					RegList::Cluster(ref clusters) => {
						let mut cluster_tokens = vec![];
						for c in clusters.iter() {
							let cname = str_to_ident(c.name.as_slice());
							let cname_const = str_to_ident(vec![name.clone(), p.name.clone(), c.name.clone()].connect("_").as_slice());

							fields_extern.push(quote_tokens!(cx, static $cname_const: $pname::$cname::Cluster; ));
							cluster_tokens.push_all(quote_tokens!(cx,
								$cname: &$cname_const,
							).as_slice())
						}
						fields_system.push(quote_tokens!(cx, pub $pname: $pname::Clusters,));
						fields_init.push(quote_tokens!(cx, $pname: $pname::Clusters { $cluster_tokens }, ));
					}
				}
			}
		}
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
			#[derive(Copy)]
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
