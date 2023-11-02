use console::Term;
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};

mod commands;

fn print_error<S: std::fmt::Display>(line_num: u32, e: S) {
	eprintln!("Error (line: {}): {}", line_num, e);
}

fn print_syntax_error<S: std::fmt::Display>(e: S) {
	eprintln!("{}{}", console::style("Syntax Error: ").red().bright(), console::style(e).red().bright());
}

fn read_command(term: &Term, history: &Vec<String>) -> String {
	let mut inp: String = "".to_string();
	let mut cursor_pos: i32 = 0;
	let mut history_position: Option<i32> = None;
	loop {
		match term.read_key() {
			Ok(o) => {
				if let console::Key::Char(c) = o {
					if c.to_string() != "\t" {
						inp.insert(cursor_pos as usize, c);
						term.hide_cursor().unwrap();
						if let Err(e) = term.write_str(&inp[cursor_pos as usize..]) {
							print_error(line!(), e);
						}
						term.move_cursor_left(inp.len() - cursor_pos as usize).unwrap();
						term.show_cursor().unwrap();
						term.move_cursor_right(1).unwrap();
						cursor_pos += 1;
					}
					// if let Err(e) = term.write_str(c.to_string().as_str()) {
					// 	print_error(line!(), e);
					// }

				} else if o == console::Key::Backspace && cursor_pos >= 1 {
					term.hide_cursor().unwrap();
					term.move_cursor_left(1).unwrap();
					term.write_str(&inp[cursor_pos as usize..]).unwrap();
					term.write_str(" ").unwrap();
					term.move_cursor_left(inp.len()-cursor_pos as usize+1).unwrap();
					term.show_cursor().unwrap();
					inp.remove(cursor_pos as usize-1);
					cursor_pos -= 1;
				} else if o == console::Key::Enter {
					print!("\n");
					return inp;
				} else if o == console::Key::ArrowLeft {
					if cursor_pos >= 1 {
						term.move_cursor_left(1).unwrap();
						cursor_pos -= 1;
					}
				} else if o == console::Key::ArrowRight {
					if (cursor_pos as usize) < inp.len() {
						term.move_cursor_right(1).unwrap();
						cursor_pos += 1;
					}
				} else if o == console::Key::Del && (cursor_pos as usize) < inp.len() {
					term.hide_cursor().unwrap();
					inp.remove(cursor_pos as usize);
					term.write_str(&inp[cursor_pos as usize..]).unwrap();
					term.write_str(" ").unwrap();
					term.move_cursor_left(inp.len()-cursor_pos as usize+1).unwrap();
					term.show_cursor().unwrap();
				} else if o == console::Key::ArrowUp && history.len() > 0 {
					if history_position.is_none() {
						history_position = Some(history.len() as i32-1);
					} else {
						if history_position.unwrap() != 0 {
							history_position = Some(history_position.unwrap()-1);
						}
						
					}
					term.move_cursor_left(cursor_pos as usize).unwrap();
					term.write_str(&history[history_position.unwrap() as usize]).unwrap();
					if inp.len() > history[history_position.unwrap() as usize].len() {
						term.write_str(&" ".repeat(inp.len() - history[history_position.unwrap() as usize].len())).unwrap();
						term.move_cursor_left(inp.len() - history[history_position.unwrap() as usize].len()).unwrap();
					}
					cursor_pos = history[history_position.unwrap() as usize].len() as i32;
					inp = history[history_position.unwrap() as usize].clone();
				} else if o == console::Key::ArrowDown && history_position.is_some() {
					if history_position.unwrap() != history.len() as i32-1 {
						history_position = Some(history_position.unwrap()+1);
					}
					term.move_cursor_left(cursor_pos as usize).unwrap();
					term.write_str(&history[history_position.unwrap() as usize]).unwrap();
					if inp.len() > history[history_position.unwrap() as usize].len() {
						term.write_str(&" ".repeat(inp.len() - history[history_position.unwrap() as usize].len())).unwrap();
						term.move_cursor_left(inp.len() - history[history_position.unwrap() as usize].len()).unwrap();
					}
					cursor_pos = history[history_position.unwrap() as usize].len() as i32;
					inp = history[history_position.unwrap() as usize].clone();
				} else if o == console::Key::Home {
					term.move_cursor_left(cursor_pos as usize).unwrap();
					cursor_pos = 0;
				} else if o == console::Key::End {
					term.move_cursor_right(inp.len() - cursor_pos as usize).unwrap();
					cursor_pos = inp.len() as i32;
				}
				// if inp.len() > 0 {
				// 	let mut available: bool = false;
				// 	for i in (0..history.len()).rev() {
				// 		if history[i].starts_with(&inp) && inp.len() < history[i].len() {
				// 			term.hide_cursor().unwrap();
				// 			term.move_cursor_right(inp.len()-cursor_pos as usize).unwrap();
				// 			print!("{}", console::style(&history[i][inp.len()..]).dim().to_string().as_str());
				// 			term.move_cursor_left(history[i].len()-cursor_pos as usize).unwrap();
				// 			term.show_cursor().unwrap();
				// 			available = true;
				// 			break;
				// 		}
				// 	}
				// 	if !available {
				// 		term.clear_to_end_of_screen().unwrap();
				// 	}
				// } else {
				// 	term.clear_to_end_of_screen().unwrap();
				// }

			},
			Err(e) => {
				print_error(line!(), e);
			}
		}
	}
}

fn prefix(term: &Term) {
	let current = match std::env::current_dir() {
		Ok(o) => {
			o.display().to_string()
		},
		Err(e) => {
			print_error(line!(), e);
			"???".to_string()
		}
	};
	let current = current.replace("\\", "/");
	if let Err(e) = term.write_str(console::style(current).blue().bright().to_string().as_str()) {
		print_error(line!(), e);
	}
	if let Err(e) = term.write_str(" > ") {
		print_error(line!(), e);
	}
}

