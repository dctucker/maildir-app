//#![windows_subsystem = "windows"]
use app::{Cmd, Message, MAILDIR_PATH, UserData};

extern crate actix_rt;
extern crate actix_web;
extern crate futures;
extern crate mime_guess;
extern crate rust_embed;

#[macro_use]
extern crate cached;
use cached::SizedCache;

extern crate serde_json;

use std::fs;
use std::io::prelude::*;

use std::{borrow::Cow, sync::mpsc, thread};
use rust_embed::RustEmbed;
use actix_web::{body::Body, web, App, HttpRequest, HttpResponse, HttpServer};

#[derive(RustEmbed)]
#[folder = "../assets"]
struct Asset;

fn assets(req: HttpRequest) -> HttpResponse {
	let path = if req.path() == "/" {
		"index.html"
	} else {
		&req.path()[1..] // trim leading '/'
	};

	// query the file from embedded asset with specified path
	match Asset::get(path) {
		Some(content) => {
			let body: Body = match content {
				Cow::Borrowed(bytes) => bytes.into(),
				Cow::Owned(bytes) => bytes.into(),
			};
			HttpResponse::Ok()
				.content_type(mime_guess::from_path(path).first_or_octet_stream().as_ref())
				.body(body)
		}
		None => HttpResponse::NotFound().body("404 Not Found"),
	}
}

cached_result!{
	MESSAGES: SizedCache<String, Message> = SizedCache::with_size(50);
	fn load_message(path: String) -> Result<Message, ()> = {
		let path = path.replace(":", "\u{f022}");
		let path = format!("{}/{}", MAILDIR_PATH, path);
		if let Ok(mut f) = fs::File::open(path.clone()) {
			let mut d = Vec::<u8>::new();
			f.read_to_end(&mut d).unwrap();
			let parsed = mailparse::parse_mail(&d).unwrap();
			let msg = Message::from_parsed_mail(&parsed);
			Ok(msg)
		} else {
			eprintln!("Unable to open {}", path.clone());
			Err(())
		}
	}
}

fn traverse_message<'a>(msg: &'a Message, loc: &[usize]) -> Result<&'a Message, ()> {
	if loc.len() == 1 {
		return Ok(&msg.parts[loc[0]]);
	}
	traverse_message(&msg.parts[loc[0]], &loc[1..])
}


fn get_mail(req: HttpRequest) -> HttpResponse {
	let path = req.match_info().get("path").unwrap();
	if let Ok(msg) = load_message(path.to_string()) {
		let query = req.query_string();
		if query.len() == 0 {
			println!("{}", serde_json::to_string_pretty(&msg.skeleton().parts).unwrap());
			return HttpResponse::Ok().json(msg.skeleton());
		} else if query == "," {
			return HttpResponse::Ok()
				.content_type(msg.ctype.clone())
				.body(msg.body.clone())
		}
		println!("query = {}", query.clone());
		let loc = query.split(",").map(|e| usize::from_str_radix(e, 10).unwrap()).collect::<Vec<usize>>();
		if let Ok(m) = traverse_message(&msg, &loc) {
			HttpResponse::Ok()
				.content_type(m.ctype.clone())
				.body(m.body.clone())
		} else {
			eprintln!("404 traversal {}", path);
			HttpResponse::NotFound().body("Not Found")
		}
	} else {
		eprintln!("404 Unable to load message {}", path.to_string());
		HttpResponse::NotFound().body("Not Found")
	}
}

use std::sync::mpsc::Sender;
pub struct Server {
	pub server: actix_web::dev::Server,
	pub port: u16,
	pub thread: Option<std::thread::JoinHandle<()>>,
	pub tx: Sender<Cmd>, //, rx: Receiver<Server>,
}
impl Server {
	pub fn join(&mut self) {
		self.thread.take().unwrap().join().unwrap();
	}
}

use std::sync::Mutex;
use actix_web::web::Data;
pub fn run_server() -> Result<Server, String> {
	let (server_tx, server_rx) = mpsc::channel();
	let (control_tx, control_rx) = mpsc::channel();
	let tx = server_tx.clone();

	let thread = thread::spawn(move || {
		let sys = actix_rt::System::new("actix-example");

		let server = HttpServer::new(|| {
				let user_data = Data::new(
					Mutex::new(
						UserData::new().load_mailboxes()
					)
				);
				App::new()
					.register_data(user_data.clone())
					//.route("/mail/message.json", web::get().to(get_mail))
					//.route("/mail/box/{dir:.*}/{msg_id}/headers.json", web::get().to(mail_headers))
					.route("/mail/messages/{path:.*}", web::get().to(get_mail))
					.route("/mail/boxes", web::get().to(|req: HttpRequest| {
						let data = req.get_app_data::<Mutex<UserData>>().unwrap().lock().unwrap().mailboxes.clone();
						HttpResponse::Ok().json(data)
					}))
					.route("/mail/box/{path:.*}", web::get().to(|req: HttpRequest| {
						let path = req.match_info().get("path").unwrap();
						let data : Data<Mutex<UserData>> = req.get_app_data().unwrap();
						let mut data = data.lock().unwrap();
						data.set_current_mailbox(path.to_string());
						HttpResponse::Ok().json(data.clone())
					}))
					.route("*", web::get().to(assets))
			})
			.bind("127.0.0.1:0")
			.unwrap();

		let port = server.addrs().first().unwrap().port();
		let server = server.start();
		let server2 = server.clone();

		let _ = tx.clone().send(Server {
			server: server,
			port: port,
			thread: None,
			tx: control_tx.clone(),
		});
		let _ = sys.run();

		loop {
			use futures::future::Future;
			use app::Cmd::*;
			if let Ok(cmd) = control_rx.recv() {
				match cmd {
					Exit {} => {
						let _ = server2.clone().stop(true).wait();
					},
					_ => {},
				};
			}
		}
	});
	if let Ok(mut server) = server_rx.recv() {
		server.thread = Some(thread);
		return Ok(server);
	} else {
		return Err("Server didn't start".to_string());
	}
}
