import {useAppSelector} from "@/store/hooks";

export const useLoadingActivityTx = () => {
    return useAppSelector(state => state.history.loadingActivityHistory);
}
export const useActivityTransactions = () => {
    return useAppSelector(state => state.history.activityHistory);
}

export const useActivityPerDay = () => {
    return useAppSelector(state => state.history.perDay);
}

export const useInExecutionTx = () => {
    return useAppSelector(state => state.history.inExecutionTx);
}
export const useLoadingAvailableUtxos = () => {
    return useAppSelector(state => state.history.loadingAvailableUtxos);
}
export const useAvailableUtxos = () => {
    return useAppSelector(state => state.history.availableUtxos);
}


