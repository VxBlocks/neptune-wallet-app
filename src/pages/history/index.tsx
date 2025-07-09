import WithTitlePageHeader from "@/components/header/withTitlePageHeader";
import { useState } from "react";
import { SegmentedControl, Space } from "@mantine/core";
import NewUtxoTable from "./component/new-utxo-table";
import ActivityTableCard from "./component/activity-table-card";
import { useActivityPerDay } from "@/store/history/hooks";
import { BarChart } from '@mantine/charts';

export default function HistoryPage() {
    const [section, setSection] = useState('activity');
    const perDay = useActivityPerDay();

    return (<WithTitlePageHeader title="History">
        {
            perDay && perDay.length > 0 && <BarChart
                h={150}
                data={perDay}
                yAxisProps={{ domain: [0, 'auto'] }}
                dataKey="data"
                withTooltip={false}
                valueFormatter={(value) => new Intl.NumberFormat('en-US').format(Math.floor(value))}
                withBarValueLabel
                valueLabelProps={{ fill: 'teal' }}
                style={{ marginBottom: 10 }}
                series={[
                    { name: 'Received', color: 'violet.6' },
                    { name: 'Spent', color: 'teal.6' },
                ]}
            />
            
        }
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