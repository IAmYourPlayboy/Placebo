function Stub({ title, emoji }: { title: string; emoji: string }) {
  return (
    <div className="screen screen--center">
      <div className="placeholder-icon">{emoji}</div>
      <p className="placeholder-title">{title}</p>
      <p className="placeholder-sub">Скоро появится контент</p>
    </div>
  );
}
export default function FriendsScreen() { return <Stub title="Друзья" emoji="👥" />; }
