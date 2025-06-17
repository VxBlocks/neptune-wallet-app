import WithTitlePageHeader from "@/components/header/withTitlePageHeader";
import { useState } from "react";
import { SegmentedControl, Space } from "@mantine/core";
import NewUtxoTable from "./component/new-utxo-table";
import ActivityTableCard from "./component/activity-table-card";

export default function HistoryPage() {
    const [section, setSection] = useState('activity');

    return (<WithTitlePageHeader title="History">
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