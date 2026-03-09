import { useState, useRef, useEffect } from "react";

// ── Icons ──────────────────────────────────────────────────────────
function IcBack() {
  return (
    <svg width={22} height={22} viewBox="0 0 24 24" fill="none">
      <path d="M15 18l-6-6 6-6" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round"/>
    </svg>
  );
}
function IcBellW() {
  return (
    <svg width={20} height={20} viewBox="0 0 24 24" fill="none">
      <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9M13.73 21a2 2 0 01-3.46 0"
        stroke="currentColor" strokeWidth={1.8} strokeLinecap="round"/>
    </svg>
  );
}
function IcSearchW() {
  return (
    <svg width={20} height={20} viewBox="0 0 24 24" fill="none">
      <circle cx={11} cy={11} r={7} stroke="currentColor" strokeWidth={1.8}/>
      <path d="M16.5 16.5L21 21" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round"/>
    </svg>
  );
}
function IcSettings() {
  return (
    <svg width={20} height={20} viewBox="0 0 24 24" fill="none">
      <circle cx={12} cy={12} r={3} stroke="currentColor" strokeWidth={1.8}/>
      <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"
        stroke="currentColor" strokeWidth={1.8}/>
    </svg>
  );
}
function IcSkipBack() {
  return (
    <svg width={28} height={28} viewBox="0 0 24 24" fill="none">
      <polygon points="19 20 9 12 19 4 19 20" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round"/>
      <line x1={5} y1={19} x2={5} y2={5} stroke="currentColor" strokeWidth={1.8} strokeLinecap="round"/>
    </svg>
  );
}
function IcSkipFwd() {
  return (
    <svg width={28} height={28} viewBox="0 0 24 24" fill="none">
      <polygon points="5 4 15 12 5 20 5 4" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round"/>
      <line x1={19} y1={4} x2={19} y2={20} stroke="currentColor" strokeWidth={1.8} strokeLinecap="round"/>
    </svg>
  );
}
function IcPlay({ playing }: { playing: boolean }) {
  return playing ? (
    <svg width={36} height={36} viewBox="0 0 24 24" fill="none">
      <rect x={6} y={4} width={4} height={16} rx={1} fill="currentColor"/>
      <rect x={14} y={4} width={4} height={16} rx={1} fill="currentColor"/>
    </svg>
  ) : (
    <svg width={36} height={36} viewBox="0 0 24 24" fill="none">
      <polygon points="5 3 19 12 5 21 5 3" fill="currentColor"/>
    </svg>
  );
}
function IcFullscreen() {
  return (
    <svg width={16} height={16} viewBox="0 0 24 24" fill="none">
      <path d="M8 3H5a2 2 0 00-2 2v3M21 8V5a2 2 0 00-2-2h-3M3 16v3a2 2 0 002 2h3M16 21h3a2 2 0 002-2v-3"
        stroke="currentColor" strokeWidth={2} strokeLinecap="round"/>
    </svg>
  );
}
function IcSend() {
  return (
    <svg width={18} height={18} viewBox="0 0 24 24" fill="none">
      <path d="M22 2L11 13M22 2L15 22l-4-9-9-4 20-7z"
        stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round"/>
    </svg>
  );
}

// ── Types ──────────────────────────────────────────────────────────
interface Message { id: number; user: string; avatar: string; text: string; time: string; }
interface Participant { id: number; name: string; avatar: string; isHost?: boolean; }

// ── Mock data ──────────────────────────────────────────────────────
const INIT_MESSAGES: Message[] = [
  { id: 1, user: "sakura88",  avatar: "🌸", text: "Yooooo",        time: "12:48" },
  { id: 2, user: "kira_mv",   avatar: "🦊", text: "Наконец-то смотрим это аниме!", time: "12:49" },
  { id: 3, user: "sakura88",  avatar: "🌸", text: "Damn she's so pretty 😭",      time: "12:50" },
  { id: 4, user: "neo_drev",  avatar: "🐺", text: "Ahahaha",       time: "12:51" },
];

const PARTICIPANTS: Participant[] = [
  { id: 1, name: "sakura88",  avatar: "🌸", isHost: true },
  { id: 2, name: "kira_mv",   avatar: "🦊" },
  { id: 3, name: "neo_drev",  avatar: "🐺" },
  { id: 4, name: "you",       avatar: "😶" },
];

// ── Helpers ────────────────────────────────────────────────────────
function fmt(sec: number) {
  const m = Math.floor(sec / 60).toString().padStart(2, "0");
  const s = (sec % 60).toString().padStart(2, "0");
  return `${m}:${s}`;
}

// ── Component ──────────────────────────────────────────────────────
type Tab = "chat" | "participants" | "settings";

