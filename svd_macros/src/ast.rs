//! AST structure

use std::collections::VecMap;

#[derive(Show, Clone)]
pub enum RegList {
	Registers(VecMap<RegisterAst>),
	Cluster(Vec<Cluster>),
}

#[derive(Show, Clone)]
pub struct PeripheralAst {
	pub name:String,
	pub address:u32,
	pub regs:RegList,
	pub derives:Option<String>,
}

#[derive(Show, Clone)]
pub struct Cluster {
	pub name:String,
	pub registers:VecMap<RegisterAst>,
}

#[derive(Show, Clone)]
pub struct RegisterAst {
	pub name:String,
	pub fields:VecMap<FieldAst>,
	pub width:uint,
	pub dim:Option<(u32, u32)>,
}

#[derive(Show, Clone)]
pub struct FieldAst {
	pub name:String,
	pub width:uint,
	pub enumerate:Option<Vec<(String, uint)>>
}
