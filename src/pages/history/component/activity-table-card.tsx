import { queryActivityHistory } from "@/store/history/history-slice"
import { useLoadingActivityTx, useActivityTransactions } from "@/store/history/hooks"
import { useAppDispatch } from "@/store/hooks"
import { useSettingActionData } from "@/store/settings/hooks"
import { useLatestBlock, useSyncedBlock } from "@/store/sync/hooks"
import { useCurrentWalledId } from "@/store/wallet/hooks"
import { Flex, Box, LoadingOverlay, ScrollArea, Table, Center, Select } from "@mantine/core"
import { useState, useEffect } from "react"
import ActivityTableItem from "./activity-table-item"
import DetailModal from "./datail-modal"
import { MerageHistory } from "@/store/types"

export default function ActivityTableCard() {
    const loading = useLoadingActivityTx()
    const historyList = useActivityTransactions()
    const { serverUrl } = useSettingActionData()
    const addressId = useCurrentWalledId()
    const dispatch = useAppDispatch();
    const latestBlock = useLatestBlock();
    const syncedBlock = useSyncedBlock()
    const [historyType, setHistoryType] = useState("All")
    const [selectedHistory, setSelectedHistory] = useState({} as MerageHistory)
    const [showDetail, setShowDetail] = useState(false)
    useEffect(() => {
        if (latestBlock && syncedBlock && latestBlock <= syncedBlock) {
            dispatch(queryActivityHistory({ serverUrl, addressId,historyType }))
        }
    }, [latestBlock, syncedBlock, addressId,historyType])

    useEffect(() => {
        dispatch(queryActivityHistory({ serverUrl, addressId, historyType }))
    }, [dispatch, addressId, serverUrl, historyType]);

    return (<Flex direction={"column"} gap={8}>
        <DetailModal
            history={selectedHistory}
            opened={showDetail} onClose={() => setShowDetail(false)} />
        <Flex justify={"end"}>
            <Select
                styles={
                    {
                        input: {
                            outline: "none",
                            border: "none",
                        },
                    }}
                variant="filled"
                size="xs"
                w={120}
                data={['All', 'Send', 'Receive']}
                value={historyType}
                onChange={(value) => {
                    setHistoryType(value ?? "All")
                }}
                defaultValue={historyType}
                allowDeselect={false}
            />
        </Flex>
        <Box pos="relative">
            <LoadingOverlay
                visible={loading}
                zIndex={1000}
                overlayProps={{ radius: 'sm', blur: 2 }}
                loaderProps={{ color: 'pink' }}
            />
            {
                <ScrollArea h={"calc(100vh - 320px)"} scrollbarSize={0}>
                    <Table striped highlightOnHover stickyHeaderOffset={0} stickyHeader verticalSpacing={"sm"} withRowBorders={false}>
                        <Table.Thead>
                            <Table.Tr>
                                <Table.Th>Height</Table.Th>
                                <Table.Th>
                                    <Center>
                                        Amount Change
                                    </Center>
                                </Table.Th>
                                <Table.Th>
                                    <Center>
                                        Time
                                    </Center>
                                </Table.Th>
                                <Table.Th>
                                </Table.Th>
                            </Table.Tr>
                        </Table.Thead>
                        <Table.Tbody>
                            {
                                historyList && historyList.length > 0 && historyList.map((item, index) => {
                                    return (<ActivityTableItem
                                        key={index}
                                        element={item}
                                        showMoreDetail={() => {
                                            setSelectedHistory(item)
                                            setShowDetail(true)
                                        }} />)
                                })
                            }
                        </Table.Tbody>
                    </Table>
                </ScrollArea>
            }
        </Box>
    </Flex>)
}