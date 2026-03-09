import { useState } from "react";
import BottomNav from "./components/BottomNav";
import HomeScreen from "./screens/HomeScreen";
import ExploreScreen from "./screens/ExploreScreen";
import CreateScreen from "./screens/CreateScreen";
import FriendsScreen from "./screens/FriendsScreen";
import ProfileScreen from "./screens/ProfileScreen";
import WatchRoomScreen from "./screens/WatchRoomScreen";

export type Screen = "home" | "explore" | "create" | "friends" | "profile";

export default function App() {
  const [screen, setScreen] = useState<Screen>("home");
  const [inRoom, setInRoom] = useState(false);

  if (inRoom) {
    return (
      <div className="app-shell">
        <WatchRoomScreen onBack={() => setInRoom(false)} />
      </div>
    );
  }

  const screens: Record<Screen, JSX.Element> = {
    home:    <HomeScreen onEnterRoom={() => setInRoom(true)} />,
    explore: <ExploreScreen />,
    create:  <CreateScreen />,
    friends: <FriendsScreen />,
    profile: <ProfileScreen />,
  };

  return (
    <div className="app-shell">
      <div className="app-content" key={screen}>
        {screens[screen]}
      </div>
      <BottomNav active={screen} onChange={setScreen} />
    </div>
  );
}
