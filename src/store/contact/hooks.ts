import { useAppSelector } from "@/store/hooks";

export const useLoadingContacts = () => {
    return useAppSelector(state => state.contact.loadingContacts);
}
export const useAllContacts = () => {
    return useAppSelector(state => state.contact.contacts);
} 