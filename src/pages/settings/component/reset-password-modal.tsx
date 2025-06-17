import { set_password } from "@/commands/password";
import { useAppDispatch } from "@/store/hooks";
import { querySettingActionData } from "@/store/settings/settings-slice";
import { Modal, Flex, Button, PasswordInput, Stack, Text } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";
import { useEffect, useState } from "react";

export default function ResetPasswordModal({ opened, close }: { opened: boolean, close: () => void }) {
    const [visible, { toggle }] = useDisclosure(false);
    const [oldpassword, setOldPassword] = useState("");
    const [password, setPassword] = useState("");
    const [confirmPassword, setConfirmPassword] = useState("");
    const dispatch = useAppDispatch();
    useEffect(() => {
        setOldPassword("");
        setPassword("");
        setConfirmPassword("");
    }, [opened])
    async function handleSetPassword() {
        if (password === confirmPassword) {
            try {
                await set_password(oldpassword, password);
                dispatch(querySettingActionData())
                notifications.show({
                    position: "top-right",
                    title: "Success",
                    message: "Password updated successfully",
                    color: "green",
                })
                close();
            } catch (error: any) {
                notifications.show({
                    position: "top-right",
                    title: "Error",
                    message: error || "Failed to set password",
                    color: "red",
                })
            }

        } else {
            notifications.show({
                position: "top-right",
                title: "Error",
                message: "Please enter the same password in both fields",
                color: "red",
            })
        }
    }
    return (<Modal opened={opened} onClose={close} title="Reset Password" centered>
        <Flex direction="column" gap={16}> 
            <Stack>
                <PasswordInput
                    label="Old password"
                    value={oldpassword}
                    onChange={(event) => setOldPassword(event.target.value)}
                    visible={visible}
                    onVisibilityChange={toggle}
                />
                <PasswordInput
                    label="New password"
                    value={password}
                    onChange={(event) => setPassword(event.target.value)}
                    visible={visible}
                    onVisibilityChange={toggle}
                />
                <PasswordInput
                    label="Confirm new password"
                    value={confirmPassword}
                    onChange={(event) => setConfirmPassword(event.target.value)}
                    visible={visible}
                    onVisibilityChange={toggle}
                />
            </Stack>
            <Button variant={"light"} disabled={!oldpassword || !password || password !== confirmPassword} onClick={handleSetPassword}>Update</Button>
        </Flex>
    </Modal>)
}