import { Flex } from "@mantine/core";

export default function BlockSyncCard() {
    // const { serverUrl } = useSettingActionData()
    // const dispatch = useAppDispatch();


    return (<Flex direction={"row"} justify="end" w={"100%"} px={30} wrap="nowrap" style={{ margin: "10px 0" }} visibleFrom="sm">
        {/*<Flex direction={"row"} gap={8} align={"center"}*/}
        {/*    style={{ cursor: "pointer" }}*/}
        {/*    onClick={() => {*/}
        {/*        dispatch(changeWalletRestartSyncBlock({ serverUrl }))*/}
        {/*        dispatch(queryLatestBlock({ serverUrl }))*/}
        {/*        dispatch(querySyncBlockStatus({ serverUrl }))*/}
        {/*    }}>*/}
        {/*    <Text>Block Synchronize</Text>*/}
        {/*    <IconReload size={20} />*/}
        {/*</Flex>*/}
    </Flex>)
}