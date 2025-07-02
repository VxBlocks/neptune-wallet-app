import WithTitlePageHeader from "@/components/header/withTitlePageHeader";
import { useState } from "react";
import { Flex, SegmentedControl, Space, Text } from "@mantine/core";
import NewUtxoTable from "./component/new-utxo-table";
import ActivityTableCard from "./component/activity-table-card";
import { useActivityPerDay } from "@/store/history/hooks";

export default function HistoryPage() {
    const [section, setSection] = useState('activity');
    const perDay = useActivityPerDay();

    return (<WithTitlePageHeader title="History">
        {perDay && perDay.length > 0 && <Flex direction={"column"} gap={8}>
            {
                perDay.map((day, index) => {
                    return <Flex key={index} direction={"row"} gap={8}>
                        <Text>Receive: </Text>
                        <Text c={"green"}>{day.r_total}</Text>
                        <Text>Send: </Text>
                        <Text c={"red"}>{day.s_total}</Text>
                        <Text> {day.data}</Text>
                    </Flex>
                })
            }
        </Flex>}
        <SegmentedControl
            value={section}
            onChange={(value: any) => setSection(value)}
            transitionTimingFunction="ease"
            fullWidth
            data={[
                { label: 'Activity', value: 'activity' },
                { label: 'Utxos', value: 'utxos' },
            ]}
        />
        <Space h={16}></Space>
        {section === "activity" && <ActivityTableCard />}
        {section === "utxos" && <NewUtxoTable />}

    </WithTitlePageHeader>)
}