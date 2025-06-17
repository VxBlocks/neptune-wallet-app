

import { readLocalStorageValue } from "@mantine/hooks";
import axios from "axios";

const service = axios.create({
    baseURL: "",
    timeout: 1 * 60 * 60 * 1000,

})

service.interceptors.request.use((config) => {
    const token = readLocalStorageValue({ key: "token", defaultValue: "" });
    if (token) {
        config.headers["authorization"] = `${token}`;
    }
    return config;
})

service.interceptors.response.use((response) => {
    if (response.status === 200 && response.data.code === 10012) {
        localStorage.setItem("token", "");
    }
    return response;
}, (error) => {  
    return Promise.reject(error);
});

export const url = (url: string) => {
    return url;
}


export default service;