export default function WatchRoomScreen({ onBack }: { onBack?: () => void }) {
  const [playing, setPlaying] = useState(false);
  const [progress, setProgress] = useState(1455); // 24:15
  const DURATION = 6300; // 1:45:00

  const [tab, setTab] = useState<Tab>("chat");
  const [messages, setMessages] = useState<Message[]>(INIT_MESSAGES);
  const [draft, setDraft] = useState("");
  const chatRef = useRef<HTMLDivElement>(null);

  // Auto-scroll chat
  useEffect(() => {
    if (chatRef.current) chatRef.current.scrollTop = chatRef.current.scrollHeight;
  }, [messages, tab]);

  // Fake playback tick
  useEffect(() => {
    if (!playing) return;
    const id = setInterval(() => setProgress((p) => Math.min(p + 1, DURATION)), 1000);
    return () => clearInterval(id);
  }, [playing]);

  function sendMsg() {
    const text = draft.trim();
    if (!text) return;
    const now = new Date();
    setMessages((m) => [...m, {
      id: Date.now(),
      user: "you",
      avatar: "😶",
      text,
      time: `${now.getHours()}:${now.getMinutes().toString().padStart(2, "0")}`,
    }]);
    setDraft("");
  }

  const pct = (progress / DURATION) * 100;

  return (
    <div className="wr-root">
      {/* ── Top bar ── */}
      <header className="wr-topbar">
        <button className="wr-icon-btn" onClick={onBack} aria-label="Назад"><IcBack /></button>
        <div className="wr-topbar-actions">
          <button className="wr-icon-btn" aria-label="Уведомления"><IcBellW /></button>
          <button className="wr-icon-btn" aria-label="Поиск"><IcSearchW /></button>
          <button className="wr-icon-btn" aria-label="Настройки"><IcSettings /></button>
        </div>
      </header>

      {/* ── Player ── */}
      <div className="wr-player">
        <div className="wr-player-center">
          <div className="wr-controls">
            <button className="wr-ctrl-btn" onClick={() => setProgress((p) => Math.max(0, p - 10))}><IcSkipBack /></button>
            <button className="wr-play-btn" onClick={() => setPlaying((p) => !p)}><IcPlay playing={playing} /></button>
            <button className="wr-ctrl-btn" onClick={() => setProgress((p) => Math.min(DURATION, p + 10))}><IcSkipFwd /></button>
          </div>
        </div>

        {/* Progress */}
        <div className="wr-progress-wrap">
          <div className="wr-time">{fmt(progress)}</div>
          <div
            className="wr-seekbar"
            onClick={(e) => {
              const rect = e.currentTarget.getBoundingClientRect();
              const ratio = (e.clientX - rect.left) / rect.width;
              setProgress(Math.round(ratio * DURATION));
            }}
          >
            <div className="wr-seekbar-fill" style={{ width: `${pct}%` }} />
            <div className="wr-seekbar-thumb" style={{ left: `${pct}%` }} />
          </div>
          <div className="wr-time">{fmt(DURATION)}</div>
          <button className="wr-icon-btn" aria-label="Полный экран"><IcFullscreen /></button>
        </div>

        {/* Info */}
        <div className="wr-info">
          <p className="wr-title">Violet Evergarden Ep.1</p>
          <p className="wr-sub">Видео от автора «yuki_star» · {PARTICIPANTS.length} зрителя</p>
        </div>
      </div>

      {/* ── Tabs ── */}
      <div className="wr-tabs">
        {(["chat", "participants", "settings"] as Tab[]).map((t) => (
          <button
            key={t}
            className={`wr-tab${tab === t ? " wr-tab--active" : ""}`}
            onClick={() => setTab(t)}
          >
            {t === "chat" ? "Чат" : t === "participants" ? "Участники" : "Настройки комнаты"}
          </button>
        ))}
      </div>

      {/* ── Content ── */}
      <div className="wr-body">
        {tab === "chat" && (
          <>
            <div className="wr-chat" ref={chatRef}>
              {messages.map((m) => (
                <div key={m.id} className={`wr-msg${m.user === "you" ? " wr-msg--me" : ""}`}>
                  {m.user !== "you" && <span className="wr-msg-avatar">{m.avatar}</span>}
                  <div className="wr-msg-bubble">
                    {m.user !== "you" && <p className="wr-msg-name">{m.user}</p>}
                    <p className="wr-msg-text">{m.text}</p>
                    <p className="wr-msg-time">{m.time}</p>
                  </div>
                </div>
              ))}
            </div>
            <div className="wr-input-row">
              <input
                className="wr-input"
                placeholder="Сообщение..."
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && sendMsg()}
              />
              <button
                className={`wr-send-btn${draft.trim() ? " wr-send-btn--active" : ""}`}
                onClick={sendMsg}
                aria-label="Отправить"
              >
                <IcSend />
              </button>
            </div>
          </>
        )}

        {tab === "participants" && (
          <div className="wr-participants">
            {PARTICIPANTS.map((p) => (
              <div key={p.id} className="wr-participant">
                <span className="wr-participant-avatar">{p.avatar}</span>
                <div className="wr-participant-info">
                  <p className="wr-participant-name">{p.name}</p>
                  {p.isHost && <p className="wr-participant-role">Хост</p>}
                </div>
              </div>
            ))}
          </div>
        )}

        {tab === "settings" && (
          <div className="wr-settings">
            <div className="wr-setting-item">
              <span className="wr-setting-label">Название комнаты</span>
              <span className="wr-setting-val">Violet Evergarden марафон</span>
            </div>
            <div className="wr-setting-item">
              <span className="wr-setting-label">Доступ</span>
              <span className="wr-setting-val">Публичная</span>
            </div>
            <div className="wr-setting-item">
              <span className="wr-setting-label">Синхронизация</span>
              <span className="wr-setting-val wr-setting-on">Включена</span>
            </div>
            <button className="wr-leave-btn">Покинуть комнату</button>
          </div>
        )}
      </div>
    </div>
  );
}
