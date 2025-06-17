import { useAppSelector } from "@/store/hooks";


export const useLoadingLogs = () => {
    return useAppSelector(state => state.log.loadingLogs);
}
export const useLogs = () => {
    return useAppSelector(state => state.log.logs);
} 
