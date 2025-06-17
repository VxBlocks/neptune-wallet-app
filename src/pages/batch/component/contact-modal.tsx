import { Modal } from "@mantine/core";
import ContactTable from "./contact-table";
import { useEffect } from "react";
import { useAppDispatch } from "@/store/hooks";
import { queryAllContacts } from "@/store/contact/contact-slice";

interface Props {
    opened: boolean;
    close: () => void;
}
export default function ContactModal({ opened, close }: Props) {
    const dispatch = useAppDispatch()
    useEffect(() => {
        dispatch(queryAllContacts())
    }, [dispatch])
    return (<Modal size={"lg"} opened={opened} onClose={close} title="Address Book">
        <ContactTable />
    </Modal>)
}