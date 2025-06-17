import {useActivityTransactions, useLoadingActivityTx} from "@/store/history/hooks";
import {Accordion, Box, Flex, LoadingOverlay, ScrollArea, Select} from "@mantine/core";
import NewActivityItem from "./new-activity-item";
import EmptyTable from "@/components/empty-table";
import {useEffect, useState} from "react";
import {queryActivityHistory} from "@/store/history/history-slice.ts";
import {useSettingActionData} from "@/store/settings/hooks.ts";
import {useCurrentWalledId} from "@/store/wallet/hooks.ts";
import {useAppDispatch} from "@/store/hooks.ts";
import {useLatestBlock, useSyncedBlock} from "@/store/sync/hooks.ts";

export default function NewActivityTable() {
    const loading = useLoadingActivityTx()
    const historyList = useActivityTransactions()
    const {serverUrl} = useSettingActionData()
    const addressId = useCurrentWalledId()
    const dispatch = useAppDispatch();
    const latestBlock = useLatestBlock();
    const syncedBlock = useSyncedBlock()
    const [historyType, setHistoryType] = useState("All")
    useEffect(() => {
        if (latestBlock && syncedBlock && latestBlock <= syncedBlock) {
            dispatch(queryActivityHistory({serverUrl, addressId}))
        }
    }, [latestBlock, syncedBlock, addressId])

    useEffect(() => {
        dispatch(queryActivityHistory({serverUrl, addressId, historyType}))
    }, [dispatch, addressId, serverUrl, historyType]);
    return (
        <Flex direction={"column"} gap={16}>
            {
                historyList && historyList.length > 0 ?
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
                    </Flex> : null
            }

            <Box pos="relative">
                <LoadingOverlay
                    visible={loading}
                    zIndex={1000}
                    overlayProps={{radius: 'sm', blur: 2}}
                    loaderProps={{color: 'pink'}}
                />
                {
                    !loading && historyList && historyList.length > 0 ?
                        <ScrollArea h={"calc(100vh - 180px)"} scrollbarSize={0}>
                            <Accordion variant="separated" radius="md">
                                {
                                    historyList && historyList.length > 0 && historyList.map((item, index) => {
                                        return <NewActivityItem keyIndex={index} key={index} item={item}/>
                                    })
                                }
                            </Accordion>
                        </ScrollArea> :
                        <EmptyTable/>
                }
            </Box>
        </Flex>

    )
}