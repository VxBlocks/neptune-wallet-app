import { useAppSelector } from "@/store/hooks";

export const useLoadingSttrings = () => {
    return useAppSelector(state => state.settings.loadingSettings);
}
export const useSettingActionData = () => {
    return useAppSelector(state => state.settings.acctionData);
} 

export const useCurrentPlatform = () => {
    return useAppSelector(state => state.settings.platform);
} 

export const useCacheFiles = () => {
    return useAppSelector(state => state.settings.cacheFiles);
} 