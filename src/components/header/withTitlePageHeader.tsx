import { Container, Divider, Flex, Space, Text } from "@mantine/core";

export default function WithTitlePageHeader({ children, title, buttons }: { children: React.ReactNode | React.ReactNode[], title: string, buttons?: React.ReactNode }) {
    return (<Container fluid w={"100%"}>
        <Flex direction={"column"} px={24} w={"100%"}>
            <Space h={16} />
            <Flex direction={"column"} gap={2}>
                <Flex direction={"row"} justify={"space-between"} align={"center"}>
                    <Text fw={500} fz={24}>
                        {title}
                    </Text>
                    {buttons ? buttons : null}
                </Flex>
                <Divider />
            </Flex>
            <Space h={16} />
            {children}
        </Flex>
    </Container>)
}