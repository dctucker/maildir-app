extern crate chrono;
use chrono::prelude::*;

extern crate serde_derive;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub const MAILDIR_PATH: &str = "E:/Maildir";


#[derive(Clone,Debug,Serialize)]
pub struct UserData {
	pub mailboxes: Vec<String>,
	pub current_mailbox: String,
	pub messages: HashMap<String,MessageHeaders>,
	pub current_message: String,
}
impl UserData {
	pub fn new() -> UserData {
		UserData {
			current_mailbox: "".to_string(),
			current_message: "".to_string(),
			mailboxes: vec![],
			messages: HashMap::new(),
		}
	}
	pub fn load_mailboxes(mut self) -> Self {
		let full_path = MAILDIR_PATH.clone();
		self.mailboxes = walkdir::WalkDir::new(full_path.clone())
			.into_iter()
			.filter_entry(|e| e.file_type().is_dir())
			.map(|e| format_filename(e.unwrap().into_path().display().to_string(), full_path))
			.filter(|s| ! (s.ends_with("/new") || s.ends_with("/cur") || s.ends_with("/tmp") || s.len() == 0) )
			.collect::<Vec<String>>();
		self
	}
	pub fn set_current_mailbox(&mut self, path: String) -> &Self {
		let full_path = format!("{}/{}", MAILDIR_PATH, path);
		let dir = maildir::Maildir::from(full_path.clone());
		self.messages = map_messages( dir.list_new(), full_path.clone(), 1 );
		self.messages.extend( map_messages( dir.list_cur(), full_path.clone(), 0 ) );
		self.current_mailbox = path;
		self
	}
}

#[derive(Deserialize)]
#[serde(tag = "cmd")]
pub enum Cmd {
	Init {},
	LoadMail {},
	SetMailbox { path: String },
	Browse { url: String },
	Exit {},
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

extern crate maildir;
use mailparse::ParsedMail;

type MessageHeaders = HashMap<String,String>;

#[derive(Serialize, Clone)]
pub struct Message {
	pub headers: MessageHeaders,
	pub parts: Vec<Message>,
	pub ctype: String,
	pub body: Vec<u8>,
}

fn format_filename(s: String, full_path: &str) -> String {
	s.replace("\\","/")
		.replace(full_path,"")
		.replace("\u{f022}",":")
}
fn format_date(s: String) -> String {
	let date = mailparse::dateparse(&s).unwrap();
	let date: DateTime<Local> = Utc.timestamp(date, 0).into();
	date.format("%Y-%m-%d %H:%M:%S").to_string()
}
fn format_headers(parsed: Vec<mailparse::MailHeader>, new: usize) -> HashMap<String,String> {
	let mut headers = parsed.iter().filter(|h| match h.get_key().as_str() {
			"From" | "Date" | "Subject" => true,
			_ => false,
		})
		.map(|h| { (h.get_key(), h.get_value()) })
		.collect::<HashMap<String,String>>();
	*(headers.get_mut("Date").unwrap()) = format_date(headers["Date"].clone());
	headers.entry("new".to_string()).or_insert(format!("{}", new));
	headers
}
fn map_messages(list: maildir::MailEntries, full_path: String, new: usize) -> HashMap<String,HashMap<String,String>> {
	list.map(|e| {
		let mut e = e.unwrap();
		let real_path = e.path();
		let path = format_filename(real_path.display().to_string(), &full_path);
		let parsed = e.headers().unwrap();
		let headers = format_headers(parsed, new);
		(path, headers)
	}).collect::<HashMap<_,_>>()
}


impl Message {
	pub fn from_parsed_mail(parsed: &ParsedMail<'_>) -> Self {
		Message {
			headers: parsed.headers.iter().map(|h| { (h.get_key(), h.get_value()) }).collect(),
			body: if parsed.ctype.mimetype.starts_with("text/html") {
					sanitize(parsed.get_body().unwrap()).into_bytes()
				} else {
					parsed.get_body_raw().unwrap()
				},
			ctype: parsed.ctype.mimetype.clone(),
			parts: parsed.subparts.iter().map(|s| { Message::from_parsed_mail(s) }).collect(),
		}
	}
	pub fn skeleton(&self) -> Message {
		Message {
			headers: self.headers.clone(),
			ctype: self.ctype.clone(),
			parts: self.parts.iter().map(|s| s.skeleton() ).collect(),
			body: vec![],
		}
	}
}

use html_sanitizer::TagParser;
fn sanitize(input: String) -> String {
	let mut tag_parser = TagParser::new(&mut input.as_bytes());
	tag_parser.walk(|tag| {
		if tag.name == "html" || tag.name == "body" {
			tag.ignore_self(); // ignore <html> and <body> tags, but still parse their children
		} else if tag.name == "head" || tag.name == "script" || tag.name == "style" {
			tag.ignore_self_and_contents(); // Ignore <head>, <script> and <style> tags, and all their children
		} else if tag.name == "a" {
			tag.allow_attribute(String::from("href")); // Allow specific attributes
		} else if tag.name == "img" {
			tag.allow_attribute(String::from("src"));
			tag.allow_attribute(String::from("width"));
			tag.allow_attribute(String::from("height"));
			//tag.rewrite_as(String::from("<b>Images not allowed</b>")); // Completely rewrite tags and their children
		} else {
			tag.allow_attribute(String::from("style")); // Allow specific attributes
		}
	})
}
