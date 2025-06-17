import { Suspense } from "react";
import { AppShell } from "@mantine/core";
import Navbar from "../../navbar";
import { routesConfig } from "../../../routers";
import { useRoutes } from "react-router-dom";
import { useAuth } from "@/store/auth/hooks";
import LockPage from "@/pages/lock";
import LoadingPage from "@/components/loading-card";

export const ViewPort = () => {
  const routes = useRoutes(routesConfig);
  const { hasAuth } = useAuth();
  if (!hasAuth) {
    return <LockPage />;
  }  
  return (
    <Suspense fallback={<LoadingPage />}>
      <AppShell
        withBorder={false}
        header={{ height: 0 }}
        navbar={{
          width: 170,
          breakpoint: 'sm', collapsed: { mobile: false }
        }}>

        <AppShell.Navbar visibleFrom="sm" style={{ backgroundColor: "transparent" }}>
          <Navbar />
        </AppShell.Navbar>
        <AppShell.Main style={{ "width": "100%" }}>
          {routes}
        </AppShell.Main>
      </AppShell>
    </Suspense >
  );
};
