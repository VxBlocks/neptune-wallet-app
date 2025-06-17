import { Activity } from "@/store/types";
import { Flex, NumberFormatter, Table, Text } from "@mantine/core";

export default function CustomItem({ item }: { item: Activity }) {
    function amount_to_fixed(amount: string) {
        if (!amount) return "0"
        let len = amount.length;
        return amount.substring(0, len - 30)
    }
    return (<Flex direction={"column"} style={{ borderBottom: "1px solid #f0f0f0" }} py={8}>
        <Table
            variant="vertical"
            layout="fixed"
            withRowBorders={false}
            striped={false}
            styles={{
                th: {
                    fontSize: "16px",
                    fontWeight: "600",
                    justifyContent: "center",
                    justifyItems: "center",
                    alignItems: "center",
                    background: "transparent",
                },
                tr: {
                    fontSize: "10px",
                    fontWeight: "500",
                    justifyContent: "center",
                    justifyItems: "center",
                    alignItems: "center",
                }
            }}>
            <Table.Tbody>
                <Table.Tr>
                    <Table.Th>Block Height</Table.Th>
                    <Table.Td>
                        <Flex w={"100%"} justify={"end"}>
                            <Text c={"#0A8030"}>
                                <NumberFormatter value={item.height} thousandSeparator />
                            </Text>
                        </Flex>
                    </Table.Td>
                </Table.Tr>
                <Table.Tr>
                    <Table.Th>Index:</Table.Th>
                    <Table.Td>
                        <Flex w={"100%"} justify={"end"}>
                            <Text c={"#0A8030"}>
                                <NumberFormatter value={item.index} thousandSeparator />
                            </Text>
                        </Flex>
                    </Table.Td>
                </Table.Tr>
                <Table.Tr>
                    <Table.Th>Amount:</Table.Th>
                    <Table.Td>
                        <Flex w={"100%"} justify={"end"}>
                            <Text c={"#0A8030"}>
                                <NumberFormatter value={amount_to_fixed(item.amount)} thousandSeparator />
                            </Text>
                        </Flex>
                    </Table.Td>
                </Table.Tr>
            </Table.Tbody>
        </Table>
    </Flex>)
}