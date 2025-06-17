import { useAppSelector } from "@/store/hooks";

export const useLoadingExecution = () => {
    return useAppSelector(state => state.execution.loadingExecution);
}
export const useExecutionDatas = () => {
    return useAppSelector(state => state.execution.executionData);
}

export const useSendState = () => {
    return useAppSelector(state => state.execution.send_state);
}

export const usePendingExecution = () => {
    return useAppSelector(state => state.execution.executionPending);
}

export const useRequesetSendTransactionResponse = () => {
    return useAppSelector(state => state.execution.requesetSendTransactionResponse);
} 