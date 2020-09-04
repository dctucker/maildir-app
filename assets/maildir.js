rpc = {
	invoke: function(cmd, args) {
		if( args === undefined ){
			args = {};
		}
		args.cmd = cmd;
		external.invoke(JSON.stringify(args));
	},
	init : function() {
		rpc.invoke('Init');
	},
	render: function(data) {
		/**
		//Object.assign(rpc.user_data, data);
		if(data.messages !== undefined){
			rpc.user_data.messages = data.messages;
		}
		if(data.current_mailbox !== undefined) {
			rpc.user_data.current_mailbox = data.current_mailbox;
		}
		//document.getElementById("mailbox_list").innerHTML = formatMailboxList(rpc.user_data.mailboxes);
		document.getElementById("mail").getElementsByTagName("tbody")[0].innerHTML = formatMessages(rpc.user_data.messages);
		*/
	},
	setMailbox: (i) => {
		document.getElementById("mail").getElementsByTagName("tbody")[0].innerHTML  = "<tr><td colspan='4'>Loading...</td></tr>";
		box = rpc.user_data.mailboxes[i];
		fetch('/mail/box' + box).then(resp => resp.json()).then(data => {
			if(data.messages !== undefined){
				rpc.user_data.messages = data.messages;
			}
			if(data.current_mailbox !== undefined) {
				rpc.user_data.current_mailbox = data.current_mailbox;
			}
			document.getElementById("mail").getElementsByTagName("tbody")[0].innerHTML = formatMessages(rpc.user_data.messages);
		});
		//rpc.invoke("SetMailbox", {path: box});
		rpc.user_data.current_mailbox = box;
	},
	setMessage: (i) => {
		rpc.user_data.current_message = i;
		fetchBody().then(resp => resp.json()).then(data => setPreview(data));
	},
};

window.onload = () => {
	rpc.user_data = {};
	rpc.init();
};

var content_loaded = function(){
	document.addEventListener('mouseover', function(event) {
		var hovered = event.target; // The actual element which was hovered.
		if (hovered.tagName !== 'A') {
			document.getElementById("href").innerHTML = "";
			return;
		}
		//console.log(hovered.href); // Do what we want here!
		document.getElementById("href").innerHTML = escapeHtml(hovered.href);
	});

	document.addEventListener('click', function(event) {
		if (event.target.tagName !== 'A') {
			return;
		}
		rpc.invoke('Browse', { url: event.target.href });
		return false;
	});

	document.onclick = function(event) {
		event = event || window.event;
		var element = event.target || event.srcElement;
		if( element.tagName == "A" ){
			rpc.invoke('Browse', { url: element.href });
			return false;
		}
	}

	//rpc.invoke('LoadMail');
	fetch('/mail/boxes').then(resp => resp.json()).then(data => {
		rpc.user_data.mailboxes = data;
		document.getElementById("mailbox_list").innerHTML = formatMailboxList(rpc.user_data.mailboxes);
	});
	/*
	fetch('/mail/messages/gmail/Arelí/cur/1597730753_1.1.18955d0b6ab1,U=3,FMD5=12cbe7315a369d36a41b7a83f277c87d:2,S')
		.then(resp => resp.json())
		.then(data => setPreview(data));
	*/
};


if (
	document.readyState === "complete" ||
	(document.readyState !== "loading" && !document.documentElement.doScroll)
) {
	content_loaded();
} else {
  document.addEventListener("DOMContentLoaded", content_loaded);
}

function escapeHtml(s) {
	if( s === undefined ){
		return "N/A";
	}
	return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
}

function toBinary(string) {
	const codeUnits = new Uint16Array(string.length);
	for (let i = 0; i < codeUnits.length; i++) {
		codeUnits[i] = string.charCodeAt(i);
	}
	return String.fromCharCode(...new Uint8Array(codeUnits.buffer));
}

function bodyUrl(loc) {
	path = '/mail/messages/';
	path += encodeURI(rpc.user_data.current_mailbox);
	path += '/';
	path += encodeURI(rpc.user_data.current_message);
	if( loc !== undefined ){
		str = loc.join(',');
		if( str == "" ){
			str = ",";
		}
		path += '?' + str;
	}
	return path;
}

