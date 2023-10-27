use std::{sync::{mpsc::Receiver, Mutex}, collections::HashMap, process::exit};
use lazy_static::lazy_static;

lazy_static! {
	static ref data: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Clone)]
pub struct Command<'a> {
	pub func: &'a fn(Vec<String>, String, Option<Receiver<i16>>) -> Result<(), String>,
	pub name: String,
	pub help: String,
}

pub fn create_commands() -> Vec<Command<'static>> {
	let mut cmds: Vec<Command<'static>> = Vec::new();
	cmds.push(Command {
		func: &(echo as fn(Vec<String>, String, Option<Receiver<i16>>) -> Result<(), String>),
		name: "echo".to_string(),
		help: "Echos args to stdout".to_string(),
	});
	cmds.push(Command {
		func: &(close as fn(Vec<String>, String, Option<Receiver<i16>>) -> Result<(), String>),
		name: "exit".to_string(),
		help: "Exits the terminal".to_string(),
	});
	return cmds;
}

fn echo(args: Vec<String>, args_string: String, _: Option<Receiver<i16>>) -> Result<(), String> {
	if args.len() == 1 {
		return Ok(());
	}
	println!("{}", args_string);
	return Ok(());
}

fn close(_: Vec<String>, _: String, _: Option<Receiver<i16>>) -> Result<(), String> {
	exit(0);
}
