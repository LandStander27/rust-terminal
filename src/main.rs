use console::Term;
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::process::Command as Cmd;

#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;

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
	let p: Vec<String> = match std::env::var("PATH") {
		Ok(o) => {
			if cfg!(windows) {
				o.split(";").map(|x| x.to_string()).collect::<Vec<String>>()
			} else {
				o.split(":").map(|x| x.to_string()).collect::<Vec<String>>()
			}
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

	let d = std::fs::metadata(s).unwrap().mode();
	if d & 0o111 != 0 {
		return true;
	}
	return false;

	// return vec!["exe", "bat", "com"].contains(&s.as_str());
}

#[cfg(target_os = "windows")]
fn executables() -> Vec<&'static str> {
	return vec!["exe", "bat", "com", "cmd"];
}

#[cfg(target_os = "windows")]
fn is_executable<S: Into<String>>(s: S) -> bool {
	let s: String = s.into();
	return executables().iter().any(|x| s.ends_with(x));
}

fn is_valid_exe_in_path<S: Into<String>>(file: S) -> Option<String> {

	let p = match commands::path.lock() {
		Ok(o) => {
			o
		},
		Err(e) => {
			print_error(line!(), format!("Could not grab path var: {}", e));
			return None;
		}
	};

	let mut file: String = file.into();

	for i in p.iter() {
		let path = std::path::Path::new(i);
		if !path.join(&file).is_file() {
			if cfg!(windows) {
				let mut found = false;
				for ext in executables() {
					if path.join(format!("{}.{}", file, ext)).is_file() {
						file.push('.');
						file.push_str(ext);
						found = true;
						break;
					}
				}
				if !found {
					continue;
				}
			} else {
				continue;
			}
		}
		let path: &std::path::Path = &path.join(&file);
		if is_executable(path.to_str().unwrap()) && path.file_name().unwrap().to_str().unwrap() == file {
			return Some(path.to_str().unwrap().to_string());
		}
	}

	return None;
}

fn is_valid_exe_in_current_path<S: Into<String>>(file: S) -> Option<String> {

	let mut file: String = file.into();

	let path = std::path::Path::new(".");
	if !path.join(&file).is_file() {
		if cfg!(windows) {
			let mut found = false;
			for ext in executables() {
				if path.join(format!("{}.{}", file, ext)).is_file() {
					file.push('.');
					file.push_str(ext);
					found = true;
					break;
				}
			}
			if !found {
				return None;
			}
		}
		return None;
	}
	let path: &std::path::Path = &path.join(&file);
	if is_executable(path.to_str().unwrap()) && path.file_name().unwrap().to_str().unwrap() == file {
		return Some(path.to_str().unwrap().to_string());
	}

	return None;
}

fn run_command(inp: String, rc2: Option<&Receiver<i16>>, cmds: &Vec<commands::Command<'static>>) {
	match shellwords::split(inp.as_str()) {
		Ok(parsed) => {
			if parsed.len() == 0 {
				return;
			}
			let parsed_clone = parsed.clone();
			if let Some(s) = is_valid_exe_in_current_path(parsed[0].clone()) {
				debug("creating command thread");
				let start_time = std::time::Instant::now();
				let mut c = match Cmd::new(s).args(&parsed[1..]).spawn() {
					Ok(o) => {
						o
					},
					Err(e) => {
						println!("Error: {}", e);
						return;
					}
				};
				let mut showed_error = false;
				loop {
					match c.try_wait() {
						Ok(Some(_)) => {
							break;
						},
						Ok(None) => (),
						Err(e) => {
							if !showed_error {
								print_error(line!(), format!("Error getting status of process: {}", e));
							}
							showed_error = true;
						}
					}
					if rc2.is_some() {
						if let Ok(msg) = rc2.unwrap().try_recv() {
							if msg == 1 {
								break;
							}
						}
					}

				}
				c.kill().unwrap();
				debug(format!("command took {} seconds to complete", start_time.elapsed().as_secs_f32()));
			} else {
				let mut found = false;
				for cmd in cmds {
					if cmd.name == parsed.clone()[0] {
						found = true;
						let (sc, rc): (Sender<i16>, Receiver<i16>) = mpsc::channel();
						debug("starting command thread");
						let start_time = std::time::Instant::now();
						let cmd = cmd.clone();

						let current_command = if rc2.is_some() {
							thread::spawn(move || -> Result<(), String> {
								return (cmd.func)(parsed, inp[inp.len().min(cmd.name.len()+1)..].to_string(), Some(rc));
							})
						} else {
							thread::spawn(move || -> Result<(), String> {
								return (cmd.func)(parsed, inp[inp.len().min(cmd.name.len()+1)..].to_string(), None);
							})
						};

						while !current_command.is_finished() {
							if rc2.is_some() {
								if let Ok(msg) = rc2.unwrap().try_recv() {
									if msg == 1 {
										if let Err(e) = sc.send(1) {
											print_error(line!(), e);
										}
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
					if let Some(s) = is_valid_exe_in_path(parsed_clone[0].clone()) {
						debug("creating command thread");
						let start_time = std::time::Instant::now();
						let mut c = match Cmd::new(s).args(&parsed_clone[1..]).spawn() {
							Ok(o) => {
								o
							},
							Err(e) => {
								println!("Error: {}", e);
								return;
							}
						};
						let mut showed_error = false;
						loop {
							match c.try_wait() {
								Ok(Some(_)) => {
									break;
								},
								Ok(None) => (),
								Err(e) => {
									if !showed_error {
										print_error(line!(), format!("Error getting status of process: {}", e));
									}
									showed_error = true;
								}
							}
							if rc2.is_some() {
								if let Ok(msg) = rc2.unwrap().try_recv() {
									if msg == 1 {
										break;
									}
								}
							}
						}
						c.kill().unwrap();
						debug(format!("command took {} seconds to complete", start_time.elapsed().as_secs_f32()));
					} else {
						print_syntax_error(format!("{} does not exist as a command or executable", parsed_clone[0]));
					}
				}
			}
		},
		Err(_) => {
			print_syntax_error("Mismatched quotes");
		}
	}
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
		let d = commands::path.lock().unwrap();
		if cfg!(windows) {
			inp = inp.replace("$PATH$", &d.join(";"));
		} else {
			inp = inp.replace("$PATH$", &d.join(":"));
		}
		drop(d);
		run_command(inp, Some(&rc2), &cmds);
	}
	
}
