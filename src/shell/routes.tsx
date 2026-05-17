import { JSX } from "react";
import { Navigate, RouteObject } from "react-router-dom";
import { AuthGuard } from "../auth/AuthGuard";
import HomePlaceholder from "../screens/main/HomePlaceholder";
import NotificationsScreen from "../screens/skeletons/NotificationsScreen";
import HistoryScreen from "../screens/skeletons/HistoryScreen";
import FavoritesScreen from "../screens/skeletons/FavoritesScreen";
import FoldersScreen from "../screens/skeletons/FoldersScreen";
import PeopleScreen from "../screens/skeletons/PeopleScreen";
import SettingsScreen from "../screens/settings/SettingsScreen";
import ProfilePlaceholder from "../screens/profile/ProfilePlaceholder";
import WelcomeScreen from "../screens/auth/WelcomeScreen";
import RegisterScreen from "../screens/auth/RegisterScreen";
import LoginScreen from "../screens/auth/LoginScreen";

// Старые экраны-прототипы: продолжают работать до момента своего переписывания.
import ExploreScreen from "../screens/ExploreScreen";
import CreateScreen from "../screens/CreateScreen";
import WatchRoomScreen from "../screens/WatchRoomScreen";
import World3DScreen from "../screens/World3DScreen";

/**
 * Wrap a route element with AuthGuard. Anonymous visitors are redirected to /welcome;
 * the bootstrapping case shows a brief loading state to avoid flashing the welcome
 * page on every reload while /me is in flight.
 */
const guarded = (el: JSX.Element) => <AuthGuard>{el}</AuthGuard>;

export const routes: RouteObject[] = [
  // Public auth routes — must NOT be guarded, otherwise an anonymous redirect would loop.
  { path: "/welcome", element: <WelcomeScreen /> },
  { path: "/register", element: <RegisterScreen /> },
  { path: "/login", element: <LoginScreen /> },

  { path: "/", element: <Navigate to="/home" replace /> },
  { path: "/home", element: guarded(<HomePlaceholder />) },
  { path: "/categories", element: guarded(<ExploreScreen />) },
  { path: "/create", element: guarded(<CreateScreen />) },
  { path: "/people", element: guarded(<PeopleScreen />) },
  { path: "/notifications", element: guarded(<NotificationsScreen />) },
  { path: "/history", element: guarded(<HistoryScreen />) },
  { path: "/favorites", element: guarded(<FavoritesScreen />) },
  { path: "/folders", element: guarded(<FoldersScreen />) },
  { path: "/settings", element: guarded(<SettingsScreen />) },
  { path: "/profile", element: guarded(<ProfilePlaceholder />) },
  { path: "/profile/:username", element: guarded(<ProfilePlaceholder />) },
  // TODO(M5/M4): WatchRoomScreen и World3DScreen ждут onBack. window.history.back()
  // дёргает историю Tauri webview, а не per-tab MemoryRouter, поэтому в рамках shell
  // это no-op. Эти экраны переписываются в M4/M5 на useNavigate(-1) – тогда проп уйдёт.
  { path: "/room/:id", element: guarded(<WatchRoomScreen onBack={() => window.history.back()} />) },
  { path: "/world", element: guarded(<World3DScreen onBack={() => window.history.back()} />) },
  { path: "*", element: <Navigate to="/home" replace /> },
];
