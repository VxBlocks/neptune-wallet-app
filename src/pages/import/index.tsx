import { Card, Flex, Space, Stepper, Text } from "@mantine/core";
import { useState } from "react";
import CompletedContent from "../create/component/completed-content";
import ImportCreatePassword from "./component/import-create-password";
import ImportCecret from "./component/import-cecret";
import { IconChevronLeft } from "@tabler/icons-react";
interface Props {
    onBack: () => void
}
export default function ImportPage(props: Props) {
    const { onBack } = props;
    const [active, setActive] = useState(0);
    const nextStep = () => setActive((current) => (current < 3 ? current + 1 : current));
    const prevStep = () => setActive((current) => (current > 0 ? current - 1 : current));

    return (<Card shadow="sm"
        radius="md"
        p={"lg"} withBorder
        style={{
            width: "600px",
            minWidth: "600px"
        }}>
        {
            active <= 1 && <Flex direction={"column"} >
                <Flex
                    direction={"row"}
                    style={{ cursor: "pointer" }}
                    onClick={() => {
                        if (active == 0) {
                            onBack()
                        } else {
                            prevStep();
                        }
                    }}
                >
                    <IconChevronLeft size={21} />
                    <Text>Go Back</Text>
                </Flex>
                <Space h={16} />
            </Flex>
        }
        <Stepper size="xs"
            iconSize={24}
            active={active}
            onStepClick={setActive}
            allowNextStepsSelect={false}
            styles={{
                steps: {
                    padding: "10px 30px",
                }
            }}
            style={{ justifyContent: "center", alignItems: "center" }}>
            <Stepper.Step label="First step" description="Create password">
                <ImportCreatePassword nextStep={nextStep} />
            </Stepper.Step>
            <Stepper.Step label="Second step" description="Secure wallet">
                <ImportCecret nextStep={nextStep} />
            </Stepper.Step>
            <Stepper.Completed>
                <CompletedContent />
            </Stepper.Completed>
        </Stepper>
    </Card>)
}