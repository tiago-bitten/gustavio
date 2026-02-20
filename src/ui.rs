pub const HTML: &str = r##"<!DOCTYPE html>
<html lang="pt-BR">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Gustavio Chat</title>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  :root {
    --bg: #0f0f0f;
    --surface: #1a1a1a;
    --surface2: #242424;
    --border: #2a2a2a;
    --text: #e0e0e0;
    --text-dim: #888;
    --accent: #00ff88;
    --accent2: #00e0ff;
    --danger: #ff6b6b;
    --sent-bg: #1a2e1a;
    --recv-bg: #1e1e2e;
  }
  body {
    font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
    background: var(--bg);
    color: var(--text);
    height: 100vh;
    overflow: hidden;
    display: flex;
    user-select: none;
  }

  /* ── Setup Screen ─────────────────────────────── */
  #setup-screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    gap: 24px;
  }
  #setup-screen h1 {
    font-size: 32px;
    font-weight: 700;
    color: var(--accent);
    letter-spacing: 4px;
  }
  #setup-screen p {
    color: var(--text-dim);
    font-size: 14px;
  }
  #setup-screen input {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px 20px;
    font-size: 16px;
    color: var(--text);
    outline: none;
    width: 300px;
    text-align: center;
    transition: border-color 0.2s;
  }
  #setup-screen input:focus { border-color: var(--accent); }
  #setup-screen button {
    background: var(--accent);
    color: #000;
    border: none;
    border-radius: 8px;
    padding: 12px 32px;
    font-size: 15px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.2s;
  }
  #setup-screen button:hover { opacity: 0.85; }

  /* ── Chat Layout ──────────────────────────────── */
  #chat-screen { display: none; width: 100%; height: 100%; }

  /* Sidebar */
  #sidebar {
    width: 280px;
    min-width: 280px;
    background: var(--surface);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  #sidebar-header {
    padding: 20px 16px 12px;
    border-bottom: 1px solid var(--border);
  }
  #sidebar-header h2 {
    font-size: 18px;
    font-weight: 700;
    color: var(--accent);
    letter-spacing: 2px;
  }
  #sidebar-header .my-name {
    font-size: 12px;
    color: var(--text-dim);
    margin-top: 4px;
  }
  #sidebar-sections { flex: 1; overflow-y: auto; }
  .section-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 1px;
    padding: 16px 16px 8px;
  }
  .peer-item, .group-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    cursor: pointer;
    transition: background 0.15s;
    position: relative;
  }
  .peer-item:hover, .group-item:hover { background: var(--surface2); }
  .peer-item.active, .group-item.active { background: var(--surface2); border-left: 3px solid var(--accent); }
  .online-dot {
    width: 8px; height: 8px;
    border-radius: 50%;
    background: var(--accent);
    flex-shrink: 0;
  }
  .peer-name, .group-name {
    font-size: 14px;
    font-weight: 500;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .badge {
    background: var(--accent);
    color: #000;
    font-size: 11px;
    font-weight: 700;
    border-radius: 10px;
    padding: 1px 7px;
    min-width: 18px;
    text-align: center;
    display: none;
  }
  .badge.visible { display: inline-block; }
  .group-icon {
    width: 20px; height: 20px;
    display: flex; align-items: center; justify-content: center;
    font-size: 14px;
    flex-shrink: 0;
  }

  /* New group button */
  #new-group-btn {
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    font-size: 13px;
    color: var(--accent2);
    cursor: pointer;
    text-align: center;
    font-weight: 500;
    transition: background 0.15s;
  }
  #new-group-btn:hover { background: var(--surface2); }

  /* Chat Area */
  #chat-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  #chat-header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--surface);
  }
  #chat-header-name {
    font-size: 16px;
    font-weight: 600;
  }
  #chat-header-status {
    font-size: 12px;
    color: var(--text-dim);
  }
  #no-chat-selected {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-dim);
    font-size: 15px;
  }
  #messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  #messages-container::-webkit-scrollbar { width: 6px; }
  #messages-container::-webkit-scrollbar-track { background: transparent; }
  #messages-container::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }

  .message-bubble {
    max-width: 65%;
    padding: 8px 14px;
    border-radius: 12px;
    font-size: 14px;
    line-height: 1.5;
    word-wrap: break-word;
    position: relative;
  }
  .message-bubble.sent {
    align-self: flex-end;
    background: var(--sent-bg);
    border: 1px solid #2a4a2a;
    border-bottom-right-radius: 4px;
  }
  .message-bubble.received {
    align-self: flex-start;
    background: var(--recv-bg);
    border: 1px solid #2a2a4a;
    border-bottom-left-radius: 4px;
  }
  .message-sender {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent2);
    margin-bottom: 2px;
  }
  .message-text {
    color: var(--text);
  }
  .message-meta {
    font-size: 11px;
    color: var(--text-dim);
    text-align: right;
    margin-top: 4px;
  }
  .message-status { font-size: 10px; margin-left: 4px; }

  /* Input area */
  #input-area {
    padding: 12px 20px;
    border-top: 1px solid var(--border);
    display: flex;
    gap: 10px;
    background: var(--surface);
  }
  #msg-input {
    flex: 1;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 14px;
    font-size: 14px;
    color: var(--text);
    outline: none;
    transition: border-color 0.2s;
    font-family: inherit;
  }
  #msg-input:focus { border-color: var(--accent); }
  #send-btn {
    background: var(--accent);
    color: #000;
    border: none;
    border-radius: 8px;
    padding: 10px 20px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.2s;
    white-space: nowrap;
  }
  #send-btn:hover { opacity: 0.85; }

  /* ── Group Modal ──────────────────────────────── */
  #modal-overlay {
    display: none;
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.6);
    z-index: 100;
    align-items: center;
    justify-content: center;
  }
  #modal-overlay.visible { display: flex; }
  #modal {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 24px;
    width: 380px;
    max-height: 80vh;
    overflow-y: auto;
  }
  #modal h3 {
    font-size: 18px;
    font-weight: 600;
    margin-bottom: 16px;
    color: var(--accent2);
  }
  #modal input[type="text"] {
    width: 100%;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px 12px;
    font-size: 14px;
    color: var(--text);
    outline: none;
    margin-bottom: 16px;
  }
  #modal input[type="text"]:focus { border-color: var(--accent); }
  .modal-label {
    font-size: 12px;
    color: var(--text-dim);
    margin-bottom: 8px;
    text-transform: uppercase;
    letter-spacing: 1px;
  }
  .member-check {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 0;
  }
  .member-check input[type="checkbox"] { accent-color: var(--accent); }
  .member-check label { font-size: 14px; cursor: pointer; }
  .modal-actions {
    display: flex;
    gap: 10px;
    margin-top: 20px;
    justify-content: flex-end;
  }
  .modal-actions button {
    border: none;
    border-radius: 6px;
    padding: 8px 20px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.2s;
  }
  .modal-actions button:hover { opacity: 0.85; }
  .btn-cancel { background: var(--surface2); color: var(--text-dim); }
  .btn-create { background: var(--accent); color: #000; }

  /* ── System Message ───────────────────────────── */
  .system-msg {
    text-align: center;
    font-size: 12px;
    color: var(--text-dim);
    padding: 4px 0;
  }
</style>
</head>
<body>

<!-- Setup Screen -->
<div id="setup-screen">
  <h1>GUSTAVIO</h1>
  <p>Chat local pela rede — sem servidor, sem internet</p>
  <input type="text" id="username-input" placeholder="Seu nome..." maxlength="24" autofocus>
  <button id="setup-btn" onclick="submitUsername()">Entrar</button>
</div>

<!-- Chat Screen -->
<div id="chat-screen">
  <div id="sidebar">
    <div id="sidebar-header">
      <h2>GUSTAVIO</h2>
      <div class="my-name" id="my-name-label"></div>
    </div>
    <div id="sidebar-sections">
      <div class="section-title">Online</div>
      <div id="peer-list"></div>
      <div class="section-title">Grupos</div>
      <div id="group-list"></div>
    </div>
    <div id="new-group-btn" onclick="openGroupModal()">+ Novo Grupo</div>
  </div>
  <div id="chat-area">
    <div id="no-chat-selected">Selecione uma conversa</div>
    <div id="chat-header" style="display:none">
      <div>
        <div id="chat-header-name"></div>
        <div id="chat-header-status"></div>
      </div>
    </div>
    <div id="messages-container" style="display:none"></div>
    <div id="input-area" style="display:none">
      <input type="text" id="msg-input" placeholder="Digite uma mensagem..." autocomplete="off">
      <button id="send-btn" onclick="sendMessage()">Enviar</button>
    </div>
  </div>
</div>

<!-- Group Modal -->
<div id="modal-overlay">
  <div id="modal">
    <h3>Criar Grupo</h3>
    <div class="modal-label">Nome do grupo</div>
    <input type="text" id="group-name-input" placeholder="Nome do grupo..." maxlength="32">
    <div class="modal-label">Membros</div>
    <div id="modal-members"></div>
    <div class="modal-actions">
      <button class="btn-cancel" onclick="closeGroupModal()">Cancelar</button>
      <button class="btn-create" onclick="createGroup()">Criar</button>
    </div>
  </div>
</div>

<script>
// ── State ──────────────────────────────────────────
let myPeerId = null;
let myUsername = null;
let peers = [];
let groups = [];
let currentChat = null; // { type: 'dm'|'group', id: string }
let unreadCounts = {};  // conversationId -> count

// ── IPC ────────────────────────────────────────────
function send(obj) {
  window.ipc.postMessage(JSON.stringify(obj));
}

// Called by Rust via evaluate_script
window.onRustMessage = function(event, data) {
  switch(event) {
    case 'config_loaded':
      myPeerId = data.peer_id;
      if (data.username) {
        myUsername = data.username;
        showChatScreen();
      }
      break;
    case 'peer_list':
      peers = data || [];
      renderPeerList();
      break;
    case 'incoming_message':
      handleIncomingMessage(data);
      break;
    case 'history':
      renderHistory(data || []);
      break;
    case 'group_list':
      groups = data || [];
      renderGroupList();
      break;
    case 'group_created':
      closeGroupModal();
      break;
    case 'message_ack':
      updateMessageStatus(data.message_id, data.status);
      break;
    case 'error':
      console.error('Rust error:', data);
      break;
  }
};

// ── Setup ──────────────────────────────────────────
document.getElementById('username-input').addEventListener('keydown', function(e) {
  if (e.key === 'Enter') submitUsername();
});

function submitUsername() {
  const name = document.getElementById('username-input').value.trim();
  if (!name) return;
  send({ cmd: 'set_username', username: name });
}

function showChatScreen() {
  document.getElementById('setup-screen').style.display = 'none';
  document.getElementById('chat-screen').style.display = 'flex';
  document.getElementById('my-name-label').textContent = myUsername;
  send({ cmd: 'get_peers' });
  send({ cmd: 'get_groups' });
}

// ── Peer List ──────────────────────────────────────
function renderPeerList() {
  const el = document.getElementById('peer-list');
  el.innerHTML = '';
  peers.forEach(function(p) {
    const div = document.createElement('div');
    div.className = 'peer-item' + (currentChat && currentChat.type === 'dm' && currentChat.id === p.peer_id ? ' active' : '');
    const convId = p.peer_id;
    const unread = unreadCounts[convId] || 0;
    div.innerHTML =
      '<span class="online-dot"></span>' +
      '<span class="peer-name">' + escapeHtml(p.username) + '</span>' +
      '<span class="badge ' + (unread > 0 ? 'visible' : '') + '" id="badge-' + convId + '">' + unread + '</span>';
    div.onclick = function() { openDm(p.peer_id, p.username); };
    el.appendChild(div);
  });
}

function renderGroupList() {
  const el = document.getElementById('group-list');
  el.innerHTML = '';
  groups.forEach(function(g) {
    const div = document.createElement('div');
    div.className = 'group-item' + (currentChat && currentChat.type === 'group' && currentChat.id === g.group_id ? ' active' : '');
    const unread = unreadCounts[g.group_id] || 0;
    div.innerHTML =
      '<span class="group-icon">#</span>' +
      '<span class="group-name">' + escapeHtml(g.name) + '</span>' +
      '<span class="badge ' + (unread > 0 ? 'visible' : '') + '" id="badge-' + g.group_id + '">' + unread + '</span>';
    div.onclick = function() { openGroup(g.group_id, g.name); };
    el.appendChild(div);
  });
}

// ── Open Chat ──────────────────────────────────────
function openDm(peerId, peerName) {
  currentChat = { type: 'dm', id: peerId, name: peerName };
  unreadCounts[peerId] = 0;
  showChatUI(peerName, 'online');
  send({ cmd: 'load_history', conversation_id: peerId });
  send({ cmd: 'mark_read', conversation_id: peerId });
  renderPeerList();
}

function openGroup(groupId, groupName) {
  currentChat = { type: 'group', id: groupId, name: groupName };
  unreadCounts[groupId] = 0;
  showChatUI(groupName, 'grupo');
  send({ cmd: 'load_history', conversation_id: groupId });
  send({ cmd: 'mark_read', conversation_id: groupId });
  renderGroupList();
}

function showChatUI(name, status) {
  document.getElementById('no-chat-selected').style.display = 'none';
  document.getElementById('chat-header').style.display = 'flex';
  document.getElementById('messages-container').style.display = 'flex';
  document.getElementById('input-area').style.display = 'flex';
  document.getElementById('chat-header-name').textContent = name;
  document.getElementById('chat-header-status').textContent = status;
  document.getElementById('messages-container').innerHTML = '';
  document.getElementById('msg-input').focus();
}

// ── Messages ───────────────────────────────────────
function handleIncomingMessage(msg) {
  const convId = msg.conversation_id;
  // If this message belongs to the current chat, render it
  if (currentChat && convId === currentChat.id) {
    appendMessage(msg);
    scrollToBottom();
  } else {
    // Increment unread
    unreadCounts[convId] = (unreadCounts[convId] || 0) + 1;
    renderPeerList();
    renderGroupList();
  }
}

function renderHistory(messages) {
  const container = document.getElementById('messages-container');
  container.innerHTML = '';
  messages.forEach(function(msg) {
    appendMessage(msg);
  });
  scrollToBottom();
}

function appendMessage(msg) {
  const container = document.getElementById('messages-container');
  const isMine = msg.from_id === myPeerId;
  const div = document.createElement('div');
  div.className = 'message-bubble ' + (isMine ? 'sent' : 'received');
  div.id = 'msg-' + msg.id;

  let senderHtml = '';
  if (!isMine && msg.is_group) {
    senderHtml = '<div class="message-sender">' + escapeHtml(msg.from_name) + '</div>';
  }

  const time = formatTime(msg.timestamp);
  const statusIcon = isMine ? statusToIcon(msg.status) : '';

  div.innerHTML = senderHtml +
    '<div class="message-text">' + escapeHtml(msg.content) + '</div>' +
    '<div class="message-meta">' + time + '<span class="message-status">' + statusIcon + '</span></div>';

  container.appendChild(div);
}

function updateMessageStatus(messageId, status) {
  const el = document.getElementById('msg-' + messageId);
  if (el) {
    const statusEl = el.querySelector('.message-status');
    if (statusEl) statusEl.textContent = statusToIcon(status);
  }
}

function statusToIcon(status) {
  if (status === 'delivered') return '\u2713\u2713';
  if (status === 'sent') return '\u2713';
  return '';
}

function scrollToBottom() {
  const c = document.getElementById('messages-container');
  setTimeout(function() { c.scrollTop = c.scrollHeight; }, 50);
}

// ── Send ───────────────────────────────────────────
function sendMessage() {
  const input = document.getElementById('msg-input');
  const text = input.value.trim();
  if (!text || !currentChat) return;
  input.value = '';

  if (currentChat.type === 'dm') {
    send({ cmd: 'send_message', peer_id: currentChat.id, content: text });
  } else {
    send({ cmd: 'send_group_message', group_id: currentChat.id, content: text });
  }
}

document.addEventListener('keydown', function(e) {
  if (e.key === 'Enter' && !e.shiftKey && document.activeElement.id === 'msg-input') {
    e.preventDefault();
    sendMessage();
  }
});

// ── Group Modal ────────────────────────────────────
function openGroupModal() {
  const membersDiv = document.getElementById('modal-members');
  membersDiv.innerHTML = '';
  peers.forEach(function(p) {
    const label = document.createElement('div');
    label.className = 'member-check';
    label.innerHTML =
      '<input type="checkbox" id="gm-' + p.peer_id + '" value="' + p.peer_id + '">' +
      '<label for="gm-' + p.peer_id + '">' + escapeHtml(p.username) + '</label>';
    membersDiv.appendChild(label);
  });
  document.getElementById('group-name-input').value = '';
  document.getElementById('modal-overlay').classList.add('visible');
  document.getElementById('group-name-input').focus();
}

function closeGroupModal() {
  document.getElementById('modal-overlay').classList.remove('visible');
}

function createGroup() {
  const name = document.getElementById('group-name-input').value.trim();
  if (!name) return;
  const checks = document.querySelectorAll('#modal-members input[type=checkbox]:checked');
  const members = [];
  checks.forEach(function(c) { members.push(c.value); });
  if (members.length === 0) return;
  send({ cmd: 'create_group', name: name, members: members });
}

// ── Helpers ────────────────────────────────────────
function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function formatTime(ts) {
  try {
    const d = new Date(ts);
    return d.toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' });
  } catch(e) {
    return '';
  }
}
</script>
</body>
</html>
"##;
