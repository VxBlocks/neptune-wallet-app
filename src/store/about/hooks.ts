import { useAppSelector } from "@/store/hooks";

export const useLoadingAbout = () => {
    return useAppSelector(state => state.about.loadingAbout);
}
export const useBuildInfo = () => {
    return useAppSelector(state => state.about.buildInfo);
}

export const useVersion = () => {
    return useAppSelector(state => state.about.version);
}

export const useTauriVersion = () => {
    return useAppSelector(state => state.about.tauriVersion);
}
export const useUpdateVersion = () => {
    return useAppSelector(state => state.about.updateVersion);
} 