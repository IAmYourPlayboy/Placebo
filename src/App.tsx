import { AuthProvider } from "./auth/AuthProvider";
import { ThemeProvider } from "./theme";
import { TabManager } from "./shell/tabs/TabManager";
import { Scene3DRegistry } from "./shell/scene3d/Scene3DRegistry";
import { ToastProvider } from "./components/ui/Toast";
import ShellRoot from "./shell/ShellRoot";

export default function App() {
  return (
    <ThemeProvider>
      <ToastProvider>
        <AuthProvider>
          <Scene3DRegistry>
            <TabManager initialPath="/home">
              <ShellRoot />
            </TabManager>
          </Scene3DRegistry>
        </AuthProvider>
      </ToastProvider>
    </ThemeProvider>
  );
}
