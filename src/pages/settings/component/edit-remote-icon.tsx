import { IconEdit } from "@tabler/icons-react";
import EditRemoteModal from "./edit-remote-modal";
import { useState } from "react";

export default function EditRemoteIcon({ value }: { value: string }) {
    const [showModal, setShowModal] = useState(false)
    return (<>
        <EditRemoteModal opened={showModal} close={() => setShowModal(false)} value={value} />
        <IconEdit size={18} style={{ cursor: "pointer" }} onClick={() => setShowModal(true)} />
    </>)
}