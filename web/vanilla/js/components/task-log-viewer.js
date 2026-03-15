/**
 * Semaphore Vanilla JS - Task Log Viewer
 * Real-time task log streaming via WebSocket with ANSI color support
 */

/**
 * Minimal ANSI escape code parser → HTML spans
 * Supports standard 8-color foreground/background and bold/dim/reset
 * @param {string} text - Raw log line (may contain ANSI codes)
 * @returns {string} - HTML string with <span> color wrappers
 */
function ansiToHtml(text) {
  // Escape HTML first to prevent XSS
  text = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');

  // Map of SGR codes → CSS classes
  const COLORS = {
    30: 'ansi-black',   31: 'ansi-red',     32: 'ansi-green',  33: 'ansi-yellow',
    34: 'ansi-blue',    35: 'ansi-magenta',  36: 'ansi-cyan',   37: 'ansi-white',
    90: 'ansi-bright-black',  91: 'ansi-bright-red',    92: 'ansi-bright-green',
    93: 'ansi-bright-yellow', 94: 'ansi-bright-blue',   95: 'ansi-bright-magenta',
    96: 'ansi-bright-cyan',   97: 'ansi-bright-white',
  };

  let openSpans = 0;
  const result = text.replace(/\x1b\[([0-9;]*)m/g, (_, params) => {
    const codes = params ? params.split(';').map(Number) : [0];
    let html = '';

    for (const code of codes) {
      if (code === 0) {
        // Reset — close all open spans
        html += '</span>'.repeat(openSpans);
        openSpans = 0;
      } else if (code === 1) {
        html += '<span class="ansi-bold">';
        openSpans++;
      } else if (code === 2) {
        html += '<span class="ansi-dim">';
        openSpans++;
      } else if (COLORS[code]) {
        html += `<span class="${COLORS[code]}">`;
        openSpans++;
      }
    }
    return html;
  });

  // Close any unclosed spans
  return result + '</span>'.repeat(openSpans);
}

export class TaskLogViewer {
  /**
   * @param {HTMLElement} container - Element to render log into
   * @param {Object} options
   * @param {number|string} options.projectId
   * @param {number|string} options.taskId
   * @param {Function} [options.onStatusChange] - Called with (status: string)
   * @param {Function} [options.onDone] - Called when task finishes
   */
  constructor(container, options = {}) {
    this.container = container;
    this.projectId = options.projectId;
    this.taskId = options.taskId;
    this.onStatusChange = options.onStatusChange || (() => {});
    this.onDone = options.onDone || (() => {});

    this._ws = null;
    this._autoScroll = true;
    this._lineCount = 0;

    this._render();
    this._connectWebSocket();
  }

  // ── Private ──────────────────────────────────────────────────────────────

  _render() {
    this.container.innerHTML = `
      <div class="task-log-viewer">
        <div class="task-log-toolbar">
          <span class="task-log-status" id="tlv-status">
            <i class="mdi mdi-circle-outline"></i> Подключение…
          </span>
          <label class="task-log-autoscroll">
            <input type="checkbox" id="tlv-autoscroll" checked>
            Автопрокрутка
          </label>
          <button class="v-btn v-btn--text" id="tlv-download" title="Скачать лог">
            <i class="mdi mdi-download"></i>
          </button>
        </div>
        <pre class="task-log-output" id="tlv-output"></pre>
      </div>
    `;

    this._output = this.container.querySelector('#tlv-output');
    this._statusEl = this.container.querySelector('#tlv-status');

    // Auto-scroll toggle
    const checkbox = this.container.querySelector('#tlv-autoscroll');
    checkbox.addEventListener('change', (e) => {
      this._autoScroll = e.target.checked;
    });

    // Pause auto-scroll when user scrolls up manually
    this._output.addEventListener('scroll', () => {
      const el = this._output;
      const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
      if (!atBottom) {
        this._autoScroll = false;
        const cb = this.container.querySelector('#tlv-autoscroll');
        if (cb) cb.checked = false;
      }
    });

    // Download log
    this.container.querySelector('#tlv-download').addEventListener('click', () => {
      this._downloadLog();
    });
  }

  _connectWebSocket() {
    const token = localStorage.getItem('semaphore_token');
    const proto = window.location.protocol === 'https:' ? 'wss' : 'ws';
    const host = window.location.host;
    const url = `${proto}://${host}/api/project/${this.projectId}/tasks/${this.taskId}/ws${token ? `?token=${encodeURIComponent(token)}` : ''}`;

    this._ws = new WebSocket(url);

    this._ws.addEventListener('open', () => {
      this._setStatus('running', 'Подключено');
    });

    this._ws.addEventListener('message', (event) => {
      try {
        const msg = JSON.parse(event.data);
        this._handleMessage(msg);
      } catch {
        // Raw text line fallback
        this._appendLine(event.data);
      }
    });

    this._ws.addEventListener('close', (event) => {
      if (event.code !== 1000) {
        this._setStatus('error', `Соединение закрыто (${event.code})`);
      }
    });

    this._ws.addEventListener('error', () => {
      this._setStatus('error', 'Ошибка WebSocket');
    });
  }

  _handleMessage(msg) {
    switch (msg.type) {
      case 'output':
        this._appendLine(msg.line ?? msg.output ?? '');
        break;
      case 'status':
        this._setStatus(msg.status, this._statusLabel(msg.status));
        this.onStatusChange(msg.status);
        break;
      case 'done':
        this._setStatus(msg.status || 'success', this._statusLabel(msg.status || 'success'));
        this.onStatusChange(msg.status || 'success');
        this.onDone(msg.status || 'success');
        this.disconnect();
        break;
      default:
        // Unknown message — ignore
        break;
    }
  }

  _appendLine(text) {
    if (!this._output) return;
    this._lineCount++;

    const line = document.createElement('div');
    line.className = 'task-log-line';
    line.innerHTML = ansiToHtml(text);
    this._output.appendChild(line);

    if (this._autoScroll) {
      this._output.scrollTop = this._output.scrollHeight;
    }
  }

  _setStatus(status, label) {
    if (!this._statusEl) return;
    const icons = {
      running:  'mdi-circle-slice-8 ansi-yellow',
      success:  'mdi-check-circle ansi-green',
      error:    'mdi-alert-circle ansi-red',
      stopped:  'mdi-stop-circle ansi-bright-black',
      waiting:  'mdi-clock-outline ansi-blue',
    };
    const iconClass = icons[status] || 'mdi-circle-outline';
    this._statusEl.innerHTML = `<i class="mdi ${iconClass}"></i> ${label}`;
    this._statusEl.dataset.status = status;
  }

  _statusLabel(status) {
    const labels = {
      running: 'Выполняется',
      success: 'Успешно',
      error:   'Ошибка',
      stopped: 'Остановлено',
      waiting: 'Ожидание',
    };
    return labels[status] || status;
  }

  _downloadLog() {
    const lines = Array.from(this._output.querySelectorAll('.task-log-line'))
      .map(el => el.textContent)
      .join('\n');
    const blob = new Blob([lines], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `task-${this.taskId}-log.txt`;
    a.click();
    URL.revokeObjectURL(url);
  }

  // ── Public API ────────────────────────────────────────────────────────────

  /**
   * Append pre-fetched log lines (from REST API) before WS connects
   * @param {Array<{output: string}>} lines
   */
  appendHistoricLog(lines) {
    if (!Array.isArray(lines)) return;
    for (const entry of lines) {
      this._appendLine(entry.output ?? entry.line ?? '');
    }
  }

  /**
   * Close the WebSocket connection
   */
  disconnect() {
    if (this._ws && this._ws.readyState < WebSocket.CLOSING) {
      this._ws.close(1000, 'done');
    }
    this._ws = null;
  }

  /**
   * Destroy the component and free resources
   */
  destroy() {
    this.disconnect();
    this.container.innerHTML = '';
  }
}

export default TaskLogViewer;
