import { useState } from "react"
import ResyncModal from "./resync-modal"
import { IconRotate } from "@tabler/icons-react"

export default function ResyncIcon() {
    const [showModal, setShowModal] = useState(false)
    return (<>
        <ResyncModal opened={showModal} close={() => setShowModal(false)}/>
        <IconRotate size={18} style={{ cursor: "pointer" }} onClick={() => setShowModal(true)} />
    </>)
}