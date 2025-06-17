import CopyedIcon from "@/components/copyed-icon";
import { MerageHistory } from "@/store/types";
import { ellipsisFormatLen } from "@/utils/ellipsis-format";
import { amount_to_positive_fixed } from "@/utils/math-util";
import { Divider, Flex, FocusTrap, Modal, NumberFormatter, ScrollArea, Table, Text } from "@mantine/core";
import { format } from "date-fns";
import "./index.css";
import HistoryUtxoCard from "./history-utxo";
import { bigNumberPlusToString } from "@/utils/common";

interface Props {
    history: MerageHistory,
    opened: boolean,
    onClose: () => void
}
export default function DetailModal(props: Props) {
    const { history, opened, onClose } = props
    function handleAmount() {
        let amount = "0"
        if (history && history.batchOutput && history.batchOutput.length > 0) {
            history.batchOutput.forEach((output) => {
                amount = bigNumberPlusToString(amount, output.amount)
            })
        }
        return amount_to_positive_fixed(amount)
    }
    return (<Modal
        centered
        opened={opened}
        size="lg"
        onClose={onClose}
        title={"History Detail"}
        scrollAreaComponent={ScrollArea.Autosize}
    >
        <FocusTrap.InitialFocus />
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
                    history && history.txid ?
                        <Table.Tr>
                            <Table.Th w={80}>Tx:</Table.Th>
                            <Table.Td style={{
                                wordWrap: "break-word",
                                overflowWrap: "break-word",
                            }}>
                                <Flex align={"end"} direction={"column"} gap={8}>
                                    <Flex direction={"row"} gap={8} align={"center"}>
                                        <Text c={"#0A8030"}>
                                            {ellipsisFormatLen(history.txid, 15)}
                                        </Text>
                                        <CopyedIcon size={16} value={history.txid} />
                                    </Flex>

                                </Flex>
                            </Table.Td>
                        </Table.Tr> : null
                }
                {
                    history && history.height ?
                        <Table.Tr>
                            <Table.Th>Height:</Table.Th>
                            <Table.Td>
                                <Flex w={"100%"} justify={"end"}>
                                    <Text c={"#0A8030"}>
                                        <NumberFormatter value={history.height} thousandSeparator />
                                    </Text>
                                </Flex> 
                            </Table.Td> 
                        </Table.Tr> : null

                }
                {
                    history.batchOutput && history.batchOutput.length > 0 ?
                        <Table.Tr>
                            <Table.Th w={80}>To:</Table.Th>
                            <Table.Td style={{
                                wordWrap: "break-word",
                                overflowWrap: "break-word",
                            }}>
                                <Flex direction={"column"} gap={8}>
                                    {
                                        history.batchOutput?.map((output, index) => {
                                            return (
                                                <Flex align={"end"} direction={"column"} gap={8} key={index}>
                                                    <Flex direction={"row"} gap={8} align={"center"}>
                                                        <Text c={"#0984c3"}>
                                                            {ellipsisFormatLen(output.toAddress, 10)}
                                                        </Text>
                                                        <CopyedIcon size={16} value={output.toAddress} />
                                                        <Flex direction={"row"} gap={3}>
                                                            <Text c={"#6c757d"}>
                                                                {`(Sent: `}
                                                            </Text>
                                                            <Text c={"#0A8030"}>
                                                                <NumberFormatter value={amount_to_positive_fixed(output.amount)} thousandSeparator />
                                                            </Text>
                                                            <Text c={"#6c757d"}>
                                                                {`)`}
                                                            </Text>
                                                        </Flex>
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
                    history.batchOutput && history.batchOutput.length > 1 ?
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
                    history.outputs && history.outputs.length > 0 ?
                        <Table.Tr>
                            <Table.Th w={80}>Outputs:</Table.Th>
                            <Table.Td style={{
                                wordWrap: "break-word",
                                overflowWrap: "break-word",
                            }}>
                                <Flex direction={"column"} gap={8}>
                                    {
                                        history.outputs?.map((output, index) => {
                                            return (
                                                <Flex align={"end"} direction={"column"} gap={8} key={index}>
                                                    <Flex direction={"row"} gap={8} align={"center"}>
                                                        <Text>
                                                            {ellipsisFormatLen(output, 15)}
                                                        </Text>
                                                        <CopyedIcon size={16} value={output} />
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
                    history.fee ?
                        <Table.Tr>
                            <Table.Th w={80}>Fee:</Table.Th>
                            <Table.Td>
                                <Flex w={"100%"} justify={"end"}>
                                    <Text c={"#0A8030"}>
                                        <NumberFormatter value={amount_to_positive_fixed(history.fee)} thousandSeparator />
                                    </Text>
                                </Flex>
                            </Table.Td>
                        </Table.Tr> : null
                }
                <Table.Tr>
                    <Table.Th w={80}>Time:</Table.Th>
                    <Table.Td>
                        <Flex w={"100%"} justify={"end"}>
                            <Text c={"#0A8030"}>
                                {format(history && history.timestamp ? history.timestamp : "0", 'yyyy-MM-dd HH:mm:ss')}
                            </Text>
                        </Flex>
                    </Table.Td>
                </Table.Tr>
            </Table.Tbody>
        </Table>
        <Divider my={16} mx={8} />
        <Flex direction={"column"} gap={8}>
            <Text style={{ fontWeight: "bold", fontSize: "16px" }}>
                {"Utxo Changes"}
            </Text>
            <HistoryUtxoCard datas={history.utxos} />
        </Flex>

    </Modal>)
}