import { useState } from "react";
import BottomNav from "./components/BottomNav";
import HomeScreen from "./screens/HomeScreen";
import ExploreScreen from "./screens/ExploreScreen";
import CreateScreen from "./screens/CreateScreen";
import FriendsScreen from "./screens/FriendsScreen";
import ProfileScreen from "./screens/ProfileScreen";
import WatchRoomScreen from "./screens/WatchRoomScreen";
import World3DScreen from "./screens/World3DScreen";

export type Screen = "home" | "explore" | "create" | "friends" | "profile";

export default function App() {
  const [screen, setScreen] = useState<Screen>("home");
  const [inRoom, setInRoom] = useState(false);
  const [in3DWorld, setIn3DWorld] = useState(false);

  // 3D World – полноэкранный режим поверх всего
  if (in3DWorld) {
    return <World3DScreen onBack={() => setIn3DWorld(false)} />;
  }

  if (inRoom) {
    return (
      <div className="app-shell">
        <WatchRoomScreen onBack={() => setInRoom(false)} />
      </div>
    );
  }

  const screens: Record<Screen, JSX.Element> = {
    home:    <HomeScreen onEnterRoom={() => setInRoom(true)} onEnter3D={() => setIn3DWorld(true)} />,
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
