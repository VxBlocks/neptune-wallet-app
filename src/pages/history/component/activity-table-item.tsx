import { MerageHistory } from "@/store/types";
import { Table, Center, NumberFormatter, Text } from "@mantine/core";
import { format } from "date-fns";
import "./index.css"

interface Props {
    element: MerageHistory
    showMoreDetail: () => void
}
export default function ActivityTableItem(props: Props) {
    const { element, showMoreDetail } = props
    return (
        <Table.Tr>
            <Table.Td>
                <Text c={"#0A8030"}>
                    <NumberFormatter value={element.height} thousandSeparator />
                </Text>
            </Table.Td>
            <Table.Td>
                <Center>
                    {element.changeAmount.startsWith("-") ?
                        <Text fw={600} c={"red"}>
                            {element.changeAmount}
                        </Text> : <Text fw={600} c={"green"}>
                            {element.changeAmount}
                        </Text>}
                </Center>
            </Table.Td>
            <Table.Td>
                <Center>
                    <Text c={"#0A8030"}>
                        {format(element.timestamp, 'yyyy-MM-dd HH:mm:ss')}
                    </Text>
                </Center>
            </Table.Td>
            <Table.Td>
                <Text className="more" onClick={() => showMoreDetail()}>
                    More
                </Text>
            </Table.Td>
        </Table.Tr>
    )
}