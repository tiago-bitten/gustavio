pub const HTML: &str = r##"<!DOCTYPE html>
<html lang="pt-BR">
<head>
<meta charset="UTF-8">
<title>Gustavio</title>
<style>
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&display=swap');

* { margin:0; padding:0; box-sizing:border-box; }

:root {
  --bg:       #0a0a0a;
  --surface:  #111111;
  --surface2: #181818;
  --border:   #1e1e1e;
  --text:     #b0b0b0;
  --bright:   #d4d4d4;
  --dim:      #484848;
  --green:    #39ff14;
  --green-d:  #1a3a10;
  --cyan:     #00d4ff;
  --cyan-d:   #0a1a22;
  --yellow:   #f0c000;
  --red:      #ff4444;
  --magenta:  #e040e0;
  --blue:     #5c7cfa;
}

body {
  font-family: 'JetBrains Mono', 'Cascadia Code', 'Fira Code', 'SF Mono', Consolas, monospace;
  font-size: 13px;
  background: var(--bg);
  color: var(--text);
  height: 100vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  user-select: none;
  line-height: 1.55;
}

/* ── TOP BAR ─────────────────────────────────── */
#topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 14px;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  min-height: 38px;
  -webkit-app-region: drag;
}
#topbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}
#logo {
  font-weight: 700;
  font-size: 13px;
  color: var(--green);
  text-shadow: 0 0 8px rgba(57,255,20,0.3);
  letter-spacing: 2px;
}
#my-info {
  font-size: 11px;
  color: var(--dim);
}
#topbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
  -webkit-app-region: no-drag;
}
.tb-btn {
  background: none;
  border: 1px solid transparent;
  color: var(--dim);
  font-family: inherit;
  font-size: 11px;
  padding: 3px 8px;
  cursor: pointer;
  transition: all 0.15s;
  border-radius: 2px;
}
.tb-btn:hover { color: var(--bright); border-color: var(--border); }
.tb-btn.active { color: var(--green); border-color: var(--green-d); background: var(--green-d); }
.tb-btn.pin-active { color: var(--yellow); border-color: #332a00; background: #1a1500; }

/* ── MAIN LAYOUT ─────────────────────────────── */
#main { display: none; flex: 1; overflow: hidden; }
#main.visible { display: flex; }

/* ── SIDEBAR ─────────────────────────────────── */
#sidebar {
  width: 200px;
  min-width: 200px;
  background: var(--surface);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.sb-section {
  font-size: 10px;
  font-weight: 600;
  color: var(--dim);
  letter-spacing: 2px;
  text-transform: uppercase;
  padding: 14px 12px 6px;
}
.sb-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  cursor: pointer;
  transition: background 0.1s;
  white-space: nowrap;
  overflow: hidden;
}
.sb-item:hover { background: var(--surface2); }
.sb-item.active { background: var(--surface2); border-right: 2px solid var(--green); }
.sb-dot {
  width: 6px; height: 6px;
  border-radius: 50%;
  background: var(--green);
  flex-shrink: 0;
  box-shadow: 0 0 4px rgba(57,255,20,0.5);
}
.sb-name {
  font-size: 12px;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
}
.sb-hash {
  color: var(--dim);
  font-size: 12px;
  flex-shrink: 0;
}
.sb-badge {
  margin-left: auto;
  background: var(--green);
  color: #000;
  font-size: 9px;
  font-weight: 700;
  padding: 0 5px;
  border-radius: 2px;
  display: none;
}
.sb-badge.vis { display: inline; }
#sb-list { flex: 1; overflow-y: auto; }
#sb-list::-webkit-scrollbar { width: 4px; }
#sb-list::-webkit-scrollbar-thumb { background: var(--border); }

#new-group {
  padding: 10px 12px;
  border-top: 1px solid var(--border);
  font-size: 11px;
  color: var(--dim);
  cursor: pointer;
  transition: color 0.15s;
}
#new-group:hover { color: var(--cyan); }

/* ── CHAT AREA ───────────────────────────────── */
#chat-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
#chat-header {
  display: none;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--border);
  background: var(--surface);
  font-size: 12px;
}
#chat-header.vis { display: flex; }
#ch-name { color: var(--cyan); font-weight: 600; }
#ch-status { color: var(--dim); font-size: 11px; }

#empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  gap: 8px;
}
#empty-state .ascii {
  color: var(--green);
  font-size: 11px;
  text-shadow: 0 0 10px rgba(57,255,20,0.2);
  text-align: center;
  line-height: 1.3;
}
#empty-state .hint { color: var(--dim); font-size: 11px; }

/* ── MESSAGES ────────────────────────────────── */
#messages {
  flex: 1;
  overflow-y: auto;
  padding: 12px 0;
  display: none;
  flex-direction: column;
}
#messages.vis { display: flex; }
#messages::-webkit-scrollbar { width: 4px; }
#messages::-webkit-scrollbar-thumb { background: var(--border); }

.msg-line {
  display: flex;
  padding: 2px 16px;
  gap: 0;
  transition: background 0.1s;
}
.msg-line:hover { background: rgba(255,255,255,0.02); }

.msg-time {
  color: var(--dim);
  font-size: 11px;
  min-width: 44px;
  flex-shrink: 0;
  padding-top: 1px;
}
.msg-sep {
  color: var(--border);
  margin: 0 8px;
  flex-shrink: 0;
  padding-top: 1px;
}
.msg-user {
  font-weight: 600;
  min-width: 80px;
  flex-shrink: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.msg-user.me { color: var(--green); }
.msg-content {
  color: var(--bright);
  word-break: break-word;
  flex: 1;
  user-select: text;
}
.msg-status {
  color: var(--dim);
  font-size: 10px;
  margin-left: 6px;
  flex-shrink: 0;
}

/* Censorship mode */
body.censored .msg-content {
  filter: blur(6px);
  transition: filter 0.2s ease;
  cursor: default;
}
body.censored .msg-content:hover {
  filter: none;
  cursor: text;
}

/* System messages */
.msg-system {
  padding: 4px 16px;
  color: var(--dim);
  font-size: 11px;
  font-style: italic;
}

/* ── INPUT ───────────────────────────────────── */
#input-area {
  display: none;
  align-items: center;
  padding: 8px 16px;
  gap: 0;
  border-top: 1px solid var(--border);
  background: var(--surface);
}
#input-area.vis { display: flex; }
#input-prompt {
  color: var(--green);
  font-weight: 700;
  margin-right: 8px;
  font-size: 14px;
  text-shadow: 0 0 6px rgba(57,255,20,0.3);
}
#msg-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--bright);
  font-family: inherit;
  font-size: 13px;
  caret-color: var(--green);
}
#msg-input::placeholder { color: var(--dim); }

/* ── SETUP SCREEN ────────────────────────────── */
#setup {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  gap: 20px;
}
#setup .ascii-logo {
  color: var(--green);
  font-size: 10px;
  text-shadow: 0 0 12px rgba(57,255,20,0.3);
  text-align: center;
  line-height: 1.2;
}
#setup .sub { color: var(--dim); font-size: 11px; }
#setup-input {
  background: var(--surface);
  border: 1px solid var(--border);
  padding: 10px 16px;
  font-family: inherit;
  font-size: 14px;
  color: var(--bright);
  outline: none;
  width: 260px;
  text-align: center;
  caret-color: var(--green);
  border-radius: 2px;
}
#setup-input:focus { border-color: var(--green); }
#setup-btn {
  background: var(--green);
  color: #000;
  border: none;
  padding: 8px 28px;
  font-family: inherit;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  letter-spacing: 1px;
  border-radius: 2px;
}
#setup-btn:hover { opacity: 0.85; }

