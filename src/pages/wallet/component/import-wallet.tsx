import { addWallet, setCurrentWallet } from "@/commands/wallet";
import { Button, Flex, NumberInput, Textarea, TextInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useState } from "react";

export default function ImportWallet({ onCreated }: { onCreated: () => void }) {
    const [importData, setImportData] = useState({
        name: "",
        mnemonic: "",
        numKeys: 25,
        startHeight: 0,
    });
    const [loading, setLoading] = useState(false);

    async function handleImport() {
        try {
            setLoading(true);
            let walletID = await addWallet(
                importData.name,
                importData.mnemonic,
                importData.numKeys || 25,
                importData.startHeight || 0,
                false
            );
            await setCurrentWallet(walletID);
            onCreated()
        } catch (error: any) {
            console.log(error);
            notifications.show({
                position: 'top-right',
                message: error || "Failed to import wallet",
                color: "red",
                title: "Error",
            })
        }
        setLoading(false);
    }

    function checkDisabled() {
        if (importData.name === "" || importData.mnemonic === "" || importData.numKeys === 0) {
            return true;
        }
        return false;
    }

    return (<Flex direction={"column"} gap={8} style={{ height: "100%", marginTop: "8px" }}>
        <TextInput
            label="Wallet Name"
            data-autofocus
            placeholder="Enter a name for your wallet"
            value={importData.name}
            onChange={(event) =>
                setImportData({
                    ...importData,
                    name: event.target.value,
                })
            }

        />
        <Textarea
            label="18-Word Phrase"
            placeholder="Enter a mnemonic phrase"
            autosize
            minRows={4}
            value={importData.mnemonic}
            onChange={(event) => {
                if (event && event.target.value) {
                    let newValue = event.target.value.split('\n').map(line => line.replace(/^\d+\.\s*/, '').trim()).join(' ');
                    setImportData({
                        ...importData,
                        mnemonic: newValue,
                    })
                } else {
                    setImportData({
                        ...importData,
                        mnemonic: "",
                    })
                }
            }


            }
        />
        <Flex direction={"row"} gap={16}>
            <NumberInput
                label="Num Keys"
                w={"50%"}
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
                label="Start Height"
                thousandSeparator
                w={"50%"}
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
        <Button variant="light" disabled={checkDisabled()} loading={loading} onClick={handleImport}>Import</Button>
    </Flex>)
}