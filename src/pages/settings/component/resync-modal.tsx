
import { resetToHeight } from "@/commands/wallet";
import { Alert, Button, Flex, FocusTrap, Modal, NumberInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useEffect, useState } from "react";

export default function ResyncModal({ opened, close }: { opened: boolean, close: () => void }) {
    const [height, setHeight] = useState("0")
    const [loading, setLoading] = useState(false)

    useEffect(() => {
        setHeight("0")
        setLoading(false)
    }, [opened])

    async function resyncHeight() {
        setLoading(true)
        try {
            await resetToHeight(Number(height))
            notifications.show({
                position: "top-right",
                title: "Success",
                message: "Resync Block Height Successfully!",
                color: "green",
            })
            close()
        } catch (error: any) {
            notifications.show({
                position: "top-right",
                title: "Error",
                message: error || "Resync Block Height Failed!",
                color: "red",
            })
        }
        setLoading(false)

    }
    return (<Modal opened={opened} onClose={close} title="Resync Block" centered>
        <FocusTrap.InitialFocus />
        <Flex direction="column" gap={16}>
            <Alert variant="light" color="yellow" >
                Reset all historical records of the current account and resync the height.
            </Alert>
            <NumberInput
                label="Resync Start Height"
                placeholder="Input height"
                thousandSeparator=","
                rightSection={null}
                value={height}
                allowDecimal={false}
                allowNegative={false}
                hideControls
                onChange={(value) => setHeight(value.toString())}
                min={0}
            />
            <Button loading={loading} variant={"light"} disabled={!height} onClick={() => resyncHeight()}>Resync</Button>
        </Flex>
    </Modal>)
}