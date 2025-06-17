import { addWallet, setCurrentWallet } from "@/commands/wallet";
import { useAppDispatch } from "@/store/hooks";
import { useSettingActionData } from "@/store/settings/hooks";
import { useLatestBlock } from "@/store/sync/hooks";
import { queryLatestBlock } from "@/store/sync/sync-slice";
import { Button, Flex, TextInput, Text, Box, LoadingOverlay, Grid, Center } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { IconCircleCheck, IconCopy, IconEye, IconReload } from "@tabler/icons-react";
import { useEffect, useState } from "react";

export default function CreateWallet({ onCreated, mnemonic, refreshMnemonic }: { onCreated: () => void, mnemonic: string, refreshMnemonic: () => void }) {
    const [name, setName] = useState("");
    const [loading, setLoading] = useState(false);
    const { serverUrl } = useSettingActionData()
    const [showCopyIcon, setShowCopyIcon] = useState(false) 
    const [visibleMnemonic, setVisibleMnemonic] = useState(false);
    const [copyed, setCopyed] = useState(false);
    const latestBlock = useLatestBlock()

    const dispatch = useAppDispatch()

    function showMnemonic() {
        setVisibleMnemonic(true)
        setTimeout(() => {
            setVisibleMnemonic(false)
        }, 5000)
    }


    useEffect(() => {
        dispatch(queryLatestBlock({ serverUrl }))
    }, [dispatch, serverUrl])
    async function handleCreate() {
        try {
            setLoading(true);
            let walletID = await addWallet(name, mnemonic, 25, latestBlock, true);
            await setCurrentWallet(walletID); 
            onCreated()
            notifications.show({
                position: 'top-right',
                message: "Wallet created successfully!",
                color: "green",
                title: "Success",
            });
        }
        catch (error: any) {
            console.log(error);
            notifications.show({
                position: 'top-right',
                message: error ?? "Add wallet failed!",
                color: "red",
                title: "Error",
            })
        }
        setLoading(false);
    }
    return (<Flex direction={"column"} gap={8} style={{ minHeight: "200px", marginTop: "8px" }}>
        <TextInput
            data-autofocus
            label="Wallet Name"
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="Enter a name for your wallet" 
            
        />
        <Flex direction={"column"}>
            <Flex direction={"row"} gap={4}>
                <Text>
                    18-Word Phrase
                </Text>
                <Text c="var(--input-asterisk-color, var(--mantine-color-error))">
                    *
                </Text>
            </Flex>
            <Box pos="relative">
                <LoadingOverlay
                    visible={!visibleMnemonic}
                    overlayProps={{ radius: 'sm', blur: 4 }}
                    loaderProps={{
                        children: <Center style={{ cursor: "pointer" }}
                            onClick={() => showMnemonic()}>
                            <Flex direction={"column"} align={"center"}>
                                <IconEye />
                                <Text>
                                    Make sure nobody is looking
                                </Text>
                            </Flex>
                        </Center>
                    }} />
                <Box style={{ width: "100%", border: "1px solid #000000", borderRadius: "5px", padding: "16px" }}>
                    <Grid>
                        {mnemonic && mnemonic.split(" ").map((word, index) => {
                            return (<Grid.Col span={4} key={index}>
                                <Flex direction={"row"} justify={"center"} align={"center"} gap={8}>
                                    <Text style={{ minWidth: "18px", textAlign: "center" }} fw={"bold"}>{`${index + 1}.`}</Text>
                                    <Flex style={{
                                        border: "1px solid #000000",
                                        borderRadius: "5px",
                                        padding: "4px",
                                        minWidth: "120px"
                                    }} justify={"center"}>
                                        <Text>
                                            {word}
                                        </Text>
                                    </Flex>
                                </Flex>
                            </Grid.Col>)
                        })}
                    </Grid>
                </Box>
            </Box>
        </Flex>
        {
            showCopyIcon ? <Flex direction={"row"} px={"lg"} justify={"space-between"} align={"center"} w={"100%"}>
                <Flex
                    direction={"row"}
                    align={"center"} gap={8}
                    style={{ cursor: "pointer", caretColor: "transparent", }}
                    onClick={() => {
                        refreshMnemonic()
                        showMnemonic()
                    }}>
                    <IconReload />
                    <Text fz={14} fw={600}>{
                        "Change Seed Phrase"
                    }</Text>
                </Flex>
                <Flex direction={"row"} align={"center"} gap={8} style={{ cursor: "pointer", caretColor: "transparent", }}
                    onClick={() => {
                        if (copyed) {
                            return;
                        }
                        navigator.clipboard.writeText(mnemonic)
                        setCopyed(true)
                        setTimeout(() => {
                            setCopyed(false)
                        }, 2000)
                    }}
                >
                    {
                        copyed ? <IconCircleCheck color="green" /> : <IconCopy />
                    }
                    <Text fz={14} fw={600}>
                        {
                            copyed ? "Copied" : "Copy to clipboard"
                        }
                    </Text>
                </Flex>
            </Flex> : null
        }

        <Flex direction={"row"}
            align={"center"}
            justify={"center"} gap={8} style={{ cursor: "pointer", caretColor: "transparent", marginTop: "16px" }} w={"100%"}>
            {
                showCopyIcon ?
                    <Button variant="light" fullWidth disabled={!name || !mnemonic} loading={loading} onClick={handleCreate}>
                        Create
                    </Button> :
                    <Button variant="light" fullWidth onClick={() => {
                        setShowCopyIcon(true)
                        showMnemonic()
                    }}>
                        Reveal Secret Recovery Phrase
                    </Button>
            }

        </Flex>
    </Flex>)
}