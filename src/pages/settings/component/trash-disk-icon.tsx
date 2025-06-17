import { delete_cache } from "@/commands/config"
import { useAppDispatch } from "@/store/hooks"
import { useCacheFiles } from "@/store/settings/hooks"
import { queryDiskCacheFiles } from "@/store/settings/settings-slice"
import { BlockCacheFile } from "@/store/types"
import { Button, Flex, FocusTrap, Modal, NumberFormatter, ScrollArea, Table } from "@mantine/core"
import { notifications } from "@mantine/notifications"
import { IconTrashX } from "@tabler/icons-react"
import { useEffect, useState } from "react"

export default function TrashDiskIcon() {
    const [showModal, setShowModal] = useState(false)
    const dispatch = useAppDispatch()
    useEffect(() => {
        if (showModal) {
            dispatch(queryDiskCacheFiles())
        }
    }, [dispatch, showModal])
    return (
        <>
            <TrashModal opened={showModal} close={() => setShowModal(false)} />
            <IconTrashX size={18} style={{ cursor: "pointer" }} onClick={() => setShowModal(true)} />
        </>
    )
}
export function TrashModal({ opened, close }: { opened: boolean, close: () => void }) {
    const cacheFiles = useCacheFiles()
    const rows = cacheFiles && cacheFiles.map((item, index) => (
        <ChacheFileItem key={index} item={item} />
    ));
    return (<Modal opened={opened} size="auto" onClose={close} title="Delete Disk Caches" centered
        scrollAreaComponent={ScrollArea.Autosize}>
        <FocusTrap.InitialFocus />
        <Table striped withTableBorder horizontalSpacing="lg" verticalSpacing="sm"
            styles={{
                th: {
                    textAlign: "center",
                    alignItems: "center",
                },
                td: {
                    textAlign: "center",
                    alignItems: "center",
                }
            }}
        >
            <Table.Thead>
                <Table.Tr>
                    <Table.Th>
                        Network
                    </Table.Th>
                    <Table.Th>
                        Block Range
                    </Table.Th>
                    <Table.Th>
                        Options
                    </Table.Th>
                </Table.Tr>
            </Table.Thead>
            <Table.Tbody>{rows}</Table.Tbody>
        </Table>
    </Modal>)
}

export function ChacheFileItem({ item }: { item: BlockCacheFile }) {
    const dispatch = useAppDispatch()
    const [loading, setLoading] = useState(false)
    async function deleteCache() {
        setLoading(true)
        try {
            await delete_cache(item.path)
            dispatch(queryDiskCacheFiles())
            notifications.show({
                position: "top-right",
                title: "Success",
                message: "Delete cache file success!",
                color: "green",
            })
        } catch (error: any) {
            notifications.show({
                position: "top-right",
                title: "Error",
                message: error || "Delete cache file failed!",
                color: "red",
            })
        }
        setLoading(false)
    }
    return (<Table.Tr>
        <Table.Td>
            {item.network == "main" ? "Mainnet" : "Testnet"}
        </Table.Td>
        <Table.Td>
            {item.range && item.range.length > 1 ?
                <Flex direction="row" gap={3}>
                    <NumberFormatter value={item.range[0]} thousandSeparator />
                    {"-"}
                    <NumberFormatter value={item.range[1]} thousandSeparator />
                </Flex> : "--"}
        </Table.Td>
        <Table.Td>
            <Button color="red" loading={loading} size="compact-xs" onClick={() => deleteCache()}> Delete </Button>
        </Table.Td>
    </Table.Tr>)
}