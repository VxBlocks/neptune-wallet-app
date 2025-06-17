
import WithTitlePageHeader from "@/components/header/withTitlePageHeader.tsx";
import { Alert, Button, Flex, HoverCard, NumberInput, ScrollArea, Stack, Text } from "@mantine/core";
import { IconAddressBook, IconInfoCircle, IconPlus } from "@tabler/icons-react";
import { useEffect, useState } from "react";
import { Output, SendInputItem, SendTransactionParam } from "@/utils/api/types.ts";
import TransferForm from "@/pages/batch/component/transfer-form.tsx";
import { useLatestBlock, useSyncedBlock } from "@/store/sync/hooks.ts";
import { notifications } from "@mantine/notifications";
import { queryExecutionHistorys, requestSedExecutionTransaction } from "@/store/execution/execution-slice.ts";
import { useSettingActionData } from "@/store/settings/hooks.ts";
import { useAppDispatch } from "@/store/hooks.ts";
import { usePendingExecution, useRequesetSendTransactionResponse, useSendState } from "@/store/execution/hooks.ts";
import { useBalanceData, useCurrentAddress, useCurrentWalledId } from "@/store/wallet/hooks.ts";

import { queryCurrentWalletID, queryWalletBalance } from "@/store/wallet/wallet-slice.ts";
import ExecutionCard from "./component/execution-card";
import ContactModal from "./component/contact-modal";
import { useLocation } from "react-router-dom";
import { useAvailableUtxos } from "@/store/history/hooks";
import { bigNumberPlusToString } from "@/utils/common";
import { amount_to_positive_fixed } from "@/utils/math-util";

