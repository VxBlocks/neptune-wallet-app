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

    // 计算数据最大值的1.2倍
    const getYAxisMax = () => {
        if (!perDay || perDay.length === 0) return 'auto';
        const maxValue = Math.max(...perDay.flatMap(item => 
            [item.Received || 0, item.Spent || 0]
        ));
        return maxValue * 1.2;
    };

    // Y轴刻度格式化函数
    const formatYAxisTick = (value: number) => {
        if (value >= 1000000) {
            return `${(value / 1000000).toFixed(1)}m`;
        } else if (value >= 1000) {
            return `${(value / 1000).toFixed(1)}k`;
        }
        return value.toString();
    };

    return (<WithTitlePageHeader title="History">
        {
            perDay && perDay.length > 0 && <BarChart
                h={250}
                data={perDay}
                yAxisProps={{ 
                    domain: [0, getYAxisMax()],
                    tickFormatter: formatYAxisTick
                }}
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