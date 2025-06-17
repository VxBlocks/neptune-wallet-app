
import { useMnemonic } from "@/store/wallet/hooks";
import { Flex, Text, Box, Grid, LoadingOverlay, Center, Button } from "@mantine/core";
import { IconCircleCheck, IconCopy, IconEye, IconReload } from "@tabler/icons-react";
import { useState } from "react";

import * as bip39 from '@scure/bip39';
import { wordlist } from '@scure/bip39/wordlists/english';
import { useAppDispatch } from "@/store/hooks";
import { setMnemonic } from "@/store/wallet/wallet-slice";
import { notifications } from "@mantine/notifications";
interface Props {
    nextStep: () => void;
}
export default function SecureWallet(props: Props) {
    const { nextStep } = props
    const mnemonic = useMnemonic()
    const [showCopyIcon, setShowCopyIcon] = useState(false)
    const [visibleMnemonic, setVisibleMnemonic] = useState(false);
    const [copyed, setCopyed] = useState(false);
    const dispatch = useAppDispatch()

    function showMnemonic() {
        setVisibleMnemonic(true)
        setTimeout(() => {
            setVisibleMnemonic(false)
        }, 5000)
    }

    return (
        <Flex direction="column" justify={"center"} align="center" gap={8} w={"100%"}>
            <Text fz={14} fw={600} style={{ textAlign: "center" }}>
                Write down this 18-word Secret Recovery Phrase and save it in a place that you trust and only you can access.
            </Text>
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

            {
                showCopyIcon ? <Flex direction={"row"} px={"lg"} justify={"space-between"} align={"center"} w={"100%"} mt={2}>
                    <Flex direction={"row"} align={"center"} gap={8} style={{ cursor: "pointer", caretColor: "transparent", }} onClick={() => {
                        dispatch(setMnemonic(bip39.generateMnemonic(wordlist, 192)))
                        showMnemonic()
                        notifications.show({
                            position: "top-right",
                            title: "Success",
                            message: "New seed phrase generated",
                            color: "green",
                        })
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
                        <Button variant="light" fullWidth onClick={nextStep}>
                            Next
                        </Button> :
                        <Button variant="light" fullWidth onClick={() => {
                            setShowCopyIcon(true)
                            showMnemonic()
                        }}>
                            Reveal Secret Recovery Phrase
                        </Button>
                }

            </Flex>


        </Flex>
    )
}