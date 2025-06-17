import WithTitlePageHeader from "@/components/header/withTitlePageHeader";
import { checkHasUpdateVersion, queryAboutInfo } from "@/store/about/about-slice";
import { useBuildInfo, useLoadingAbout, useUpdateVersion, useVersion } from "@/store/about/hooks";
import { useAppDispatch } from "@/store/hooks";
import { Flex, Table, Text } from "@mantine/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect } from "react";

export default function AboutPage() {
    const loading = useLoadingAbout()
    const buildInfo = useBuildInfo()
    const updateVersion = useUpdateVersion()
    const version = useVersion()
    const dispatch = useAppDispatch()
    useEffect(() => {
        dispatch(queryAboutInfo())
        dispatch(checkHasUpdateVersion())
    }, [dispatch])
    return (<WithTitlePageHeader title="Neptune wallet Service">
        <Table variant="vertical"
            layout="fixed"
            withRowBorders={false}
            striped={false} styles={{
                th: {
                    fontSize: "16px",
                    fontWeight: "600",
                    justifyContent: "center",
                    justifyItems: "center",
                    alignItems: "center",
                    background: "transparent",
                },
                tr: {
                    fontSize: "12px",
                    fontWeight: "500",
                    justifyContent: "center",
                    justifyItems: "center",
                    alignItems: "center",
                }
            }}>
            <Table.Tbody>
                <Table.Tr>
                    <Table.Th w={160}>Build time:</Table.Th>
                    <Table.Td>
                        <Text>
                            {buildInfo?.time}
                        </Text>
                    </Table.Td>
                </Table.Tr>

                <Table.Tr>
                    <Table.Th>Commit:</Table.Th>
                    <Table.Td>
                        <Text>
                            {buildInfo?.commit}
                        </Text>
                    </Table.Td>
                </Table.Tr>

                <Table.Tr>
                    <Table.Th>Version:</Table.Th>
                    <Table.Td>
                        <Flex direction={"row"} align={"center"} gap={8}>
                            <Text>
                                {version}
                            </Text>
                            {updateVersion && updateVersion.version && updateVersion.version != version &&
                                <Text style={{ color: "green" }}>
                                    {` ( New Version ${updateVersion.version} )`}
                                </Text>
                            }
                        </Flex>
                    </Table.Td>
                </Table.Tr>
                {updateVersion && updateVersion.version && updateVersion.version != version &&
                    <Table.Tr>
                        <Table.Th>Download:</Table.Th>
                        <Table.Td>
                            <Flex direction={"row"} style={{ cursor: "pointer" }} onClick={() => {
                                openUrl(updateVersion.url)
                            }}>
                                <Text style={{ color: "green" }}>
                                    {`${updateVersion.url}`}
                                </Text>
                            </Flex>
                        </Table.Td>
                    </Table.Tr>}

            </Table.Tbody>
        </Table>

    </WithTitlePageHeader>)
}