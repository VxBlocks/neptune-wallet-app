import {Flex, NumberInput, Text, TextInput} from "@mantine/core";
import {IconAddressBook, IconTrash} from "@tabler/icons-react";
import {SendInputItem} from "@/utils/api/types.ts"; 
import {useState} from "react";
import SelecteContact from "./selecte-contact";

interface Props {
    keyIndex: number,
    data: SendInputItem
    showRemove: boolean,
    onChangeToAddress: (address: string) => void
    onChangeAmount: (amount: string) => void
    onRemoveWallet?: (index: number) => void,
}

export default function TransferForm(props: Props) {
    const {keyIndex, showRemove, data, onRemoveWallet, onChangeToAddress, onChangeAmount} = props
    const [showSelectContactModal, setShowSelectContactModal] = useState(false)
    return (<Flex direction={"column"} gap={4} key={data.index}>
        <SelecteContact
            opened={showSelectContactModal}
            close={() => setShowSelectContactModal(false)}
            selectedContact={(contact) => {
                onChangeToAddress(contact)
                setShowSelectContactModal(false)
            }}/>
        <Flex direction={"row"} justify={"space-between"} align={"center"}>
            <Text style={{fontSize: "16px", fontWeight: 600}}>
                {`To Address # ${keyIndex + 1}`}
            </Text>
            <Flex direction={"row"} gap={8} align={"center"}>
                <IconAddressBook
                    style={{
                        color: "#332526"
                    }} size={20}
                    cursor={"pointer"}
                    onClick={() => setShowSelectContactModal(true)}/>
                {
                    showRemove && <IconTrash
                        style={{cursor: "pointer", color: "red"}}
                        size={14}
                        onClick={() => {
                            if (onRemoveWallet) {
                                onRemoveWallet(data.index)
                            }
                        }}/>
                }
            </Flex>
        </Flex>
        <Flex direction={"column"} gap={16}>
            <TextInput
                value={data.toAddress}
                onChange={(event) => {
                    onChangeToAddress(event.target.value.trim())
                }}
                required
                placeholder="Input to address"
            />
            <NumberInput
                placeholder="Input amount to send"
                allowNegative={false}
                value={data.amount}
                onChange={(value) => {
                    onChangeAmount(value.toString())
                }}
                required
                hideControls
            />
        </Flex>

    </Flex>)
}