use std::{path::Path, sync::{mpsc::Receiver, Mutex}, collections::HashMap, process::exit};
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
	crate::debug(format!("init {}", cmds.last().unwrap().name));
	cmds.push(Command {
		func: &(close as fn(Vec<String>, String, Option<Receiver<i16>>) -> Result<(), String>),
		name: "exit".to_string(),
		help: "Exits the terminal".to_string(),
	});
	crate::debug(format!("init {}", cmds.last().unwrap().name));
	cmds.push(Command {
		func: &(change_directory as fn(Vec<String>, String, Option<Receiver<i16>>) -> Result<(), String>),
		name: "cd".to_string(),
		help: "Change the current directory".to_string(),
	});
	crate::debug(format!("init {}", cmds.last().unwrap().name));
	cmds.push(Command {
		func: &(set_variable as fn(Vec<String>, String, Option<Receiver<i16>>) -> Result<(), String>),
		name: "set".to_string(),
		help: "Set a variable to a value".to_string(),
	});
	crate::debug(format!("init {}", cmds.last().unwrap().name));
	cmds.push(Command {
		func: &(list_variables as fn(Vec<String>, String, Option<Receiver<i16>>) -> Result<(), String>),
		name: "list".to_string(),
		help: "Lists the currently defined variables".to_string(),
	});
	crate::debug(format!("init {}", cmds.last().unwrap().name));
	return cmds;
}

fn echo(args: Vec<String>, args_string: String, _: Option<Receiver<i16>>) -> Result<(), String> {
	if args.len() == 1 {
		println!("Syntax: echo {{input}}");
		return Ok(());
	}
	println!("{}", args_string);
	return Ok(());
}

fn close(_: Vec<String>, _: String, _: Option<Receiver<i16>>) -> Result<(), String> {
	exit(0);
}

fn change_directory(args: Vec<String>, _: String, _: Option<Receiver<i16>>) -> Result<(), String> {

	if args.len() == 1 {
		println!("Syntax: cd {{directory}}");
		return Ok(());
	}
	let p = Path::new(&args[1]);
	if p.exists() && p.is_dir() {
		if let Err(e) = std::env::set_current_dir(p) {
			return Err(e.to_string());
		}
	} else {
		return Err("Directory does not exist".to_string());
	}

	return Ok(());
}

fn set_variable(args: Vec<String>, _: String, _: Option<Receiver<i16>>) -> Result<(), String> {

	if args.len() != 3 {
		println!("Syntax: set {{var_name}} {{var_data}}");
		return Ok(());
	}
	if args[1].chars().any(|x| x.to_string() == " ") || args[1] == "" {
		return Err("Name cannot have whitespace".to_string());
	}
	let mut d = data.lock().unwrap();
	if d.contains_key(&args[1].clone().trim().to_string()) {
		d.remove(&args[1].clone().trim().to_string());
	}
	d.insert(args[1].clone().trim().to_string(), args[2].clone());

	return Ok(());
}

fn list_variables(_: Vec<String>, _: String, rv: Option<Receiver<i16>>) -> Result<(), String> {

	let d = data.lock().unwrap();
	
	if rv.is_none() {
		for (key, val) in d.iter() {
			println!("{} = \"{}\"", key, val);
		}
	} else {
		let channel = rv.unwrap();
		for (key, val) in d.iter() {
			if let Ok(o) = channel.try_recv() {
				if o == 1 {
					break;
				}
			}
			println!("{} = \"{}\"", key, val);
		}
	}

	return Ok(());
}
