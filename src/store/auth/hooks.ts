import { useAppSelector } from "@/store/hooks";

export const useAuth = () => {
    return useAppSelector(state => state.auth.data);
} 

export const useStartRpcServer = () => {
    return useAppSelector(state => state.auth.startRpcServer);
}