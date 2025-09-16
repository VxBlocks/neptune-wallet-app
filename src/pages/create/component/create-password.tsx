
import { useAppDispatch } from "@/store/hooks";
import { setMnemonic, setOneTimePassword, setOneTimeWalletName } from "@/store/wallet/wallet-slice";
import { Button, Flex, PasswordInput, Stack, Text } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";
import { useCallback, useState } from "react";
import * as bip39 from '@scure/bip39';
import { wordlist } from '@scure/bip39/wordlists/english';

interface Props {
    nextStep: () => void;
}

export default function CreatePassword(props: Props) {
    const { nextStep } = props
    const [password, setPassword] = useState('')
    const [visible, { toggle }] = useDisclosure(false);
    const [confirmPassword, setConfirmPassword] = useState('')
    const dispatch = useAppDispatch()

    const handleKeyPress = useCallback((event: React.KeyboardEvent<HTMLInputElement>) => {
        if (event.key === 'Enter') {
            event.preventDefault();
            if (password) {
                createPassword()
            }
        }
    }, [createPassword]);

    async function createPassword() {
        try {
            let name = "Account 1"
            dispatch(setOneTimePassword(password));
            dispatch(setOneTimeWalletName(name));
            dispatch(setMnemonic(bip39.generateMnemonic(wordlist, 192)))
            nextStep()
        } catch (error: any) {
            notifications.show({
                position: "top-right",
                message: error || "Failed to create password",
                color: "red",
                title: "Error",
            })
        }
    }
    return (<Flex direction="column" justify={"center"} align="center" gap={8} w={"100%"}>
        <Text fz={16} fw={600} style={{ textAlign: "center" }}>
            This password will unlock your neptune wallet only on this device. Neptune Wallet can not recover this password.
        </Text>
        <Stack w={"100%"}>
            <PasswordInput
                label="Password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                visible={visible}
                onVisibilityChange={toggle}
            />
            <PasswordInput
                label="Confirm password"
                value={confirmPassword}
                onKeyDown={handleKeyPress}
                onChange={(event) => setConfirmPassword(event.target.value)}
                visible={visible}
                onVisibilityChange={toggle}
            />
        </Stack>
        <Flex justify={"center"} align={"center"} w={"100%"} style={{ marginTop: "10px" }}>
            <Button variant="light" fullWidth disabled={!password || !confirmPassword || password !== confirmPassword} onClick={createPassword}>
                Create a new wallet
            </Button>
        </Flex>
    </Flex>)
}
