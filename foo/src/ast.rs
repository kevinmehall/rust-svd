//! AST structure

use std::collections::VecMap;

#[deriving(Show, Clone)]
pub struct PeripheralAst {
	pub name:String,
	pub regs:VecMap<RegisterAst>,
}

#[deriving(Show, Clone)]
pub struct RegisterAst {
	pub name:String,
	pub fields:VecMap<FieldAst>,
	pub width:uint,
}

#[deriving(Show, Clone)]
pub struct FieldAst {
	pub name:String,
	pub width:uint,
	pub enumerate:Option<Vec<(String, uint)>>
}
