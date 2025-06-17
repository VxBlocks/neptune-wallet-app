import { useAppSelector } from "@/store/hooks";

export const useSyncedBlock = () => {
    return useAppSelector(state => state.sync.syncedBlock);
}
export const useSyncBlockData = () => {
    return useAppSelector(state => state.sync.syncingData);
}
export const useLatestBlock = () => {
    return useAppSelector(state => state.sync.latestBlock);
}