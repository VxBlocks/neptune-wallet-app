import { set_password } from "@/commands/password";
import { addWallet } from "@/commands/wallet";
import { useAppDispatch } from "@/store/hooks";
import { useOneTimePassword, useOneTimeWalletName } from "@/store/wallet/hooks";
import { setOneTimePassword } from "@/store/wallet/wallet-slice";
import { Button, Flex, NumberInput, Stack, Text, Textarea } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useState } from "react";


export default function ImportCecret({ nextStep }: { nextStep: () => void }) {

    const [importData, setImportData] = useState({
        name: "",
        mnemonic: "",
        numKeys: 25,
        startHeight: 0,
    });
    const [loading, setLoading] = useState(false);
    const walletName = useOneTimeWalletName()
    const oneTimePassword = useOneTimePassword()
    const dispatch = useAppDispatch()

    async function handleImport() {
        setLoading(true);
        try {
            await set_password("", oneTimePassword)
            await addWallet(
                walletName,
                importData.mnemonic,
                importData.numKeys || 25,
                importData.startHeight || 0,
                false
            );
            dispatch(setOneTimePassword(""))
            nextStep()
        } catch (error: any) {
            notifications.show({
                position: 'top-right',
                message: error || "Failed to import wallet",
                color: "red",
                title: "Error",
            })
        }
        setLoading(false);
    }

    return (<Flex direction="column" justify={"center"} align="center" gap={8} w={"100%"}>
        <Text fz={14} fw={600} style={{ textAlign: "center" }}>
            Access your wallet with your Secret Recovery Phrase.
        </Text>
        <Stack w={"100%"}>
            <Textarea
                label="18-word Phrase"
                value={importData.mnemonic}
                onChange={(event) => {
                    if (event && event.target.value) {
                        let newValue = event.target.value.split('\n').map(line => line.replace(/^\d+\.\s*/, '').trim()).join(' ');
                        setImportData({ ...importData, mnemonic: newValue })
                    } else {
                        setImportData({
                            ...importData,
                            mnemonic: "",
                        })
                    }
                }}
                placeholder="Enter your secret recovery phrase here"
                rows={4} />

            <Flex direction={"row"} gap={16} w={"100%"}>
                <NumberInput
                    w={"50%"}
                    label="Num Keys"
                    placeholder="Enter the number keys"
                    min={1}
                    hideControls
                    thousandSeparator
                    allowDecimal={false}
                    allowNegative={false}
                    value={importData.numKeys}
                    onChange={(value) =>
                        setImportData({
                            ...importData,
                            numKeys: Number(value),
                        })
                    }
                />
                <NumberInput

                    w={"50%"}
                    label="Start Height"
                    thousandSeparator
                    placeholder="Enter the start height"
                    min={0}
                    hideControls
                    allowDecimal={false}
                    allowNegative={false}
                    value={importData.startHeight}
                    onChange={(value) =>
                        setImportData({
                            ...importData,
                            startHeight: Number(value),
                        })
                    }
                />
            </Flex>
        </Stack>


        <Flex justify={"center"} align={"center"} w={"100%"} style={{ marginTop: "10px" }}>
            <Button variant="light" fullWidth disabled={!importData.mnemonic} loading={loading} onClick={handleImport}>
                Create a new wallet
            </Button>
        </Flex>
    </Flex>)
}