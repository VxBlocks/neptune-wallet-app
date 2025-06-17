import CopyedIcon from "@/components/copyed-icon";
import EmptyTable from "@/components/empty-table";
import { queryAllContacts } from "@/store/contact/contact-slice";
import { useAllContacts, useLoadingContacts } from "@/store/contact/hooks";
import { useAppDispatch } from "@/store/hooks";
import { ellipsis } from "@/utils/ellipsis-format";
import { Box, Button, Center, Flex, LoadingOverlay, ScrollArea, Table, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { IconTrash } from "@tabler/icons-react";
import { useState } from "react";
import AddContact from "./add-contact";
import { deleteContactAddress } from "@/utils/storage";

export default function ContactTable() {
    const loading = useLoadingContacts()
    const contracts = useAllContacts()
    const dispatch = useAppDispatch()
    const [showAddContact, setShowAddContact] = useState(false)
    async function handleDelete(address: string) {
        try {
            await deleteContactAddress({ address });
            dispatch(queryAllContacts())
            notifications.show({
                position: "top-right",
                title: "Success",
                message: "Delete contact successfully",
                color: "green",
            })
        } catch (error: any) {
            notifications.show({
                position: "top-right",
                title: "Error",
                message: error || "Delete contact failed",
                color: "red",
            })
        }

    }
    const rows = contracts && contracts.map((element) => (
        <Table.Tr key={element.address}>
            <Table.Td>
                <Text style={{ minWidth: "115px" }}>{element.aliasName}</Text>
            </Table.Td>
            <Table.Td>
                <Flex direction={"row"} gap={8} align={"center"}>
                    <Text>
                        {ellipsis(element.address)}
                    </Text>
                    <CopyedIcon tooltipLable="Copy Address" size={16} value={element.address} />
                </Flex>
            </Table.Td>
            <Table.Td>
                <Center>
                    <Flex direction={"row"}>
                        <IconTrash size={14} color={element.type === "owner" ? "grey" : "red"} style={{ cursor: element.type === "owner" ? "not-allowed" : "pointer" }}
                            onClick={() => element.type === "owner" ? null : handleDelete(element.address)}></IconTrash>
                    </Flex>
                </Center>
            </Table.Td>
        </Table.Tr>
    ));
    return (<Flex direction={"column"}>
        <AddContact opened={showAddContact} close={() => setShowAddContact(false)} />
        <Flex direction={"row"}>
            <Button variant="light" data-autofocus size={"xs"} onClick={() => setShowAddContact(true)}>Add Contact</Button>
            <div data-autofocus></div>
        </Flex>
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
                                    <Table.Th>Name</Table.Th>
                                    <Table.Th>Address</Table.Th>
                                    <Table.Th>Operate</Table.Th>
                                </Table.Tr>
                            </Table.Thead>
                            <Table.Tbody>{rows}</Table.Tbody>
                        </Table>
                    </ScrollArea> : <EmptyTable />
            }

        </Box>
    </Flex>)
}