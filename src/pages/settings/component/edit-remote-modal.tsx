import { set_rest_url } from "@/commands/config";
import { useAppDispatch } from "@/store/hooks";
import { querySettingActionData } from "@/store/settings/settings-slice";
import { Button, Flex, FocusTrap, Modal, TextInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useEffect, useState } from "react";

export default function EditRemoteModal({ opened, value, close }: { opened: boolean, value: string, close: () => void }) {
    const [newValue, setNewValue] = useState(value)
    const dispatch = useAppDispatch();
    useEffect(() => {
        setNewValue(value)
    }, [value])

    function isValidUrl(urlString: string) {
        const pattern = new RegExp(
            '^(https?:\\/\\/)?' +
            '((([a-z\\d]([a-z\\d-]*[a-z\\d])*)\\.)+[a-z]{2,}|' +
            '((\\d{1,3}\\.){3}\\d{1,3}))' +
            '(\\:\\d+)?(\\/[-a-z\\d%_.~+]*)*' +
            '(\\?[;&a-z\\d%_.~+=-]*)?' +
            '(\\#[-a-z\\d_]*)?$', 'i'
        );
        return pattern.test(urlString);
    }

    async function handleUpdate() {
        if (newValue != "") {
            try {
                if (!isValidUrl(newValue)) {
                    throw "invalid url";
                }
                await set_rest_url(newValue);
                dispatch(querySettingActionData())
                notifications.show({
                    position: "top-right",
                    title: "Success",
                    message: "Update remote rest url successfully.",
                    color: "green",
                })
                close()
            } catch (error: any) {
                notifications.show({
                    position: "top-right",
                    title: "Error",
                    message: error || "Failed to change network.",
                    color: "red",
                })
            }
        }

    }
    return (<Modal opened={opened} onClose={close} title="Update Remote Rest" centered>
        <FocusTrap.InitialFocus />
        <Flex direction="column" gap={16}>
            <TextInput
                label="Remote Rest"
                placeholder="Enter remote rest url"
                value={newValue}
                onChange={(e) => setNewValue(e.target.value.trim())}
            />
            <Flex direction={"row"} justify={"space-between"}>
                <Button variant="default" w={"40%"} onClick={() => {
                    setNewValue("http://127.0.0.1:9900")
                }}>Restore default</Button>
                <Button variant={"light"} w={"40%"} disabled={!newValue} onClick={handleUpdate}>Update</Button>
            </Flex>
        </Flex>
    </Modal>)
}