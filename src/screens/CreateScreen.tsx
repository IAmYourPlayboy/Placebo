interface CreateOption {
  id: string;
  label: string;
  emoji: string;
  desc: string;
}

const OPTIONS: CreateOption[] = [
  { id: "film",   label: "Комнату для просмотра фильма",       emoji: "🎬", desc: "YouTube, VK Видео или прямая ссылка" },
  { id: "call",   label: "Комнату для созвона",                 emoji: "📞", desc: "Голос + видеосозвон с друзьями"      },
  { id: "stream", label: "Онлайн-трансляцию",                   emoji: "📡", desc: "Стримь прямо из приложения"          },
  { id: "games",  label: "Комнату для совместных игр с друзьями", emoji: "🎮", desc: "Играй вместе онлайн"               },
];

export default function CreateScreen() {
  return (
    <div className="screen">
      <header className="topbar">
        <h1 className="topbar__title">Создать</h1>
      </header>

      <div className="screen-body">
        <div className="section" style={{ paddingTop: 24 }}>
          <p className="create-subtitle">Что вы хотите создать?</p>
          <div className="create-grid">
            {OPTIONS.map((opt) => (
              <button key={opt.id} className="create-card">
                <span className="create-card__emoji">{opt.emoji}</span>
                <span className="create-card__label">{opt.label}</span>
                <span className="create-card__desc">{opt.desc}</span>
              </button>
            ))}
          </div>
        </div>

        <div className="spacer" />
      </div>
    </div>
  );
}