export default function BatchTranferPage() {
    const { serverUrl } = useSettingActionData()
    const loading = usePendingExecution()
    const dispatch = useAppDispatch()
    const sendStatus = useSendState()
    const location = useLocation()
    const [sendInputs, setSendInputs] = useState([
        {
            index: 0,
            toAddress: "",
            amount: ""
        }
    ] as SendInputItem[]);
    const [fee, setFee] = useState<string>("0.5");

    const latestBlock = useLatestBlock()
    const currentWalletID = useCurrentWalledId()
    const currentAddress = useCurrentAddress()
    const syncedBlock = useSyncedBlock();
    const [showContactModal, setShowContactModal] = useState(false)
    const requesTransactionResponse = useRequesetSendTransactionResponse()
    const [selectedInputs, setSelectedInputs] = useState([] as number[])
    const [selectedAmount, setSelectedAmount] = useState("")
    const availableUtxos = useAvailableUtxos()
    const balanceData = useBalanceData()
    useEffect(() => {
        dispatch(queryCurrentWalletID())
        dispatch(queryWalletBalance({ serverUrl }))
    }, [serverUrl])
    useEffect(() => {
        if (location.state) {
            setSelectedInputs(location.state)
            handleSelectedAmount(location.state)
        }
    }, [location]);

    function handleSelectedAmount(inputs: number[]) {
        let selectedAmount = "0"
        inputs.forEach((item) => {
            let seleced = availableUtxos.find((data) => Number(data.id) === item)
            if (seleced) {
                selectedAmount = bigNumberPlusToString(selectedAmount, seleced.amount)
            }
        })
        setSelectedAmount(amount_to_positive_fixed(selectedAmount))
    }

    useEffect(() => {
        dispatch(queryExecutionHistorys({ addressId: currentWalletID, serverUrl }))
    }, [dispatch, currentWalletID, serverUrl])

    function checkButtonDisabled() {
        let disabledButton = false
        if (loading) {
            return disabledButton
        }
        if (syncedBlock != 0 && syncedBlock < latestBlock) {
            disabledButton = true
        }
        let findInput = sendInputs.find((item) => !item.toAddress || !item.amount)
        if (findInput) {
            disabledButton = true
        }
        return disabledButton
    }

    function queryNextIndex() {
        let maxIndex = 0;
        sendInputs.find((item) => {
            if (item.index > maxIndex) {
                maxIndex = item.index;
            }
        })
        return maxIndex + 1;
    }

    async function handleSendButtonClick() {
        let hasEmptyInput = false
        let findInput = sendInputs.find((item) => !item.toAddress || !item.amount)
        if (findInput) {
            hasEmptyInput = true
        }
        if (hasEmptyInput) {
            notifications.show({
                position: 'top-right',
                color: "red",
                title: "Error",
                message: "Please complete all required fields.",
            })
            return
        }
        let outputs = [] as Output[]
        sendInputs.forEach((item) => {
            outputs.push({ address: item.toAddress, amount: item.amount.toString() })
        })
        let param = {
            outputs,
            fee: fee.toString(),
            input_rule: "maximum",
            inputs: selectedInputs
        } as SendTransactionParam

        dispatch(requestSedExecutionTransaction({
            serverUrl,
            param,
            syncedBlock,
            currentWalletID,
            currentAddress,
            sendInputs
        }))
    }

    useEffect(() => {
        handleRequesTransactionResponse()
    }, [requesTransactionResponse])
    function handleRequesTransactionResponse() {
        if (requesTransactionResponse && requesTransactionResponse.transaction) {
            clearDatas()
        }
    }
    function clearDatas() {
        setSendInputs([{
            index: 0,
            toAddress: "",
            amount: ""
        }] as SendInputItem[]
        )
        setFee("0.5"),
            setSelectedInputs([])
    }
    return (<ScrollArea w={"100%"} h={"calc(100vh - 12px)"} scrollbarSize={8}>
        <ExecutionCard />
        <ContactModal opened={showContactModal} close={() => setShowContactModal(false)} />
        <WithTitlePageHeader title="​Send" buttons={<Button
            onClick={() => setShowContactModal(true)}
            variant="light"
            size="xs"
            leftSection={<IconAddressBook size={14} />}
        >
            Contact
        </Button>}>
            <Flex direction={"row"} justify={"space-between"}>
                <Flex direction={"row"} gap={8}>
                    {
                        selectedInputs && selectedInputs.length > 0 &&
                        <Flex direction={"row"} gap={8}>
                            <Text c="gray">
                                {`Selected ${selectedInputs.length} Utxos Amount:`}
                            </Text>
                            <HoverCard width={320} shadow="md" withArrow openDelay={200} closeDelay={400}>
                                <HoverCard.Target>
                                    <Text fw={600} c="green" style={{
                                        wordWrap: "break-word",
                                        overflowWrap: "break-word",
                                    }}>
                                        {`${selectedAmount}`}
                                    </Text>
                                </HoverCard.Target>
                                <HoverCard.Dropdown>
                                    <Stack gap={5}>
                                        <Text size="sm" fw={700} style={{ lineHeight: 1 }}>
                                            Selected Utxo IDs
                                        </Text>
                                    </Stack>
                                    <Text size="xs" mt="xs">
                                        {`[${selectedInputs.join(", ")}]`}
                                    </Text>
                                </HoverCard.Dropdown>
                            </HoverCard>
                        </Flex>
                    }
                </Flex>

                <Flex direction={"row"} gap={8}>
                    <Text c="gray">Available balance:</Text>
                    <Text fw={600} c="green">{balanceData.available_balance}</Text>
                </Flex>
            </Flex>
            <Flex justify={"end"} direction={"row"}>

            </Flex>
            <Flex direction={"column"} gap={16} style={{ marginTop: "8px" }}>
                {sendInputs && sendInputs.length > 0 && sendInputs.map((item, index) => {
                    return (
                        <TransferForm
                            key={index}
                            keyIndex={index}
                            showRemove={sendInputs.length > 1}
                            onChangeAmount={(amount) => {
                                setSendInputs(prev =>
                                    prev.map((item, i) =>
                                        i === index
                                            ? { ...item, amount: amount }
                                            : item
                                    )
                                );
                            }}
                            onChangeToAddress={(address) => {
                                setSendInputs(prev =>
                                    prev.map((item, i) =>
                                        i === index
                                            ? { ...item, toAddress: address }
                                            : item
                                    )
                                );
                            }}
                            onRemoveWallet={(removeIndex) => {
                                const newItems = sendInputs.filter(input => input.index !== removeIndex);
                                setSendInputs(newItems);
                            }} data={item} />)
                })}
            </Flex>
            <Flex direction={"column"} justify={"center"} align={"start"} style={{ marginTop: "16px" }}>
                <Button
                    size="compact-xs"
                    variant="light"
                    leftSection={<IconPlus size={14} />}
                    onClick={() => {
                        let newSendInput = {
                            index: queryNextIndex(),
                            toAddress: "",
                            amount: ""
                        }
                        setSendInputs([...sendInputs, newSendInput])
                    }}>
                    Add Address
                </Button>
            </Flex>

            <Flex direction={"column"} style={{ marginTop: "16px" }}>
                <NumberInput
                    label={"Fee"}
                    styles={{
                        label: {
                            fontSize: "16px",
                            fontWeight: "bold",
                        }
                    }}
                    value={fee}
                    onChange={(value) => setFee(value.toString())}
                    required
                    placeholder="Input fee to send"
                    hideControls
                />
            </Flex>

            <Flex direction={"column"} justify={"center"} align={"center"} gap={16}>
                <Flex justify={"center"} style={{ marginTop: "16px" }}>
                    <Flex direction={"row"} gap={24}>
                        <Button variant={"light"} disabled={checkButtonDisabled()} loading={loading} onClick={handleSendButtonClick}>
                            Send
                        </Button>
                    </Flex>
                </Flex>
                {
                    (syncedBlock != 0 && syncedBlock < latestBlock) ?
                        <Text c={"red"}>* Wait for syncing...</Text> : null
                }
                {
                    (sendStatus) ?
                        <Alert variant="light" color="blue" title="Send Transaction status"
                            style={{ minWidth: "480px" }} icon={<IconInfoCircle />}>
                            {sendStatus}
                        </Alert> : null
                }
            </Flex>
        </WithTitlePageHeader>
    </ScrollArea>)
}