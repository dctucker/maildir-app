extern crate web_view;
use web_view::*;
use server;
use app::UserData;
use std::sync::mpsc::Sender;

fn render(webview: &mut WebView<UserData>) -> WVResult {
	let call = {
		let data = webview.user_data();
		println!("{:#?}", data);
		format!("rpc.render({})", serde_json::to_string(data).unwrap())
	};
	webview.eval(&call)
}

fn run_webview(port: u16, server_tx: Sender<app::Cmd>) {
	let user_data = UserData::new();
	let webview = web_view::builder()
		.title("Mail time")
		.content(Content::Url(format!("http://127.0.0.1:{}", port)))
		.size(1024, 768)
		.resizable(true)
		.debug(true)
		.user_data(user_data)
		.invoke_handler(|webview, arg| {
			use app::Cmd::*;
			if let Ok(cmd) = serde_json::from_str(arg) {
				let data = webview.user_data_mut();
				match cmd {
					Init {} => {
						render(webview).unwrap();
					},
					LoadMail {} => {},
					SetMailbox { path } => {
						data.set_current_mailbox(path);
						webview.eval(&format!("rpc.render({})", serde_json::json!({
							"current_mailbox": webview.user_data().current_mailbox,
							"messages": webview.user_data().messages,
						}))).unwrap();
					},
					Browse { url } => {
						webbrowser::open(&url).unwrap();
					},
					Exit {} => {
						let _ = server_tx.send(cmd).unwrap();
						webview.exit();
					},
				};
			} else {
				eprintln!("Invalid command: {}", arg);
			}
			Ok(())
		})
	.build().unwrap();
	let _ = webview.run().unwrap();
}

fn main() {
	let server = server::run_server().unwrap();
	println!("Port: {}", server.port);

	run_webview(server.port, server.tx.clone());
}
