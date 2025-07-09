import CopyedIcon from "@/components/copyed-icon";
import { useAvailableUtxos, useLoadingAvailableUtxos } from "@/store/history/hooks";
import { ellipsisFormatLen } from "@/utils/ellipsis-format";
import { amount_to_fixed } from "@/utils/math-util";
import { format } from "date-fns";
import { Box, Button, Center, Checkbox, Flex, LoadingOverlay, Menu, NumberFormatter, ScrollArea, Switch, Table, Text } from "@mantine/core";
import { queryAvailableUtxosList } from "@/store/history/history-slice";
import { useAppDispatch } from "@/store/hooks";
import { useSettingActionData } from "@/store/settings/hooks";
import { useLatestBlock, useSyncedBlock } from "@/store/sync/hooks";
import { useCurrentWalledId } from "@/store/wallet/hooks";
import { useState, useEffect } from "react";
import EmptyTable from "@/components/empty-table";
import { IconSortDescending } from "@tabler/icons-react";
import { useNavigate } from "react-router-dom";

export default function NewUtxoTable() {
    const loading = useLoadingAvailableUtxos()
    const availableUtxos = useAvailableUtxos()
    const navigate = useNavigate()
    const [sortType, setSortType] = useState<string>("Amount")

    const addressId = useCurrentWalledId()
    const { serverUrl } = useSettingActionData()
    const dispatch = useAppDispatch()

    const latestBlock = useLatestBlock();
    const syncedBlock = useSyncedBlock()
    const [containLocked, setContainLocked] = useState(false)

    const [selectedRows, setSelectedRows] = useState<string[]>([]);
    useEffect(() => {
        if (latestBlock && syncedBlock && latestBlock <= syncedBlock) {
            dispatch(queryAvailableUtxosList({ serverUrl, sortType, containLocked }))
        }
    }, [latestBlock, syncedBlock, addressId, containLocked])

    function onchangeSortType(type: string) {
        setSortType(type)
    }

    useEffect(() => {
        dispatch(queryAvailableUtxosList({ serverUrl, sortType }))
    }, [sortType, serverUrl]);
    const rows = availableUtxos && availableUtxos.map((element) => (
        <Table.Tr key={element.id}>
            <Table.Td>
                <Checkbox
                    aria-label="Select row"
                    disabled={element.locked||element.confirm_height > 21310}
                    checked={selectedRows.includes(element.id)}
                    onChange={(event) =>
                        setSelectedRows(
                            event.currentTarget.checked
                                ? [...selectedRows, element.id]
                                : selectedRows.filter((position) => position !== element.id)
                        )
                    }
                />
            </Table.Td>
            <Table.Td>
                <Center>
                    <NumberFormatter value={element.id} thousandSeparator />
                </Center>
            </Table.Td>
            <Table.Td>
                <Text c={element.confirm_height > 21310? "#FF0000":"#0A8030"}>
                    <NumberFormatter value={element.confirm_height} thousandSeparator />
                </Text>
            </Table.Td>
            <Table.Td>
                <Center>
                    <Text c={"#0A8030"}>
                        <NumberFormatter value={amount_to_fixed(element.amount)} thousandSeparator />
                    </Text>
                </Center>
            </Table.Td>
            <Table.Td>
                <Flex direction={"row"} gap={8} align={"center"}>
                    <Text>
                        {ellipsisFormatLen(element.hash, 12)}
                    </Text>
                    <CopyedIcon size={16} value={element.hash} />
                </Flex>
            </Table.Td>

            <Table.Td>
                <Center>
                    <Text c={element.locked ? "grey" : "green"}>
                        {element.locked ? "True" : "False"}
                    </Text>
                </Center>
            </Table.Td>
            <Table.Td>
                <Center>
                    <Text c={"#0A8030"}>
                        {format(element.confirm_timestamp, 'yyyy-MM-dd HH:mm:ss')}
                    </Text>
                </Center>

            </Table.Td>
        </Table.Tr>
    ));

    function navigateToSend() {
        navigate("/send", { state: selectedRows })
    }
    return (
        <Flex direction={"column"} gap={8}>
            <Flex direction={"row"} justify={"space-between"} align={"center"} >
                {selectedRows.length > 0 ?
                    <Flex direction={"row"} gap={16}>
                        <Button size="compact-xs" variant="light" onClick={navigateToSend}>Send</Button>
                        <Text c={"#858585"} style={{ fontSize: "14px" }}>{`(${selectedRows.length} Utxos)`}</Text>
                    </Flex>
                    : <Flex direction={"row"} gap={16}></Flex>
                }
                <Flex justify={"end"} gap={16}>
                    <Flex direction={"row"} gap={8}>
                        <Text>{"Contain Locked"}</Text>
                        <Switch
                            checked={containLocked}
                            onChange={(event) => {
                                setContainLocked(event.currentTarget.checked)
                            }}
                        />
                    </Flex>
                    <Flex direction={"row"} gap={8}>
                        <Text c={"#858585"} style={{ textAlign: "end" }}>Sort by</Text>
                        <Menu shadow="md" width={120}>
                            <Menu.Target>
                                <Flex direction={"row"} gap={2} align={"center"} style={{ cursor: "pointer" }}>
                                    <Text c={"var(--primaryhighlight)"} style={{ textAlign: "end", fontSize: "14px" }}>{sortType}</Text>
                                    <IconSortDescending size={14} style={{ color: "var(--primaryhighlight)" }} />
                                </Flex>
                            </Menu.Target>
                            <Menu.Dropdown>
                                <Menu.Item
                                    color={sortType == "Amount" ? "var(--primaryhighlight)" : ""}
                                    onClick={() => onchangeSortType("Amount")}
                                >
                                    Amount
                                </Menu.Item>
                                <Menu.Item
                                    color={sortType == "ID" ? "var(--primaryhighlight)" : ""}
                                    onClick={() => onchangeSortType("ID")}
                                >
                                    ID
                                </Menu.Item>
                            </Menu.Dropdown>
                        </Menu>
                    </Flex>
                </Flex>
            </Flex>
            <Box pos="relative">
                <LoadingOverlay
                    visible={loading}
                    zIndex={1000}
                    overlayProps={{ radius: 'sm', blur: 2 }}
                    loaderProps={{ color: 'pink' }}
                />
                {
                    !loading && availableUtxos && availableUtxos.length > 0 ?
                        <ScrollArea h={"calc(100vh - 320px)"} scrollbarSize={0}>
                            <Table striped highlightOnHover stickyHeaderOffset={0} stickyHeader verticalSpacing={"sm"} withRowBorders={false}>
                                <Table.Thead>
                                    <Table.Tr>
                                        <Table.Th />
                                        <Table.Th>
                                            <Center>
                                                ID
                                            </Center>
                                        </Table.Th>
                                        <Table.Th>Height</Table.Th>
                                        <Table.Th>
                                            <Center>
                                                Amount
                                            </Center>
                                        </Table.Th>
                                        <Table.Th>
                                            Hash
                                        </Table.Th>
                                        <Table.Th>
                                            <Center>
                                                Locked
                                            </Center>
                                        </Table.Th>
                                        <Table.Th>
                                            <Center>
                                                Time
                                            </Center>
                                        </Table.Th>
                                    </Table.Tr>
                                </Table.Thead>
                                <Table.Tbody>{rows}</Table.Tbody>
                            </Table>
                        </ScrollArea> :
                        <EmptyTable />
                }
            </Box>
        </Flex>
    )
}