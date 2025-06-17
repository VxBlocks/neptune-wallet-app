import { Tooltip } from "@mantine/core"
import { notifications } from "@mantine/notifications"
import { IconCircleCheck, IconCopy } from "@tabler/icons-react"
import { useState } from "react"

export default function CopyedIcon({ value, size = 18, tooltipLable = "Copy Value" }: { value: string, size?: number, tooltipLable?: string }) {
    const [copyed, setCopyed] = useState(false)
    return (<>
        {
            copyed ? <IconCircleCheck color="green" size={size} /> :
                <Tooltip label={tooltipLable} withArrow>
                    <IconCopy style={{ cursor: "pointer" }} size={size} onClick={() => {
                        navigator.clipboard.writeText(value)
                        setCopyed(true)
                        notifications.show({
                            position: 'top-right',
                            message: 'Copied to clipboard',
                            color: 'green',
                            title: 'Success',
                        })
                        setTimeout(() => {
                            setCopyed(false)
                        }, 2000)
                    }} />
                </Tooltip>
        }</>)
}