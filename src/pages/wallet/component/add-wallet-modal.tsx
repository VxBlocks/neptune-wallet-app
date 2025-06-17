import { Flex, Modal, SegmentedControl } from "@mantine/core";
import { useEffect, useState } from "react";
import CreateWallet from "./create-wallet";
import ImportWallet from "./import-wallet";
import { useAppDispatch } from "@/store/hooks";
import { queryWalletBalance, queryWallets } from "@/store/wallet/wallet-slice";
import { querySyncBlockStatus } from "@/store/sync/sync-slice";
import { useSettingActionData } from "@/store/settings/hooks";
import * as bip39 from '@scure/bip39';
import { wordlist } from '@scure/bip39/wordlists/english';
import { notifications } from "@mantine/notifications";

export default function AddWalletModal({ opened, onClose }: { opened: boolean, onClose: () => void }) {
    const [section, setSection] = useState('create');
    const dispatch = useAppDispatch()
    const { serverUrl } = useSettingActionData()
    const [mnemonic, setMnemonic] = useState('')
    async function onCreated() {
        dispatch(queryWallets())
        dispatch(queryWalletBalance({ serverUrl }))
        dispatch(querySyncBlockStatus({ serverUrl }))
        onClose()
    }
    useEffect(() => {
        if (section === 'create') {
            let mnemonic = bip39.generateMnemonic(wordlist, 192)
            setMnemonic(mnemonic)
        }
    }, [section, opened])
    return (<Modal opened={opened} size={"lg"} centered onClose={onClose} title="Add account">
        <Flex direction="column">
            <SegmentedControl
                value={section}
                onChange={(value: any) => setSection(value)}
                transitionTimingFunction="ease"
                fullWidth
                data={[
                    { label: 'Create Account', value: 'create' },
                    { label: 'Import Account', value: 'import' },
                ]}
            />
            {section === 'create' && <CreateWallet mnemonic={mnemonic} onCreated={onCreated} refreshMnemonic={() => {
                let mnemonic = bip39.generateMnemonic(wordlist, 192)
                setMnemonic(mnemonic)
                notifications.show({
                    position: "top-right",
                    title: "Success",
                    message: "New seed phrase generated",
                    color: "green",
                })
            }} />}
            {section === 'import' && <ImportWallet onCreated={async () => onCreated()} />}
        </Flex>
    </Modal>)
}