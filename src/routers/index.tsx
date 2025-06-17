import { Navigate, Outlet, type RouteObject } from "react-router-dom";
import { lazy } from "react";
import AboutPage from "../pages/about";
import { IconDeviceImacUp, IconHistory, IconLayoutNavbarCollapse, IconSettings, IconTimelineEventText, IconTransfer, IconWallet } from "@tabler/icons-react";

const WalletPage = lazy(async () => await import("../pages/wallet")); 
const SettingsPage = lazy(async () => await import("../pages/settings")); 
const LogPage = lazy(async () => await import("../pages/log"));
const HistoryPage = lazy(async () => await import("../pages/history"));
const BatchPage = lazy(async () => await import("../pages/batch"));
export const routesConfig: RouteObject[] = [
    {
        path: "/",
        element: (
            <div className="main">
                <div className="main-content">
                    <Outlet />
                </div>
            </div>
        ),
        children: [
            {
                index: true,
                element: <Navigate to="/wallet" />,
            },
            {
                path: "log",
                element: <LogPage />,
            }, {
                path: "wallet",
                element: <WalletPage />,
            }, {
                path: "send",
                element: <BatchPage />,
            },
            {
                path: "history",
                element: <HistoryPage />,
            },
            {
                path: "settings",
                element: <SettingsPage />,
            }, {
                path: "about",
                element: <AboutPage />,
            },
        ],
    },
];


export const linkdata = [

    { label: 'Wallet', href: "/wallet", icon: IconWallet },
    { label: 'Send', href: "/send", icon: IconTransfer },
    { label: 'History', href: "/history", icon: IconHistory },
    {
        label: 'Advanced', href: "/advanced", icon: IconDeviceImacUp,
        links: [
            {
                icon: IconTimelineEventText,
                label: "Log",
                link: "/log",
            },
        ]
    },
    { label: 'Settings', href: "/settings", icon: IconSettings, },
    { label: 'About', href: "/about", icon: IconLayoutNavbarCollapse, },
];