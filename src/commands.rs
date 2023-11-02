#![allow(non_upper_case_globals)]

use std::{path::Path, sync::{mpsc::Receiver, Mutex}, collections::HashMap, process::exit};
use lazy_static::lazy_static;

lazy_static! {
	pub static ref data: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
	pub static ref path: Mutex<Vec<String>> = Mutex::new(Vec::new());
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
		func: &(help_command as fn(Vec<String>, String, Option<Receiver<i16>>) -> Result<(), String>),
		name: "help".to_string(),
		help: "Shows this help menu".to_string(),
	});
	crate::debug(format!("init {}", cmds.last().unwrap().name));
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
	cmds.push(Command {
		func: &(list_directory as fn(Vec<String>, String, Option<Receiver<i16>>) -> Result<(), String>),
		name: "ls".to_string(),
		help: "Lists the current directory".to_string(),
	});
	crate::debug(format!("init {}", cmds.last().unwrap().name));
	return cmds;
}

fn help_command(_: Vec<String>, _: String, rv: Option<Receiver<i16>>) -> Result<(), String> {
	let mut commands = create_commands();
	commands.sort_by_key(|x| x.name.clone());

	let mut longest_command: usize = 0;
	for cmd in commands.iter() {
		if cmd.name.len() > longest_command {
			longest_command = cmd.name.len();
		}
	}

	for cmd in commands {
		println!("{}{}{}", cmd.name, " ".repeat(longest_command-cmd.name.len()+2), cmd.help);
	}

	return Ok(());
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

fn set_variable(args: Vec<String>, _: String, rv: Option<Receiver<i16>>) -> Result<(), String> {

	if args.len() != 3 {
		println!("Syntax: set {{var_name}} {{var_data}}");
		return Ok(());
	}
	if args[1].chars().any(|x| x.to_string() == " ") || args[1] == "" {
		return Err("Name cannot have whitespace".to_string());
	}
	let mut d;
	if rv.is_some() {
		let channel = rv.unwrap();
		loop {
			if let Ok(o) = channel.try_recv() {
				if o == 1 {
					return Err("Could not aquire variable mutex".to_string());
				}
			}
			if let Ok(o) = data.try_lock() {
				d = o;
				break;
			}
		}
	} else {
		d = data.lock().unwrap();
	}

	if d.contains_key(&args[1].clone().trim().to_string()) {
		d.remove(&args[1].clone().trim().to_string());
	}
	d.insert(args[1].clone().trim().to_string(), args[2].clone());

	return Ok(());
}

fn list_variables(_: Vec<String>, _: String, rv: Option<Receiver<i16>>) -> Result<(), String> {
	
	if rv.is_none() {
		let d = data.lock().unwrap();
		for (key, val) in d.iter() {
			println!("{} = \"{}\"", key, val);
		}
	} else {
		let d;
		let channel = rv.unwrap();

		loop {
			if let Ok(o) = channel.try_recv() {
				if o == 1 {
					return Err("Could not aquire variable mutex".to_string());
				}
			}
			if let Ok(o) = data.try_lock() {
				d = o;
				break;
			}
		}

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

fn list_directory(args: Vec<String>, _: String, rv: Option<Receiver<i16>>) -> Result<(), String> {

	if args.len() > 2 {
		println!("Syntax: ls {{directory}}");
		return Ok(());
	}
	let files = if args.len() == 2 {
		if !(Path::new(&args[1]).is_dir()) {
			println!("Directory does not exist");
			return Ok(());
		}
		match std::fs::read_dir(&args[1]) {
			Ok(o) => {
				o
			},
			Err(e) => {
				return Err(e.to_string());
			}
		}
	} else {
		match std::fs::read_dir(".") {
			Ok(o) => {
				o
			},
			Err(e) => {
				return Err(e.to_string());
			}
		}
	};
	// let files = files.map(|x| x.unwrap().file_name().to_str().unwrap().to_string()).collect::<Vec<String>>();
	let files = files.map(|x| x.unwrap()).collect::<Vec<std::fs::DirEntry>>();
	let mut longest_file_size = 5;

	files.iter().for_each(|x| if x.metadata().unwrap().len().to_string().len() > longest_file_size {
		longest_file_size = x.metadata().unwrap().len().to_string().len();
	});

	if rv.is_some() {
		let channel = rv.unwrap();
		for f in files {
			if let Ok(o) = channel.try_recv() {
				if o == 1 {
					return Ok(());
				}
			}
			let t: chrono::DateTime<chrono::Local> = f.metadata().unwrap().modified().unwrap().into();
			if f.metadata().unwrap().is_file() {
				let size = f.metadata().unwrap().len().to_string();
				if crate::is_executable(f.path().canonicalize().unwrap().to_str().unwrap()) {
					print!("{} {}{} {}\n", t.format("%b %d %H:%M"), " ".repeat(longest_file_size-size.len()), size, console::style(f.file_name().to_str().unwrap()).green().bright());
				} else {
					print!("{} {}{} {}\n", t.format("%b %d %H:%M"), " ".repeat(longest_file_size-size.len()), size, f.file_name().to_str().unwrap());
				}
				
				
			} else {
				print!("{} {}<DIR> {}\n", t.format("%b %d %H:%M"), " ".repeat(longest_file_size-5), console::style(f.file_name().to_str().unwrap()).blue().bright());
			}
		}
	} else {
		for f in files {
			let t: chrono::DateTime<chrono::Local> = f.metadata().unwrap().modified().unwrap().into();
			if f.metadata().unwrap().is_file() {
				let size = f.metadata().unwrap().len().to_string();
				if crate::is_executable(f.path().canonicalize().unwrap().to_str().unwrap()) {
					print!("{} {}{} {}\n", t.format("%b %d %H:%M"), " ".repeat(longest_file_size-size.len()), size, console::style(f.file_name().to_str().unwrap()).green().bright());
				} else {
					print!("{} {}{} {}\n", t.format("%b %d %H:%M"), " ".repeat(longest_file_size-size.len()), size, f.file_name().to_str().unwrap());
				}
				
				
			} else {
				print!("{} {}<DIR> {}\n", t.format("%b %d %H:%M"), " ".repeat(longest_file_size-5), console::style(f.file_name().to_str().unwrap()).blue().bright());
			}
		}
	}


	return Ok(());
}