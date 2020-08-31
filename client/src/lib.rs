#![deny(warnings)]
use log::trace;
//use sauron::html::attributes::attr;
use sauron::html::text;
use sauron::prelude::*;
use sauron::{Cmd, Component, Node, Program};

#[derive(Debug)]
pub enum Msg {
    Click,
	SetMailbox(usize),
}

//use std::collections::HashMap;
//type MessageHeaders = HashMap<String,String>;

pub struct App {
	mailboxes: Vec<String>,
	/*
	current_mailbox: String,
	messages: HashMap<String,MessageHeaders>,
	current_message: String,
	*/
}

impl App {
    pub fn new() -> Self {
        App {
			mailboxes: vec![],
			/*
			current_mailbox: String::from(""),
			messages: HashMap::new(),
			current_message: String::from(""),
			*/
		}
    }
}

impl Component<Msg> for App {
    fn view(&self) -> Node<Msg> {
		let mailboxes = &self.mailboxes;
        node! {
            <nav>
				<ul id="mailbox_list">
					{ for i in 0..mailboxes.len() {
						mailbox_li(mailboxes[i].clone(), i)
					}}
				</ul>
			</nav>
        }
		/*
                <h1>"Minimal example"</h1>
                <div class="some-class" id="some-id" {attr("data-id", 1)}>
                    <input class="client"
                            type_="button"
                            value="Click me!"
                            key=1
                            on_click={|_| {
                                trace!("Button is clicked");
                                Msg::Click
                            }}
                    />
                    <div>{text(format!("Clicked: {}", self.click_count))}</div>
                    <input type_="text" value={self.click_count}/>
                </div>
            </main>
		*/
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        trace!("App is updating with msg: {:?}", msg);
        match msg {
            //Msg::Click => self.click_count += 1,
			Msg::SetMailbox(_i) => {
			},
			_ => {},
        }
        Cmd::none()
    }
}

fn mailbox_li(path: String, i: usize) -> Node<Msg> {
	node! {
		<li on_click={move |_| Msg::SetMailbox(i) }>
			{ text(format!("{}", path)) }
		</li>
	}
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    console_error_panic_hook::set_once();
    Program::mount_to_body(App::new());
}
