import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { HashRouter } from "react-router-dom"
import { MantineProvider } from '@mantine/core';
import theme from "./theme";
import { Notifications } from "@mantine/notifications";
import '@mantine/notifications/styles.css';
import { Provider } from "react-redux";
import { store } from "./store";
import { ModalsProvider } from '@mantine/modals';

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Provider store={store}>
      <HashRouter> 
        <MantineProvider theme={theme}>
          <Notifications />
          <ModalsProvider>
            <App />
          </ModalsProvider>
        </MantineProvider>
      </HashRouter>
    </Provider>
  </React.StrictMode>,
);
