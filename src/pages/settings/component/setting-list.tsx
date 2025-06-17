import { Flex, Select, Switch } from "@mantine/core";
import BaseItem from "./base-item";
import {
    IconCirclesRelation,
    IconCube,
    IconDatabase,
    IconEye,
    IconEyeOff,
    IconFolderOpen,
    IconFolderShare,
    IconLicense,
    IconLockCog,
    IconPlugConnected,
    IconRotate,
    IconTrashX,
    IconWorld
} from "@tabler/icons-react";
import CopyedIcon from "../../../components/copyed-icon";
import { useSettingActionData } from "@/store/settings/hooks";
import { useEffect, useState } from "react";
import { LOG_LEVELS, NETWORKS } from "@/constant";
import { set_log_level } from "@/commands/log";
import { notifications } from "@mantine/notifications";
import { get_disk_cache, set_disk_cache, set_network } from "@/commands/config";
import EditRemoteIcon from "./edit-remote-icon";
import ResetPasswordIcon from "./reset-password-icon";
import { snapshot_dir } from "@/commands/app";
import { openPath } from '@tauri-apps/plugin-opener';
import ResyncIcon from "./resync-icon";
import TrashDiskIcon from "./trash-disk-icon";

export default function SettingList() {
    const { serverUrl, network, logLevel, remoteUrl } = useSettingActionData()
    const [selectedLogLevel, setSelectedLogLevel] = useState<string | null>('')

    const [selectedNetwork, setSelectedNetwork] = useState<string | null>('')
    const [dataDir, setDataDir] = useState("")
    const [checked, setChecked] = useState(false);
    useEffect(() => {
        setSelectedLogLevel(logLevel)
    }, [logLevel])
    useEffect(() => {
        setSelectedNetwork(network)
    }, [network])
    useEffect(() => {
        queryDiskCache()
        queryDataRir()
    }, [])
    async function queryDataRir() {
        let address = await snapshot_dir()
        setDataDir(address)
    }

    async function queryDiskCache() {
        let diskCache = await get_disk_cache()
        setChecked(diskCache)
    }

    async function changeDiskCache(enable: boolean) {
        try {
            await set_disk_cache(enable)
            setChecked(enable)
            notifications.show({
                position: 'top-right',
                message: "Disk cache has been changed",
                color: "green",
                title: "Success",
            })
        } catch (error: any) {
            notifications.show({
                position: 'top-right',
                color: "red",
                title: 'Failed to remove account',
                message: error || "Change  disk cache failed!",
            })
        }

    }

    async function changeLogLevel(value: string | null) {
        if (value) {
            try {
                await set_log_level(value);
                setSelectedLogLevel(value)
            } catch (error: any) {
                notifications.show({
                    position: "top-right",
                    title: "Error",
                    message: error || "Failed to change log level.",
                    color: "red",
                })
            }
        }
    }

    async function changeNetwork(value: string | null) {
        if (value) {
            try {
                await set_network(value);
                setSelectedNetwork(value)
            } catch (error: any) {
                notifications.show({
                    position: "top-right",
                    title: "Error",
                    message: error || "Failed to change network.",
                    color: "red",
                })
            }
        }
    }

    const [hideServerUrl, setHideServerUrl] = useState(true)
    return (
        <Flex direction="column" gap={16} w={"100%"}>
            <BaseItem
                leftSection={<IconCirclesRelation />}
                label={"Server url"}
                value={serverUrl}
                hide={hideServerUrl}
                rightSection={
                    <Flex direction={"row"} gap={8} align={"center"}>
                        {
                            !hideServerUrl ? <IconEyeOff style={{
                                cursor: "pointer",
                            }} size={18} onClick={() => setHideServerUrl(true)} /> : <IconEye
                                style={{
                                    cursor: "pointer",
                                }} size={18} onClick={() => setHideServerUrl(false)} />
                        }
                        <CopyedIcon value={serverUrl} />
                    </Flex>
                } />

            <BaseItem
                leftSection={<IconLicense />}
                label={"Log level"}
                rightSection={
                    <Select
                        allowDeselect
                        size="xs"
                        data={LOG_LEVELS}
                        value={selectedLogLevel}
                        onChange={changeLogLevel} />} />

            <BaseItem
                leftSection={<IconWorld />}
                label={"Network"}
                rightSection={
                    <Select
                        allowDeselect
                        size="xs"
                        data={NETWORKS}
                        value={selectedNetwork}
                        onChange={changeNetwork} />} />

            <BaseItem
                leftSection={<IconPlugConnected />}
                label={"Remote rest"}
                value={remoteUrl}
                rightSection={
                    <Flex direction={"row"} gap={8}>
                        <EditRemoteIcon value={remoteUrl} />
                        <CopyedIcon value={remoteUrl} />
                    </Flex>
                } />
            <BaseItem
                leftSection={<IconLockCog />}
                label={"Password"}
                rightSection={<ResetPasswordIcon />} />
            <BaseItem
                leftSection={<IconCube />}
                label={"Resync Block Height"}
                rightSection={<ResyncIcon/>}
            /> 

            <BaseItem
                leftSection={<IconDatabase />}
                label={"Disk Cache"}
                rightSection={
                    <Flex direction={"row"} gap={8} align={"center"}>
                        <TrashDiskIcon/>
                        <Switch
                        checked={checked}
                        onChange={(event) => changeDiskCache(event.currentTarget.checked)}
                        onLabel="ON" offLabel="OFF"
                        size="sm" />
                    </Flex>}
            />

            <BaseItem
                leftSection={<IconFolderOpen />}
                label={"Open Data Dir"}
                value={`${dataDir}`}
                rightSection={
                    <IconFolderShare
                        size={18}
                        style={{ cursor: "pointer" }}
                        onClick={async () => {
                            await openPath(dataDir);
                        }}
                    />}
            />

        </Flex>
    )
}