fn is_debug() -> bool {
	return std::env::args().collect::<Vec<String>>().contains(&"--debug".to_string());
}

fn debug<S: std::fmt::Display>(s: S) {
	if is_debug() {
		println!("{}", s);
	}
}

fn update_path() {
	let start_time = std::time::Instant::now();
	let p: Vec<String> = match std::env::var("path") {
		Ok(o) => {
			o.split(";").map(|x| x.to_string()).collect::<Vec<String>>()
		},
		Err(e) => {
			print_error(line!(), format!("Unable to update path var: {}", e));
			return;
		}
	};

	let mut p_var = commands::path.lock().unwrap();
	p_var.clear();
	p_var.clone_from(&p);
	debug(format!("Path updated in {}s", start_time.elapsed().as_secs_f32()));
}

#[cfg(target_os = "linux")]
fn is_executable<S: Into<String>>(s: S) -> bool {
	let s: String = s.into();

	use std::os::linux::fs::MetadataExt;
	let d = std::fs::metadata(s);
	println!("{:?}", d);
	return false;

	// return vec!["exe", "bat", "com"].contains(&s.as_str());
}

#[cfg(target_os = "windows")]
fn is_executable<S: Into<String>>(s: S) -> bool {
	let s: String = s.into();
	return vec!["exe", "bat", "com"].contains(&s.as_str());
}

fn is_valid_exe<S: Into<String>>(file: S) -> Option<String> {

	let p = match commands::path.lock() {
		Ok(o) => {
			o
		},
		Err(e) => {
			print_error(line!(), format!("Could not grab path var: {}", e));
			return None;
		}
	};

	let file: String = file.into();
	let file = std::ffi::OsStr::new(&file);

	for i in p.iter() {
		let path = std::path::Path::new(i);
		if path.join(file).is_file() {
			if let Some(o) = path.extension() {
				if path.is_file() && is_executable(o.to_str().unwrap()) && (path.file_stem().unwrap() == file || path.file_name().unwrap() == file) {
					return Some(path.canonicalize().unwrap().to_str().unwrap().to_string());
				}
			}
		}
	}

	let path = std::path::Path::new(".");
	if path.join(file).is_file() {
		if let Some(o) = path.extension() {
			if path.is_file() && is_executable(o.to_str().unwrap()) && (path.file_stem().unwrap() == file || path.file_name().unwrap() == file) {
				return Some(path.canonicalize().unwrap().to_str().unwrap().to_string());
			}
		}
	}

	return None;
}

fn main() {
	debug("init terminal");
	let term = Term::stdout();
	if !is_debug() {
		if let Err(e) = term.clear_screen() {
			print_error(line!(), e);
		}
	}
	
	debug("init commands");
	let cmds = commands::create_commands();
	debug("init ctrl-c handler thread channel");
	let (sc2, rc2): (Sender<i16>, Receiver<i16>) = mpsc::channel();
	debug("init ctrl-c handler thread");
	if let Err(e) = ctrlc::set_handler(move || {
		if let Err(e2) = sc2.send(1) {
			print_error(line!(), e2);
		}
	}) {
		print_error(line!(), e);
	}

	update_path();

	debug("init history vector");
	let mut history: Vec<String> = Vec::with_capacity(100);
	loop {
		prefix(&term);
		let mut inp = read_command(&term, &history).trim().to_string();
		if inp.chars().any(|x| x.to_string() != " ") {
			if history.len() > 0 {
				if history.last().unwrap() != &inp {
					if history.len() == history.capacity() {
						history.remove(0);
					}
					history.push(inp.clone());
				}
			} else {
				if history.len() == history.capacity() {
					history.remove(0);
				}
				history.push(inp.clone());
			}
		}
		let d = commands::data.lock().unwrap();
		for var in d.keys() {
			inp = inp.replace(&format!("${}$", var), &d[var]);
		}
		drop(d);
		match shellwords::split(inp.as_str()) {
			Ok(parsed) => {
				if parsed.len() == 0 {
					continue;
				}
				let mut found = false;
				let command_clone = parsed[0].clone();
				if let Some(s) = is_valid_exe(parsed.clone()[0].clone()) {
					println!("{}", s);
				}
				for cmd in cmds.clone() {
					if cmd.name == parsed.clone()[0] {
						found = true;
						let (sc, rc): (Sender<i16>, Receiver<i16>) = mpsc::channel();
						debug("starting command thread");
						let start_time = std::time::Instant::now();
						let current_command = thread::spawn(move || -> Result<(), String> {
							return (cmd.func)(parsed, inp[inp.len().min(cmd.name.len()+1)..].to_string(), Some(rc));
						});
						while !current_command.is_finished() {
							if let Ok(msg) = rc2.try_recv() {
								if msg == 1 {
									if let Err(e) = sc.send(1) {
										print_error(line!(), e);
									}
								}
							}
						}
						if let Err(e) = current_command.join().unwrap() {
							print_error(line!(), e);
						};
						debug(format!("command took {} seconds to complete", start_time.elapsed().as_secs_f32()));
						break;
						// if let Err(e) = (cmd.func)(parsed.clone(), inp.clone()[inp.len().min(cmd.name.len()+1)..].to_string()) {
						// 	print_error(line!(), e);
						// };
					}
				}
				if !found {
					print_syntax_error(format!("Command {} does not exist", command_clone));
				}
			},
			Err(_) => {
				print_syntax_error("Mismatched quotes");
			}
		}
	}
	
}
