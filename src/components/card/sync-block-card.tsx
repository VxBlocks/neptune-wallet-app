import { Card, Flex, NumberFormatter, Progress, Space, Text } from '@mantine/core';
import classes from './sync.module.css';
import { IconMeteor } from '@tabler/icons-react';
import { useLatestBlock, useSyncedBlock } from '@/store/sync/hooks';
import { useEffect } from 'react';
import { useAppDispatch } from '@/store/hooks';
import { queryLatestBlock } from '@/store/sync/sync-slice';
import { useSettingActionData } from '@/store/settings/hooks';
export default function SyncBlockCard() {
    const { serverUrl } = useSettingActionData()
    const syncedBlock = useSyncedBlock();
    const latestBlock = useLatestBlock();
    const dispatch = useAppDispatch()
    function handleProgress() {
        if (latestBlock === 0) {
            return 0
        }
        return (syncedBlock / latestBlock) * 100
    }
    useEffect(() => {
        if (latestBlock < syncedBlock) {
            dispatch(queryLatestBlock({ serverUrl }))
        }
    }, [syncedBlock, latestBlock])
    return (<Flex style={{ width: "100%" }}>
        <Card className={classes.card}>
            <Flex direction={"row"} justify={"space-between"}>
                <Flex direction={"row"}>
                    <IconMeteor color='#fa6800' size={16} />
                    <Space w={5} />
                    <Text fz={'xs'} fw={"bold"} c={"#FFFFFF"}>
                        sync status
                    </Text>
                </Flex>
                <Flex direction={"row"} gap={2}>
                    <Text fz={'xs'} fw={"bold"} c={"white"}>
                        <NumberFormatter value={(syncedBlock ?? 0) / (latestBlock ?? 0) * 100} decimalScale={1} suffix='%' />
                    </Text>
                </Flex>
            </Flex>
            <Space h={8}></Space>
            <Progress
                value={handleProgress()}
                size="4"
                animated={latestBlock != 0 && syncedBlock != latestBlock}
                radius="xl"
                classNames={{
                    root: classes.progressTrack,
                    section: classes.progressSection,
                }}
            />
            <Space h={8}></Space>
            <Flex direction={"row"} gap={2}>
                <Text fz={'xs'} fw={"bold"} c={"var(--primaryhighlight2)"}>
                    <NumberFormatter value={syncedBlock ?? 0} thousandSeparator />
                </Text>
                <Text fz={'xs'} fw={"bold"} c={"#FFFFFF"}>
                    /
                </Text>
                <Text fz={'xs'} fw={"bold"} c={"#FFFFFF"}>
                    <NumberFormatter value={latestBlock ?? 0} thousandSeparator />
                </Text>
            </Flex>

        </Card>

    </Flex>)
}