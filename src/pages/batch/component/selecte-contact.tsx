import CopyedIcon from "@/components/copyed-icon";
import EmptyTable from "@/components/empty-table";
import { queryAllContacts } from "@/store/contact/contact-slice";
import { useAllContacts, useLoadingContacts } from "@/store/contact/hooks";
import { useAppDispatch } from "@/store/hooks";
import { ellipsis } from "@/utils/ellipsis-format";
import { Box, Button, Checkbox, Flex, LoadingOverlay, Modal, ScrollArea, Table, Text } from "@mantine/core";
import { useEffect, useState } from "react";

export default function SelecteContact({ opened, close, selectedContact }: { opened: boolean, close: () => void, selectedContact: (contact: string) => void }) {
    const dispatch = useAppDispatch()
    const loading = useLoadingContacts()
    const contracts = useAllContacts()
    const [selectedRows, setSelectedRows] = useState<number[]>([]);
    useEffect(() => {
        dispatch(queryAllContacts())
    }, [dispatch])

    useEffect(() => {
        setSelectedRows([])
    }, [opened])

    const rows = contracts && contracts.map((element, index) => (
        <Table.Tr key={index}
            bg={selectedRows.includes(index) ? 'var(--mantine-color-blue-light)' : undefined}
        >
            <Table.Td>
                <Checkbox
                    aria-label="Select row"
                    checked={selectedRows.includes(index)}
                    onChange={(event) => {
                        let checked = event.currentTarget.checked
                        if (checked) {
                            setSelectedRows([index])
                        } else {
                            setSelectedRows(selectedRows.filter((position) => position !== index))
                        }
                    }
                    }
                />
            </Table.Td>
            <Table.Td>
                <Text style={{ minWidth: "115px" }}>{element.aliasName}</Text>
            </Table.Td>
            <Table.Td>
                <Flex direction={"row"} gap={8} align={"center"}>
                    <Text>
                        {ellipsis(element.address)}
                    </Text>
                    <CopyedIcon size={16} value={element.address} />
                </Flex>
            </Table.Td>
        </Table.Tr>
    ));

    return (<Modal size={"lg"} opened={opened} onClose={close} title="Select Contact to Send">
        <Flex direction={"column"} gap={16}>
            <Box pos="relative">
                <LoadingOverlay
                    visible={loading}
                    zIndex={1000}
                    overlayProps={{ radius: 'sm', blur: 2 }}
                    loaderProps={{ color: 'pink' }}
                />
                {
                    !loading && contracts && contracts.length > 0 ?
                        <ScrollArea h={"450px"} scrollbarSize={0}>
                            <Table stickyHeader verticalSpacing="sm" striped highlightOnHover
                                styles={{
                                    thead: {
                                        fontSize: "14px",
                                        fontWeight: 600,
                                    }
                                }}
                            >
                                <Table.Thead>
                                    <Table.Tr>
                                        <Table.Th></Table.Th>
                                        <Table.Th>Name</Table.Th>
                                        <Table.Th>Address</Table.Th>
                                    </Table.Tr>
                                </Table.Thead>
                                <Table.Tbody>{rows}</Table.Tbody>
                            </Table>
                        </ScrollArea> : <EmptyTable />
                }

            </Box>
            <Button variant={"light"} fullWidth disabled={selectedRows.length <= 0} onClick={() => {
                let index = selectedRows[0]
                selectedContact(contracts && contracts.length > 0 ? contracts[index].address : "")
            }}>Confirm</Button>
        </Flex>
    </Modal>)
}