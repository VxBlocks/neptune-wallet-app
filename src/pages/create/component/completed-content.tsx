import { checkAuthPassword } from "@/store/auth/auth-slice";
import { useAppDispatch } from "@/store/hooks";
import { Button, Flex, Text } from "@mantine/core";

export default function CompletedContent() {
    const dispatch = useAppDispatch();
    return (<Flex direction="column" justify={"center"} align="center" gap={8} w={"100%"}>
        <Text fz={"70px"} style={{ textAlign: "center" }}>
            🎉
        </Text>
        <Text size="md" fw={600} style={{ textAlign: "center" }}>
            Congratulations!
        </Text>
        <Text fw={600} style={{ textAlign: "center" }}>
            Keep a reminder of your Secret Recovery Phrase somewhere safe. If you lose it, no one can help you get it back. Even worse, you won’t be able access to your wallet ever again.
        </Text>

        <Flex direction={"row"}
            align={"center"}
            justify={"center"} gap={8} style={{ cursor: "pointer", caretColor: "transparent", marginTop: "16px" }} w={"100%"}>
            <Button variant="light" fullWidth onClick={() => {
                dispatch(checkAuthPassword())
            }}>
                Done
            </Button>

        </Flex>

    </Flex>)
}