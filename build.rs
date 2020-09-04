
#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
	if cfg!(target_os = "windows") {
		let mut res = winres::WindowsResource::new();
		res.set_icon("assets/favicon.ico");
		res.set_manifest(r#"
			<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
				<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
					<security>
						<requestedPrivileges>
						</requestedPrivileges>
					</security>
				</trustInfo>
			</assembly>
		"#);
		res.compile().unwrap();
	}
}


#[cfg(unix)]
fn main() {
}
