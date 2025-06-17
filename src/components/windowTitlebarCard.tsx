import { useCurrentPlatform } from "@/store/settings/hooks";
import "./titlebar.css"
import { Window } from '@tauri-apps/api/window';

export default function WindowTitlebarCard() {
    const platform = useCurrentPlatform()
    const appWindow = new Window("main");

    if (platform == "windows" || platform == "linux") {
        return (<>
            {
                <div style={{ position: "fixed", top: 0, width: "100%", "zIndex": "1000" }}>
                    <div data-tauri-drag-region className="titlebar">
                        <div className="titlebar-button" id="titlebar-minimize" onClick={() => appWindow.minimize()}>
                            <img
                                src="/mdi_window-minimize.svg"
                                alt="minimize"
                            />
                        </div>
                        <div className="titlebar-button" id="titlebar-maximize" onClick={() => appWindow.toggleMaximize()}>
                            <img
                                src="/mdi_window-maximize.svg"
                                alt="maximize"
                            />
                        </div>
                        <div className="titlebar-button" id="titlebar-close" onClick={() => appWindow.close()}>
                            <img src="/mdi_close.svg" alt="close" />
                        </div>
                    </div>
                </div>}
        </>)
    } else if (platform == "android" || platform == "ios") {
        return (<>
            {<div style={{ position: "fixed", top: 0, height: "20px", width: "100%", "zIndex": "1000", backgroundColor: "rgba(110, 111, 113, 0.5)" }}></div>}
        </>)
    } else {
        return (<>
            {<div data-tauri-drag-region style={{ position: "fixed", top: 0, height: "20px", width: "100%", "zIndex": "1000" }}></div>}
        </>)
    }
}