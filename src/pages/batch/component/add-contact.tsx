import { Contact } from "@/database/types/contact";
import { queryAllContacts } from "@/store/contact/contact-slice";
import { useAppDispatch } from "@/store/hooks";
import { addContactAddress } from "@/utils/storage"; 
import { Button, Flex, Modal, Textarea, TextInput, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useState } from "react";

export default function AddContact({ opened, close }: { opened: boolean, close: () => void }) {
    const [contact, setContact] = useState({
        aliasName: "",
        address: "",
        remark:"",
        type:"",
        createdTime:0
    } as Contact)
    const [loading, setLoading] = useState(false)
    const dispatch = useAppDispatch()

    async function handleSubmit() {
        try {
            setLoading(true)   
            contact.createdTime = new Date().getTime()
            addContactAddress({ contact })
            notifications.show({
                position: "top-right",
                message: "Contact added successfully",
                color: "green",
            })
            dispatch(queryAllContacts())
            close()
        } catch (error: any) {
            console.error(error)
            notifications.show({
                position: "top-right",
                message: error || "Failed to add contact",
                color: "red",
            })
        }
        setLoading(false)
    }
    return (<Modal opened={opened} onClose={close} title="Add Contact">
        <Flex direction="column" gap="md">
            <TextInput
                data-autofocus
                label="Name"
                value={contact.aliasName ?? ""}
                onChange={(event) => setContact({ ...contact, aliasName: event.target.value })}
                placeholder="Enter a name for address"
            />
            <Flex direction={"column"}>
                <Flex direction={"row"} justify={"space-between"}>
                    <Flex direction={"row"} gap={4}>
                        <Text>Address</Text>
                        <Text c="var(--input-asterisk-color, var(--mantine-color-error))">*</Text>
                    </Flex>
                </Flex>
                <Textarea
                    placeholder="Enter a public address"
                    autosize
                    minRows={4}
                    value={contact.address ?? ""}
                    onChange={(event) =>
                        setContact({
                            ...contact,
                            address: event.target.value,
                        })
                    }
                />
            </Flex>
            <Button variant="light" loading={loading} disabled={!contact.aliasName || !contact.address} onClick={handleSubmit}>Add</Button>
        </Flex>
    </Modal>)
}