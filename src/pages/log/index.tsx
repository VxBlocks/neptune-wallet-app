import WithTitlePageHeader from "@/components/header/withTitlePageHeader";
import {useAppDispatch} from "@/store/hooks";
import {useLogs} from "@/store/log/hooks";
import {queryLogMessages} from "@/store/log/log-slice";
import {Button, Flex, ScrollArea} from "@mantine/core";
import {useEffect, useRef, useState} from "react";
import Ansi from "ansi-to-react";
import {clear_logs} from "@/commands/log";

export default function LogPage() {
    const dispatch = useAppDispatch();
    const logs = useLogs();
    let timerId: any = null;
    const [isAtBottom, setIsAtBottom] = useState(false);
    useEffect(() => {
        timerId = setInterval(async () => {
            dispatch(queryLogMessages())
        }, 100);
        setTimeout(() => {
            scrollToBottom()
        }, 500)
        return () => {
            clearInterval(timerId)
        }
    }, [])
    useEffect(() => {
        if (isAtBottom) {
            scrollToBottom()
            setIsAtBottom(true)
        }
    }, [logs])
    const viewport = useRef<HTMLDivElement>(null);

    const scrollToBottom = () =>
        viewport.current!.scrollTo({top: viewport.current!.scrollHeight, behavior: 'smooth'});


    const handleScroll = ({y}: { x: number; y: number }) => {
        const scrollArea = document.querySelector('.mantine-ScrollArea-viewport');
        if (!scrollArea) return;
        const {scrollHeight, clientHeight, scrollTop} = scrollArea;
        const isBottom = scrollHeight - (scrollTop + clientHeight) < 20;
        setIsAtBottom(isBottom);
    };

    return (
        <WithTitlePageHeader
            title="Log" buttons={<Button size="xs" variant="light" onClick={async () => {
            await clear_logs()
        }}>Clear logs</Button>}>
            <ScrollArea h={"calc(100vh - 80px)"} scrollbarSize={8} viewportRef={viewport}
                        onScrollPositionChange={handleScroll}>
                <Flex direction="column" gap="16"
                      style={{fontSize: '14px', wordWrap: "break-word", wordBreak: "break-all"}}>
                    {logs && logs.length > 0 && logs.map((log, index) => {
                        return <Ansi key={index}>{log}</Ansi>
                    })}
                </Flex>
            </ScrollArea>
        </WithTitlePageHeader>)
}