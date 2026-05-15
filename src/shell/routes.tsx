import { RouteObject, Navigate } from "react-router-dom";
import HomePlaceholder from "../screens/main/HomePlaceholder";
import NotificationsScreen from "../screens/skeletons/NotificationsScreen";
import HistoryScreen from "../screens/skeletons/HistoryScreen";
import FavoritesScreen from "../screens/skeletons/FavoritesScreen";
import FoldersScreen from "../screens/skeletons/FoldersScreen";
import PeopleScreen from "../screens/skeletons/PeopleScreen";
import SettingsScreen from "../screens/settings/SettingsScreen";
import ProfilePlaceholder from "../screens/profile/ProfilePlaceholder";

// Старые экраны-прототипы: продолжают работать до момента своего переписывания.
import ExploreScreen from "../screens/ExploreScreen";
import CreateScreen from "../screens/CreateScreen";
import WatchRoomScreen from "../screens/WatchRoomScreen";
import World3DScreen from "../screens/World3DScreen";

export const routes: RouteObject[] = [
  { path: "/", element: <Navigate to="/home" replace /> },
  { path: "/home", element: <HomePlaceholder /> },
  { path: "/categories", element: <ExploreScreen /> },
  { path: "/create", element: <CreateScreen /> },
  { path: "/people", element: <PeopleScreen /> },
  { path: "/notifications", element: <NotificationsScreen /> },
  { path: "/history", element: <HistoryScreen /> },
  { path: "/favorites", element: <FavoritesScreen /> },
  { path: "/folders", element: <FoldersScreen /> },
  { path: "/settings", element: <SettingsScreen /> },
  { path: "/profile", element: <ProfilePlaceholder /> },
  { path: "/profile/:username", element: <ProfilePlaceholder /> },
  { path: "/room/:id", element: <WatchRoomScreen onBack={() => window.history.back()} /> },
  { path: "/world", element: <World3DScreen onBack={() => window.history.back()} /> },
  { path: "*", element: <Navigate to="/home" replace /> },
];
