import { HistoryUtxo } from "@/store/types";
import { amount_to_positive_fixed } from "@/utils/math-util";
import { Table, NumberFormatter, Text } from "@mantine/core";

export default function HistoryUtxoCard({ datas }: { datas: HistoryUtxo[] }) {
    const rows = datas.map((item, index) => (
        <Table.Tr key={index}>
            <Table.Td>
                <Text>
                    <NumberFormatter value={item.id} thousandSeparator />
                </Text>
            </Table.Td>
            <Table.Td>
                <Text c={Number(item.amount) > 0 ? "green" : "red"}>
                    {Number(item.amount) > 0 ? "+ " : "- "} <NumberFormatter value={amount_to_positive_fixed(item.amount)} thousandSeparator />
                </Text>
            </Table.Td>
        </Table.Tr>
    ));

    return (
        <Table striped highlightOnHover stickyHeaderOffset={0} stickyHeader verticalSpacing={"sm"} withRowBorders={false} tabularNums>
            <Table.Thead>
                <Table.Tr>
                    <Table.Th>ID</Table.Th>
                    <Table.Th>Amount</Table.Th>
                </Table.Tr>
            </Table.Thead>
            <Table.Tbody>{rows}</Table.Tbody>
        </Table>
    );
}