import { Info } from "@/store/types"
import { Box, Divider, Flex, LoadingOverlay, Paper, Text } from "@mantine/core"

interface Props {
    leftSection: React.ReactNode,
    label: string, 
    hide?: boolean 
    rightSection: React.ReactNode, 
    value?: string 
    valueColor?: string
    info?: Info
}
export default function BaseItem({ leftSection, label, rightSection, hide, value, valueColor, info }: Props) {
    return (
        <Flex direction={"column"} px={4}>
            <Paper shadow="xs" radius="md" p={'xs'} w={"100%"}>
                <Flex direction="row" justify="space-between" align={"center"}>
                    <Flex direction={"row"} gap={8} align={"center"}>
                        {leftSection}
                        <Text fw={600} size="md">{label}</Text>
                    </Flex>
                    {rightSection}
                </Flex>
                {
                    value && <Divider my={8} />
                }
                {
                    value &&
                    <Box pos="relative">
                        <LoadingOverlay
                            visible={hide}
                            zIndex={200}
                            overlayProps={{ radius: 'sm', blur: 3 }}
                            loaderProps={{ children: <></> }}
                        />
                        <Text px={"xl"} c={valueColor ?? ""}>
                            {value}
                        </Text>
                    </Box>
                }
                {
                    info && <Divider my={8} />
                }
                {
                    info && <Flex direction={"column"} px={"xl"} gap={4}>
                        <Text>
                            Type: {info.os_type}
                        </Text>
                        <Text>Edition: {info.edition}</Text>
                        <Text>Version: {JSON.stringify(info.version)}</Text>
                        <Text>Bitness: {info.bitness}</Text>
                        <Text>Architecture: {info.architecture}</Text>
                    </Flex>
                }
            </Paper>
        </Flex>
    )
}