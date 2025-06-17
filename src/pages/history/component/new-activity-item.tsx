import CopyedIcon from "@/components/copyed-icon";
import { MerageHistory } from "@/store/types";
import { bigNumberPlusToString } from "@/utils/common";
import { ellipsisFormatLen } from "@/utils/ellipsis-format";
import { amount_to_positive_fixed } from "@/utils/math-util";
import { Accordion, Flex, Group, NumberFormatter, Table, Text } from "@mantine/core";
import { IconTransfer } from "@tabler/icons-react";
import { format } from "date-fns";

interface Props {
    keyIndex: number
    item: MerageHistory
}

export default function NewActivityItem(props: Props) {
    const { item, keyIndex } = props;
    function handleAmount() {
        let amount = "0"
        if (item && item.batchOutput && item.batchOutput.length > 0) {
            item.batchOutput.forEach((output) => {
                amount = bigNumberPlusToString(amount, output.amount)
            })
        }
        return amount_to_positive_fixed(amount)
    }

    function AccordionLabel(item: MerageHistory) {
        return (
            <Group wrap="nowrap">
                <IconTransfer />
                <div>
                    <Flex direction={"row"} gap={24}>
                        <Text c={"#0A8030"}>
                            Height: <NumberFormatter value={item.height} thousandSeparator />
                        </Text>
                    </Flex>
                    <Text size="sm" c="dimmed" fw={400}>
                        {item.message}
                    </Text>
                </div>
            </Group>
        );
    }

    function AccordionPanel(item: MerageHistory) {
        return (
            <Group wrap="nowrap" px={16}>
                <Table
                    variant="vertical"
                    layout="fixed"
                    withRowBorders={false}
                    striped={false}
                    styles={{
                        th: {
                            fontSize: "14px",
                            fontWeight: "600",
                            justifyContent: "center",
                            justifyItems: "center",
                            alignItems: "center",
                            background: "transparent",
                            verticalAlign: "top",
                        },
                        tr: {
                            fontSize: "10px",
                            fontWeight: "500",
                            justifyContent: "center",
                            justifyItems: "center",
                            alignItems: "center",
                            verticalAlign: "top",
                        }
                    }}>
                    <Table.Tbody>
                         {
                            item.txid ?
                                <Table.Tr>
                                    <Table.Th>Tx:</Table.Th>
                                    <Table.Td>
                                        <Flex direction={"row"}
                                            gap={8} align={"center"} justify={"end"}>
                                            <Text>
                                                {ellipsisFormatLen(item.txid, 15)}
                                            </Text>
                                            <CopyedIcon size={16} value={item.txid} />
                                        </Flex>
                                    </Table.Td>

                                </Table.Tr> : null

                        }
                        <Table.Tr>
                            <Table.Th>Time:</Table.Th>
                            <Table.Td>
                                <Flex w={"100%"} justify={"end"}>
                                    <Text c={"#0A8030"}>
                                        {format(item.timestamp, 'yyyy-MM-dd HH:mm:ss')}
                                    </Text>
                                </Flex>
                            </Table.Td>
                        </Table.Tr>
                        <Table.Tr>
                            <Table.Th>Index:</Table.Th>
                            <Table.Td>
                                <Flex align={"end"} direction={"column"} gap={8}>
                                    <Flex w={"100%"} justify={"end"}>
                                        <Text c={"#0A8030"}>
                                            <NumberFormatter value={item.index} thousandSeparator />
                                        </Text>
                                    </Flex>
                                </Flex>

                            </Table.Td>
                        </Table.Tr> 
                        {
                            item.batchOutput ?
                                <Table.Tr>
                                    <Table.Th w={100}>To Address:</Table.Th>
                                    <Table.Td style={{
                                        wordWrap: "break-word",
                                        overflowWrap: "break-word",
                                    }}>
                                        <Flex direction={"column"} gap={8}>
                                            {
                                                item.batchOutput?.map((output, index) => {
                                                    return (
                                                        <Flex align={"end"} direction={"column"} gap={8} key={index}>
                                                            <Flex direction={"row"} gap={8} align={"center"}>
                                                                <Text>
                                                                    {ellipsisFormatLen(output.toAddress, 15)}
                                                                </Text>
                                                                <CopyedIcon size={16} value={output.toAddress} />
                                                            </Flex>

                                                        </Flex>
                                                    )
                                                })
                                            }
                                        </Flex>
                                    </Table.Td>
                                </Table.Tr> : null
                        }
                        {
                            item.batchOutput ?
                                <Table.Tr>
                                    <Table.Th>Amount:</Table.Th>
                                    <Table.Td>
                                        <Flex w={"100%"} justify={"end"}>
                                            <Text c={"#0A8030"}>
                                                <NumberFormatter value={handleAmount()} thousandSeparator />
                                            </Text>
                                        </Flex>
                                    </Table.Td>
                                </Table.Tr> : null
                        }
                        {
                            item.fee ?
                                <Table.Tr>
                                    <Table.Th>Fee:</Table.Th>
                                    <Table.Td>
                                        <Flex w={"100%"} justify={"end"}>
                                            <Text c={"#0A8030"}>
                                                <NumberFormatter value={amount_to_positive_fixed(item.fee)} thousandSeparator />
                                            </Text>
                                        </Flex>
                                    </Table.Td>
                                </Table.Tr> : null
                        }
                        {
                            item.priorityFee ?
                                <Table.Tr>
                                    <Table.Th>Priority Fee:</Table.Th>
                                    <Table.Td>
                                        <Flex w={"100%"} justify={"end"}>
                                            <Text c={"#0A8030"}>
                                                <NumberFormatter
                                                    value={amount_to_positive_fixed(item.priorityFee)}
                                                    thousandSeparator />
                                            </Text>
                                        </Flex>
                                    </Table.Td>
                                </Table.Tr> : null
                        }
                    </Table.Tbody>
                </Table>
            </Group>
        );
    }


    return (<Accordion.Item value={keyIndex.toString()}>
        <Accordion.Control>
            <AccordionLabel {...item} />
        </Accordion.Control>
        <Accordion.Panel>
            <AccordionPanel {...item} />
        </Accordion.Panel>
    </Accordion.Item>)
}