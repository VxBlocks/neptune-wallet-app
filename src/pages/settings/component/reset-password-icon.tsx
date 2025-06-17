import { IconPassword } from "@tabler/icons-react"
import { useState } from "react"
import ResetPasswordModal from "./reset-password-modal"

export default function ResetPasswordIcon() {
    const [showModal, setShowModal] = useState(false)
    return (<>
        <ResetPasswordModal opened={showModal} close={() => setShowModal(false)} />
        <IconPassword size={18} style={{ cursor: "pointer" }} onClick={() => setShowModal(true)} /> </>)
}