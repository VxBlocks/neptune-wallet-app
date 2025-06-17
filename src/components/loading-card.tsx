import { Center, Flex, Loader } from "@mantine/core";

export default function LoadingPage() {
    return (<Center h={"100vh"} w={"100%"}>
        <Flex direction={"column"}>
            <Loader />
        </Flex>
    </Center>)
}