function fetchBody(loc) {
	return fetch(bodyUrl(loc));
}

function formatSkeleton(elem, part, loc) {
	if( loc === undefined ){
		loc = [];
	}

	var div = document.createElement('div');
	div.setAttribute('id', 'part' + loc.join('.'));
	var ct = document.createElement('div');
	ct.classList.add('ctype');
	ct.innerHTML = part.ctype;
	div.appendChild(ct);
	div.classList.add('part');

	for( p in part.parts ){
		elem.appendChild( formatSkeleton(div, part.parts[p], loc.concat(p)) );
	}

	return div;
}

function formatBody(part, loc) {
	if( loc === undefined ){
		loc = [];
	}

	var ctype = part.ctype;
	var div = document.getElementById('part' + loc.join('.'));
	if( ctype.startsWith("image") ) {
		div.classList.add('image');
		var img = document.createElement('img');
		img.src = bodyUrl(loc);
		//ct = ctype.split(";")[0];
		//img.src = "data:" + ct + ";base64," + btoa(toBinary(part.body));
		div.appendChild(img);
	} else if( ctype.startsWith("text/") ){
		var wrapper = document.createElement('div');
		div.appendChild(wrapper);
		fetchBody(loc).then(resp => resp.text()).then(body => {
			if( ctype.startsWith("text/html") ) {
				wrapper.classList.add("html");
				wrapper.innerHTML = body;
			} else if( ctype.startsWith("text/plain") ) {
				wrapper.classList.add("content");
				wrapper.classList.add("plain");
				wrapper.innerHTML = escapeHtml(body);
				/*
				var section = document.createElement('section');
				section.classList.add('accordion');
				html = "";
				html += "<input type='checkbox' name='collapse' id='handle1'>";
				html += "<label for='handle1'>Content-Type: text/plain</label>";
				html += "<div class='content plain'>" + escapeHtml(body) + "</div>";
				section.innerHTML = html;
				wrapper.appendChild(section);
				*/
			}
		});
	} else {
		var link = document.createElement('div');
		var a = document.createElement('a');
		a.appendChild(document.createTextNode("Attachment: " + ctype));
		a.href = bodyUrl(loc);
		link.appendChild(a);
		div.appendChild(link);
	}

	for( p in part.parts ){
		formatBody(part.parts[p], loc.concat(p));
	}
}

function formatHeaders(headers) {
	html = "";
	keys = Object.keys(headers).sort();
	for( h in keys ){
		val = escapeHtml(headers[keys[h]]);
		if( keys[h][0] == "x" || keys[h][0] == "X" ){
			html += "<tr><th>"+keys[h]+": </th><td>"+val+"</td></tr>";
		} else {
			html += "<tr><th style='text-align:right'>"+keys[h]+": </th><td>"+val+"</td></tr>";
		}
	}
	return html;
}

function formatMailboxList(list) {
	html = "";
	for( i in list ){
		box = list[i];
		html += "<li onclick='rpc.setMailbox("+i+")'>";
		html += box;
		html += "</li>";
	}
	return html;
}

function formatMessages(messages) {
	html = "";
	for( i in messages ){
		m = messages[i];
		html += "<tr class='"+(m['new'] == 1 ? "new" : "")+"' onclick='rpc.setMessage(\""+i+"\")' >";
		html += "<td><input type='checkbox' value='"+i+"' /></td>";
		html += "<td>" + escapeHtml(m.From   ) + "</td>";
		html += "<td>" + escapeHtml(m.Subject) + "</td>";
		html += "<td>" + escapeHtml(m.Date   ) + "</td>";
		html += "</tr>";
	}
	return html;
}

function setPreview(data) {
	rpc.preview_data = data;
	document.getElementById("headers_label").innerHTML = data.headers["From"] + " &mdash; " + data.headers["Subject"];
	document.getElementById("headers").innerHTML = formatHeaders(data.headers);
	document.getElementById("body").innerHTML = "";
	outer = formatSkeleton(document.getElementById("body"), data);
	document.getElementById('body').appendChild(outer);
	formatBody(data);
}
