import Sidebar from "./Sidebar";
import TopBar from "./TopBar";
import TabBar from "./TabBar";
import Breadcrumbs from "./Breadcrumbs";
import TabContent from "./TabContent";
import ShellLayout from "./ShellLayout";
import GlobalCanvas from "./scene3d/GlobalCanvas";

export default function ShellRoot() {
  return (
    <>
      <ShellLayout
        sidebar={<Sidebar />}
        topbar={<TopBar />}
        tabbar={<TabBar />}
        breadcrumbs={<Breadcrumbs />}
        content={<TabContent />}
      />
      <GlobalCanvas />
    </>
  );
}
