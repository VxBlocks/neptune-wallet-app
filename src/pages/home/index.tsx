import { Button, Card, Center, Flex, Group, Text, Image } from "@mantine/core";
import { useState } from "react";
import CreatePage from "../create";
import ImportPage from "../import";

export default function HomeScreen() {
    const [activityPage, setActivePage] = useState("")
    function onBackFunction() {
        setActivePage("")
    }
    return (<Center w={"100%"} h={"100vh"}>
        {
            activityPage === "create" && <CreatePage onBack={onBackFunction} />
        }
        {
            activityPage === "import" && <ImportPage onBack={onBackFunction} />
        }
        {
            activityPage === "" && <Card shadow="sm" radius="md" withBorder w={500} h={400}>
                <Group justify="center" mb="xs">
                    <Text fz="lg" fw={"800"}>Neptune Wallet</Text>
                </Group>
                <Flex justify={"center"} align={"center"}>
                    <Image
                        data-tauri-drag-region
                        src={"/logo.png"}
                        w={"100%"}
                        h={129}
                    />
                </Flex>
                <Flex direction="column" w={"100%"} gap={16} justify={"center"} align={"center"} style={{
                    position: "absolute",
                    bottom: 16,
                    left: 0,
                    width: "100%"
                }}>
                    <Button variant="light" color="blue" fullWidth w={"60%"} radius={"md"} onClick={() => setActivePage("create")}>
                        Create a new wallet
                    </Button>
                    <Button variant="default" color="blue" fullWidth w={"60%"} radius={"md"} onClick={() => setActivePage("import")}>
                        Import an existing wallet
                    </Button>
                </Flex>
            </Card>
        }
    </Center>)
}