/* ── GROUP MODAL ─────────────────────────────── */
#modal-bg {
  display: none;
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.7);
  z-index: 100;
  align-items: center;
  justify-content: center;
}
#modal-bg.vis { display: flex; }
#modal {
  background: var(--surface);
  border: 1px solid var(--border);
  padding: 20px;
  width: 320px;
  max-height: 70vh;
  overflow-y: auto;
}
#modal h3 {
  font-size: 13px;
  color: var(--cyan);
  margin-bottom: 14px;
  font-weight: 600;
  letter-spacing: 1px;
}
#modal .lbl {
  font-size: 10px;
  color: var(--dim);
  letter-spacing: 1px;
  text-transform: uppercase;
  margin-bottom: 6px;
}
#modal input[type="text"] {
  width: 100%;
  background: var(--bg);
  border: 1px solid var(--border);
  padding: 8px 10px;
  font-family: inherit;
  font-size: 12px;
  color: var(--bright);
  outline: none;
  margin-bottom: 14px;
  caret-color: var(--green);
}
#modal input[type="text"]:focus { border-color: var(--cyan); }
.m-check {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 0;
  font-size: 12px;
}
.m-check input { accent-color: var(--green); }
.m-check label { cursor: pointer; color: var(--text); }
.m-actions {
  display: flex;
  gap: 8px;
  margin-top: 16px;
  justify-content: flex-end;
}
.m-actions button {
  border: none;
  padding: 6px 16px;
  font-family: inherit;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border-radius: 2px;
}
.btn-x { background: var(--surface2); color: var(--dim); }
.btn-x:hover { color: var(--text); }
.btn-ok { background: var(--green); color: #000; }
.btn-ok:hover { opacity: 0.85; }
</style>
</head>
<body>

<!-- ── SETUP ──────────────────────────────────── -->
<div id="setup">
  <pre class="ascii-logo">
 ██████  ██    ██ ███████ ████████  █████  ██    ██ ██  ██████
██       ██    ██ ██         ██    ██   ██ ██    ██ ██ ██    ██
██   ███ ██    ██ ███████    ██    ███████ ██    ██ ██ ██    ██
██    ██ ██    ██      ██    ██    ██   ██  ██  ██  ██ ██    ██
 ██████   ██████  ███████    ██    ██   ██   ████   ██  ██████
  </pre>
  <div class="sub">chat local via rede &mdash; sem servidor, sem internet</div>
  <input type="text" id="setup-input" placeholder="seu nome..." maxlength="20" autofocus>
  <button id="setup-btn" onclick="submitName()">ENTRAR</button>
</div>

<!-- ── TOP BAR ────────────────────────────────── -->
<div id="topbar" style="display:none">
  <div id="topbar-left">
    <span id="logo">GUSTAVIO</span>
    <span id="my-info"></span>
  </div>
  <div id="topbar-right">
    <button class="tb-btn" id="btn-censor" onclick="toggleCensor()" title="Modo censura (Ctrl+Shift+X)">CENSURA</button>
    <button class="tb-btn pin-active" id="btn-pin" onclick="togglePin()" title="Fixar janela">PIN</button>
  </div>
</div>

<!-- ── MAIN ───────────────────────────────────── -->
<div id="main">
  <div id="sidebar">
    <div id="sb-list">
      <div class="sb-section">peers</div>
      <div id="peer-list"></div>
      <div class="sb-section">grupos</div>
      <div id="group-list"></div>
    </div>
    <div id="new-group" onclick="openModal()">+ novo grupo</div>
  </div>
  <div id="chat-area">
    <div id="empty-state">
      <pre class="ascii">
   _____
  / ____/
 / /  __
/ /__/ /
\_____/  </pre>
      <div class="hint">selecione uma conversa na sidebar</div>
    </div>
    <div id="chat-header">
      <span id="ch-name"></span>
      <span id="ch-status"></span>
    </div>
    <div id="messages"></div>
    <div id="input-area">
      <span id="input-prompt">&gt;</span>
      <input type="text" id="msg-input" placeholder="digite..." autocomplete="off">
    </div>
  </div>
</div>

<!-- ── GROUP MODAL ────────────────────────────── -->
<div id="modal-bg">
  <div id="modal">
    <h3>// NOVO GRUPO</h3>
    <div class="lbl">nome</div>
    <input type="text" id="grp-name" placeholder="nome do grupo..." maxlength="24">
    <div class="lbl">membros</div>
    <div id="modal-members"></div>
    <div class="m-actions">
      <button class="btn-x" onclick="closeModal()">CANCELAR</button>
      <button class="btn-ok" onclick="createGroup()">CRIAR</button>
    </div>
  </div>
</div>

<script>
// ── State ──────────────────────────────────────
var myPeerId = null, myUsername = null;
var peers = [], groups = [];
var currentChat = null;
var unread = {};
var pinned = true;

var USER_COLORS = [
  'var(--cyan)', '#e040e0', '#f0c000', '#5c7cfa',
  '#ff7844', '#44ffa0', '#ff4488', '#aa88ff'
];

function userColor(name) {
  var h = 0;
  for (var i = 0; i < name.length; i++) h += name.charCodeAt(i);
  return USER_COLORS[h % USER_COLORS.length];
}

// ── IPC ────────────────────────────────────────
function send(o) { window.ipc.postMessage(JSON.stringify(o)); }

window.onRustMessage = function(ev, d) {
  switch(ev) {
    case 'config_loaded':
      myPeerId = d.peer_id;
      if (d.username) { myUsername = d.username; showChat(); }
      break;
    case 'peer_list':
      peers = d || [];
      renderPeers();
      break;
    case 'incoming_message':
      onMsg(d);
      break;
    case 'history':
      renderHistory(d || []);
      break;
    case 'group_list':
      groups = d || [];
      renderGroups();
      break;
    case 'group_created':
      closeModal();
      break;
    case 'message_ack':
      updAck(d.message_id, d.status);
      break;
    case 'error':
      console.error('[gustavio]', d);
      break;
  }
};

// ── Setup ──────────────────────────────────────
document.getElementById('setup-input').addEventListener('keydown', function(e) {
  if (e.key === 'Enter') submitName();
});
function submitName() {
  var n = document.getElementById('setup-input').value.trim();
  if (!n) return;
  send({ cmd: 'set_username', username: n });
}
function showChat() {
  document.getElementById('setup').style.display = 'none';
  document.getElementById('topbar').style.display = 'flex';
  document.getElementById('main').classList.add('visible');
  document.getElementById('my-info').textContent = myUsername + ' · online';
  send({ cmd: 'get_peers' });
  send({ cmd: 'get_groups' });
}

// ── Sidebar ────────────────────────────────────
function renderPeers() {
  var el = document.getElementById('peer-list');
  el.innerHTML = '';
  peers.forEach(function(p) {
    var d = document.createElement('div');
    d.className = 'sb-item' + (currentChat && currentChat.type==='dm' && currentChat.id===p.peer_id ? ' active' : '');
    var u = unread[p.peer_id] || 0;
    d.innerHTML = '<span class="sb-dot"></span><span class="sb-name">' + esc(p.username) + '</span>' +
      '<span class="sb-badge ' + (u > 0 ? 'vis' : '') + '">' + u + '</span>';
    d.onclick = function() { openDm(p.peer_id, p.username); };
    el.appendChild(d);
  });
}
function renderGroups() {
  var el = document.getElementById('group-list');
  el.innerHTML = '';
  groups.forEach(function(g) {
    var d = document.createElement('div');
    d.className = 'sb-item' + (currentChat && currentChat.type==='group' && currentChat.id===g.group_id ? ' active' : '');
    var u = unread[g.group_id] || 0;
    d.innerHTML = '<span class="sb-hash">#</span><span class="sb-name">' + esc(g.name) + '</span>' +
      '<span class="sb-badge ' + (u > 0 ? 'vis' : '') + '">' + u + '</span>';
    d.onclick = function() { openGroup(g.group_id, g.name); };
    el.appendChild(d);
  });
}

// ── Open Chat ──────────────────────────────────
function openDm(id, name) {
  currentChat = { type: 'dm', id: id, name: name };
  unread[id] = 0;
  activateChat(name, 'online');
  send({ cmd: 'load_history', conversation_id: id });
  send({ cmd: 'mark_read', conversation_id: id });
  renderPeers();
}
function openGroup(id, name) {
  currentChat = { type: 'group', id: id, name: name };
  unread[id] = 0;
  activateChat(name, 'grupo');
  send({ cmd: 'load_history', conversation_id: id });
  send({ cmd: 'mark_read', conversation_id: id });
  renderGroups();
}
function activateChat(name, status) {
  document.getElementById('empty-state').style.display = 'none';
  document.getElementById('chat-header').classList.add('vis');
  document.getElementById('messages').classList.add('vis');
  document.getElementById('messages').innerHTML = '';
  document.getElementById('input-area').classList.add('vis');
  document.getElementById('ch-name').textContent = name;
  document.getElementById('ch-status').textContent = status;
  document.getElementById('msg-input').focus();
}

// ── Messages ───────────────────────────────────
function onMsg(m) {
  var cid = m.conversation_id;
  if (currentChat && cid === currentChat.id) {
    appendMsg(m);
    scrollBottom();
  } else {
    unread[cid] = (unread[cid] || 0) + 1;
    renderPeers();
    renderGroups();
  }
}
function renderHistory(msgs) {
  document.getElementById('messages').innerHTML = '';
  msgs.forEach(function(m) { appendMsg(m); });
  scrollBottom();
}
function appendMsg(m) {
  var c = document.getElementById('messages');
  var mine = m.from_id === myPeerId;
  var div = document.createElement('div');
  div.className = 'msg-line';
  div.id = 'msg-' + m.id;

  var t = fmtTime(m.timestamp);
  var uStyle = mine ? 'me' : '';
  var uColor = mine ? '' : ' style="color:' + userColor(m.from_name) + '"';
  var ack = mine ? '<span class="msg-status">' + ackIcon(m.status) + '</span>' : '';

  div.innerHTML =
    '<span class="msg-time">' + t + '</span>' +
    '<span class="msg-sep">\u2502</span>' +
    '<span class="msg-user ' + uStyle + '"' + uColor + '>' + esc(m.from_name) + '</span>' +
    '<span class="msg-content">' + esc(m.content) + '</span>' +
    ack;

  c.appendChild(div);
}
function updAck(id, st) {
  var el = document.getElementById('msg-' + id);
  if (!el) return;
  var s = el.querySelector('.msg-status');
  if (s) s.textContent = ackIcon(st);
}
function ackIcon(s) {
  if (s === 'delivered') return '\u2713\u2713';
  if (s === 'sent') return '\u2713';
  return '';
}
function scrollBottom() {
  var c = document.getElementById('messages');
  setTimeout(function() { c.scrollTop = c.scrollHeight; }, 30);
}

// ── Send ───────────────────────────────────────
function sendMsg() {
  var inp = document.getElementById('msg-input');
  var txt = inp.value.trim();
  if (!txt || !currentChat) return;
  inp.value = '';
  if (currentChat.type === 'dm') {
    send({ cmd: 'send_message', peer_id: currentChat.id, content: txt });
  } else {
    send({ cmd: 'send_group_message', group_id: currentChat.id, content: txt });
  }
}
document.addEventListener('keydown', function(e) {
  if (e.key === 'Enter' && !e.shiftKey && document.activeElement.id === 'msg-input') {
    e.preventDefault();
    sendMsg();
  }
  // Ctrl+Shift+X = toggle censorship
  if (e.key === 'X' && e.ctrlKey && e.shiftKey) {
    e.preventDefault();
    toggleCensor();
  }
});

// ── Censorship ─────────────────────────────────
function toggleCensor() {
  document.body.classList.toggle('censored');
  var btn = document.getElementById('btn-censor');
  btn.classList.toggle('active', document.body.classList.contains('censored'));
}

// ── Pin (always on top) ────────────────────────
function togglePin() {
  pinned = !pinned;
  var btn = document.getElementById('btn-pin');
  btn.classList.toggle('pin-active', pinned);
  send({ cmd: 'set_always_on_top', enabled: pinned });
}

// ── Group Modal ────────────────────────────────
function openModal() {
  var mm = document.getElementById('modal-members');
  mm.innerHTML = '';
  peers.forEach(function(p) {
    var d = document.createElement('div');
    d.className = 'm-check';
    d.innerHTML = '<input type="checkbox" id="gm-' + p.peer_id + '" value="' + p.peer_id + '">' +
      '<label for="gm-' + p.peer_id + '">' + esc(p.username) + '</label>';
    mm.appendChild(d);
  });
  document.getElementById('grp-name').value = '';
  document.getElementById('modal-bg').classList.add('vis');
  document.getElementById('grp-name').focus();
}
function closeModal() { document.getElementById('modal-bg').classList.remove('vis'); }
function createGroup() {
  var name = document.getElementById('grp-name').value.trim();
  if (!name) return;
  var cks = document.querySelectorAll('#modal-members input:checked');
  var mem = [];
  cks.forEach(function(c) { mem.push(c.value); });
  if (!mem.length) return;
  send({ cmd: 'create_group', name: name, members: mem });
}

// ── Helpers ────────────────────────────────────
function esc(s) {
  var d = document.createElement('span');
  d.textContent = s;
  return d.innerHTML;
}
function fmtTime(ts) {
  try {
    var d = new Date(ts);
    var h = ('0'+d.getHours()).slice(-2);
    var m = ('0'+d.getMinutes()).slice(-2);
    return h + ':' + m;
  } catch(e) { return '--:--'; }
}
</script>
</body>
</html>
"##;
