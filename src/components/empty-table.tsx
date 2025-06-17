import { Flex, Center } from "@mantine/core";
import IconEmpty from "./icons/icon-empty";

const EmptyTable = ({
  height = 402,
  padding = 30,
}: {
  height?: number,
  padding?: number,
}) => {

  return (
    <Center h={height} w={"100%"} p={padding}>
      <Flex direction={"column"}>
        <IconEmpty size={60}></IconEmpty>
        <div style={{ margin: '0 auto' }}>{"No data"}</div>
      </Flex>
    </Center>
  )
}

export default EmptyTable